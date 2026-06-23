use std::collections::HashMap;

use async_trait::async_trait;
use nom::{
    IResult, Parser,
    error::{ContextError, ParseError, context},
};
use nom_language::error::VerboseError;
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            packet::{
                self,
                inbound::{FromPacketBody, TryToPacket},
                outbound::ToPacket,
                parsing::take_tlv_fields,
            },
            packet_manager::PacketHandler,
            structures::{
                BatteryLevel, CaseBatteryLevel, DualBattery, DualFirmwareVersion, FirmwareVersion,
                HostDevice, IsBatteryCharging, LimitHighVolume, SerialNumber, SingleBattery,
                TwsStatus,
            },
        },
        d1204::{
            state::D1204State,
            structures::{
                D1204ButtonAction, D1204Buttons, D1204Equalizer, DolbyAudio, EasyChat,
                ManualNoiseCanceling, SoundMode,
            },
        },
    },
};

/// Tags found in the Liberty 5 Pro Max (D1204) state update packet.
///
/// The packet uses a Tag-Length-Value layout. Only the tags that have been
/// decoded so far are named here; the remaining tags are currently ignored.
/// As more tags are decoded (see the differential capture guide), add them
/// here and wire them into [`D1204StateUpdatePacket`].
mod tag {
    pub const TWS_HOST_DEVICE: u8 = 1;
    pub const TWS_CONNECTED: u8 = 2;
    pub const BATTERY_LEFT: u8 = 3;
    pub const BATTERY_RIGHT: u8 = 4;
    pub const FIRMWARE_LEFT: u8 = 5;
    pub const FIRMWARE_RIGHT: u8 = 6;
    pub const SERIAL_NUMBER: u8 = 7;
    pub const CASE_BATTERY: u8 = 8;
    pub const EQUALIZER_PRESET: u8 = 11;
    pub const EQUALIZER_CURVE: u8 = 12;
    pub const BUTTON_LEFT_SINGLE: u8 = 13;
    pub const BUTTON_RIGHT_SINGLE: u8 = 14;
    pub const BUTTON_LEFT_DOUBLE: u8 = 15;
    pub const BUTTON_RIGHT_DOUBLE: u8 = 16;
    pub const BUTTON_LEFT_TRIPLE: u8 = 17;
    pub const BUTTON_RIGHT_TRIPLE: u8 = 18;
    pub const BUTTON_LEFT_LONG: u8 = 19;
    pub const BUTTON_RIGHT_LONG: u8 = 20;
    // Small feature bitfield: [8, flags, dolby_echo]. EasyChat is bit 0x40 of flags.
    pub const SMART_FEATURES: u8 = 27;
    pub const SOUND_MODE: u8 = 36;
    pub const MANUAL_NOISE_CANCELING: u8 = 37;
    pub const LIMIT_HIGH_VOLUME: u8 = 39;
    pub const DOLBY_AUDIO: u8 = 44;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct D1204StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub dual_battery: DualBattery,
    pub case_battery: CaseBatteryLevel,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub equalizer: D1204Equalizer,
    pub buttons: D1204Buttons,
    pub dolby_audio: DolbyAudio,
    pub easy_chat: EasyChat,
    pub sound_mode: SoundMode,
    pub manual_noise_canceling: ManualNoiseCanceling,
    pub limit_high_volume: LimitHighVolume,
}

fn parse_single_battery(value: Option<&[u8]>) -> SingleBattery {
    // Observed layout: [is_charging, level_percent]. Battery is reported as a
    // percentage (0-100) rather than the 0-10 scale used by older devices.
    let bytes = value.unwrap_or(&[0, 0]);
    SingleBattery {
        is_charging: IsBatteryCharging::from(bytes.first().copied().unwrap_or(0) == 1),
        level: BatteryLevel(bytes.get(1).copied().unwrap_or(0)),
    }
}

fn parse_firmware(value: Option<&[u8]>) -> Option<FirmwareVersion> {
    value.and_then(|bytes| {
        FirmwareVersion::take::<VerboseError<&[u8]>>(bytes)
            .ok()
            .map(|(_, version)| version)
    })
}

impl FromPacketBody for D1204StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("d1204 state update packet", |input| {
            let (remaining, fields) = take_tlv_fields(input)?;
            let fields: HashMap<u8, &[u8]> = fields.into_iter().collect();
            let get = |tag: u8| fields.get(&tag).copied();

            let host_device = get(tag::TWS_HOST_DEVICE)
                .and_then(|b| b.first())
                .copied()
                .unwrap_or(0);
            let tws_status = TwsStatus {
                host_device: HostDevice::from_repr(host_device).unwrap_or_default(),
                is_connected: get(tag::TWS_CONNECTED)
                    .and_then(|b| b.first())
                    .map(|&v| v == 1)
                    .unwrap_or(true),
            };

            let dual_battery = DualBattery {
                left: parse_single_battery(get(tag::BATTERY_LEFT)),
                right: parse_single_battery(get(tag::BATTERY_RIGHT)),
            };

            // Case battery: [flags/charging, level_percent]. Only the level
            // (second byte) is currently understood.
            let case_battery = CaseBatteryLevel(BatteryLevel(
                get(tag::CASE_BATTERY)
                    .and_then(|b| b.get(1))
                    .copied()
                    .unwrap_or(0),
            ));

            let equalizer_preset = get(tag::EQUALIZER_PRESET)
                .filter(|b| b.len() >= 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .unwrap_or(0);
            let equalizer =
                D1204Equalizer::new(equalizer_preset, get(tag::EQUALIZER_CURVE).unwrap_or(&[]));

            let button = |t: u8| D1204ButtonAction::from_bytes(get(t).unwrap_or(&[]));
            let buttons = D1204Buttons {
                left_single: button(tag::BUTTON_LEFT_SINGLE),
                right_single: button(tag::BUTTON_RIGHT_SINGLE),
                left_double: button(tag::BUTTON_LEFT_DOUBLE),
                right_double: button(tag::BUTTON_RIGHT_DOUBLE),
                left_triple: button(tag::BUTTON_LEFT_TRIPLE),
                right_triple: button(tag::BUTTON_RIGHT_TRIPLE),
                left_long: button(tag::BUTTON_LEFT_LONG),
                right_long: button(tag::BUTTON_RIGHT_LONG),
            };

            let dolby_audio = DolbyAudio::from_bytes(get(tag::DOLBY_AUDIO).unwrap_or(&[]));
            let easy_chat = EasyChat::from_bytes(get(tag::SMART_FEATURES).unwrap_or(&[]));

            let dual_firmware_version =
                match (parse_firmware(get(tag::FIRMWARE_LEFT)), parse_firmware(get(tag::FIRMWARE_RIGHT))) {
                    (Some(left), Some(right)) => DualFirmwareVersion::Both { left, right },
                    (Some(left), None) => DualFirmwareVersion::LeftOnly(left),
                    (None, Some(right)) => DualFirmwareVersion::RightOnly(right),
                    (None, None) => DualFirmwareVersion::default(),
                };

            let serial_number = get(tag::SERIAL_NUMBER)
                .and_then(|b| b.get(..16))
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(SerialNumber::from)
                .unwrap_or_default();

            let sound_mode = get(tag::SOUND_MODE)
                .and_then(|b| b.first())
                .and_then(|&v| SoundMode::from_repr(v))
                .unwrap_or_default();

            let manual_noise_canceling = ManualNoiseCanceling(
                get(tag::MANUAL_NOISE_CANCELING)
                    .and_then(|b| b.first())
                    .copied()
                    .unwrap_or(0),
            );

            let limit_high_volume = get(tag::LIMIT_HIGH_VOLUME)
                .and_then(|b| {
                    LimitHighVolume::take::<VerboseError<&[u8]>>(b)
                        .ok()
                        .map(|(_, value)| value)
                })
                .unwrap_or_default();

            Ok((
                remaining,
                Self {
                    tws_status,
                    dual_battery,
                    case_battery,
                    dual_firmware_version,
                    serial_number,
                    equalizer,
                    buttons,
                    dolby_audio,
                    easy_chat,
                    sound_mode,
                    manual_noise_canceling,
                    limit_high_volume,
                },
            ))
        })
        .parse_complete(input)
    }
}

fn tlv(tag: u8, value: &[u8]) -> impl Iterator<Item = u8> {
    [tag, value.len() as u8]
        .into_iter()
        .chain(value.iter().copied())
}

impl ToPacket for D1204StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> packet::Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        let firmware_bytes = |version: Option<FirmwareVersion>| {
            version.map(|v| v.bytes()).unwrap_or([0; 5])
        };
        tlv(tag::TWS_HOST_DEVICE, &[self.tws_status.host_device as u8])
            .chain(tlv(tag::TWS_CONNECTED, &[self.tws_status.is_connected as u8]))
            .chain(tlv(
                tag::BATTERY_LEFT,
                &[
                    self.dual_battery.left.is_charging as u8,
                    self.dual_battery.left.level.0,
                ],
            ))
            .chain(tlv(
                tag::BATTERY_RIGHT,
                &[
                    self.dual_battery.right.is_charging as u8,
                    self.dual_battery.right.level.0,
                ],
            ))
            .chain(tlv(tag::CASE_BATTERY, &[0, self.case_battery.0.0]))
            .chain(tlv(
                tag::FIRMWARE_LEFT,
                &firmware_bytes(self.dual_firmware_version.left()),
            ))
            .chain(tlv(
                tag::FIRMWARE_RIGHT,
                &firmware_bytes(self.dual_firmware_version.right()),
            ))
            .chain(tlv(
                tag::SERIAL_NUMBER,
                self.serial_number.as_str().as_bytes(),
            ))
            .chain(tlv(tag::EQUALIZER_PRESET, &self.equalizer.preset_id_bytes()))
            .chain(tlv(tag::EQUALIZER_CURVE, &self.equalizer.curve_bytes()))
            .chain(tlv(tag::BUTTON_LEFT_SINGLE, &self.buttons.left_single.bytes()))
            .chain(tlv(
                tag::BUTTON_RIGHT_SINGLE,
                &self.buttons.right_single.bytes(),
            ))
            .chain(tlv(tag::BUTTON_LEFT_DOUBLE, &self.buttons.left_double.bytes()))
            .chain(tlv(
                tag::BUTTON_RIGHT_DOUBLE,
                &self.buttons.right_double.bytes(),
            ))
            .chain(tlv(tag::BUTTON_LEFT_TRIPLE, &self.buttons.left_triple.bytes()))
            .chain(tlv(
                tag::BUTTON_RIGHT_TRIPLE,
                &self.buttons.right_triple.bytes(),
            ))
            .chain(tlv(tag::BUTTON_LEFT_LONG, &self.buttons.left_long.bytes()))
            .chain(tlv(tag::BUTTON_RIGHT_LONG, &self.buttons.right_long.bytes()))
            .chain(tlv(tag::DOLBY_AUDIO, &self.dolby_audio.bytes()))
            .chain(tlv(
                tag::SMART_FEATURES,
                &[
                    8,
                    0x01 | if self.easy_chat.0 { 0x40 } else { 0 },
                    if self.dolby_audio == DolbyAudio::Off { 0 } else { 2 },
                ],
            ))
            .chain(tlv(tag::SOUND_MODE, &[self.sound_mode as u8, 0, 0]))
            .chain(tlv(
                tag::MANUAL_NOISE_CANCELING,
                &[self.manual_noise_canceling.0, 1],
            ))
            .chain(tlv(tag::LIMIT_HIGH_VOLUME, &self.limit_high_volume.bytes()))
            .collect()
    }
}

struct StateUpdatePacketHandler;

#[async_trait]
impl PacketHandler<D1204State> for StateUpdatePacketHandler {
    async fn handle_packet(
        &self,
        state: &watch::Sender<D1204State>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        let packet: D1204StateUpdatePacket = packet.try_to_packet()?;
        state.send_modify(|state| *state = packet.into());
        Ok(())
    }
}

impl ModuleCollection<D1204State> {
    pub fn add_state_update(&mut self) {
        self.packet_handlers.set_handler(
            packet::inbound::STATE_COMMAND,
            Box::new(StateUpdatePacketHandler),
        );
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use crate::devices::soundcore::{
        common::{
            packet::inbound::FromPacketBody,
            structures::{DualFirmwareVersion, FirmwareVersion},
        },
        d1204::structures::{DolbyAudio, SoundMode},
    };

    use super::D1204StateUpdatePacket;

    /// Real Liberty 5 Pro Max (D1204) state update packet body captured from hardware.
    const REAL_PACKET_BODY: &[u8] = &[
        1, 1, 1, 2, 1, 1, 3, 2, 0, 99, 4, 2, 0, 99, 5, 5, 48, 51, 46, 52, 48, 6, 5, 48, 51, 46,
        52, 48, 7, 17, 49, 50, 48, 52, 55, 67, 69, 57, 49, 51, 56, 66, 53, 52, 68, 67, 0, 8, 2,
        128, 77, 9, 5, 48, 49, 46, 51, 56, 10, 1, 49, 11, 2, 2, 0, 12, 32, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 2, 15, 15,
        15, 2, 3, 6, 17, 2, 15, 15, 19, 2, 4, 4, 21, 2, 0, 0, 23, 2, 1, 1, 14, 2, 15, 15, 16, 2,
        6, 6, 18, 2, 15, 15, 20, 2, 4, 4, 22, 2, 0, 0, 24, 2, 1, 1, 25, 2, 1, 2, 42, 1, 1, 26, 1,
        255, 27, 3, 8, 65, 2, 28, 1, 1, 35, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 3, 2, 0, 0, 37, 2, 5, 1, 38, 2, 0,
        0, 39, 3, 0, 90, 0, 41, 6, 0, 127, 29, 19, 86, 169, 44, 3, 1, 1, 1, 46, 1, 99, 48, 1, 2,
        49, 1, 0, 50, 2, 0, 0, 51, 6, 0, 0, 0, 0, 0, 0, 52, 1, 0, 53, 1, 0, 54, 2, 1, 1, 68, 1, 0,
    ];

    #[test]
    fn it_parses_a_real_packet() {
        let (remaining, packet) =
            D1204StateUpdatePacket::take::<VerboseError<&[u8]>>(REAL_PACKET_BODY).unwrap();
        assert!(remaining.is_empty(), "entire packet should be consumed");

        assert_eq!(packet.serial_number.as_str(), "12047CE9138B54DC");
        assert_eq!(
            packet.dual_firmware_version,
            DualFirmwareVersion::Both {
                left: FirmwareVersion::new(3, 40),
                right: FirmwareVersion::new(3, 40),
            }
        );
        assert_eq!(packet.dual_battery.left.level.0, 99);
        assert_eq!(packet.dual_battery.right.level.0, 99);
        assert_eq!(packet.case_battery.0.0, 77);
        // tag 15 = [3, 6] -> NextSong (connected) ; tag 19 = [4, 4] -> AmbientSoundMode
        assert_eq!(packet.buttons.left_double.name(), Some("NextSong"));
        assert_eq!(packet.buttons.right_double.name(), Some("PlayPause"));
        assert_eq!(packet.buttons.left_long.name(), Some("AmbientSoundMode"));
        // tag 13 = [15, 15] -> disabled gesture.
        assert_eq!(packet.buttons.left_single.name(), None);
        // tag 44 = [1, 1, 1] -> Dolby Audio, Fixed mode.
        assert_eq!(packet.dolby_audio, DolbyAudio::Fixed);
        // tag 27 = [8, 65, 2] -> EasyChat on (bit 0x40).
        assert!(packet.easy_chat.0);
        // tag 11 = [2, 0] -> preset id 2 (not custom); tag 12 zeroed -> flat curve.
        assert_eq!(packet.equalizer.preset_id, 2);
        assert!(!packet.equalizer.is_custom());
        assert_eq!(packet.equalizer.bands_db(), [0.0; 8]);
        assert_eq!(packet.sound_mode, SoundMode::Normal);
        assert_eq!(packet.manual_noise_canceling.0, 5);
        assert_eq!(packet.limit_high_volume.db_limit, 90);
        assert!(!packet.limit_high_volume.enabled);
    }

    #[test]
    fn it_round_trips_through_to_packet() {
        use crate::devices::soundcore::common::packet::outbound::ToPacket;

        let (_, packet) =
            D1204StateUpdatePacket::take::<VerboseError<&[u8]>>(REAL_PACKET_BODY).unwrap();
        let body = packet.body();
        let (_, reparsed) =
            D1204StateUpdatePacket::take::<VerboseError<&[u8]>>(&body).unwrap();
        assert_eq!(packet, reparsed);
    }
}
