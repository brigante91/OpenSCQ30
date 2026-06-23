use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    common::structures::{
        CaseBatteryLevel, DualBattery, DualFirmwareVersion, LimitHighVolume, SerialNumber,
        TwsStatus,
    },
    d1204::{
        packets::inbound::D1204StateUpdatePacket,
        structures::{
            D1204Buttons, D1204Equalizer, DolbyAudio, EasyChat, ManualNoiseCanceling, SoundMode,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct D1204State {
    tws_status: TwsStatus,
    dual_battery: DualBattery,
    case_battery: CaseBatteryLevel,
    dual_firmware_version: DualFirmwareVersion,
    serial_number: SerialNumber,
    equalizer: D1204Equalizer,
    buttons: D1204Buttons,
    dolby_audio: DolbyAudio,
    easy_chat: EasyChat,
    sound_mode: SoundMode,
    manual_noise_canceling: ManualNoiseCanceling,
    limit_high_volume: LimitHighVolume,
}

impl From<D1204StateUpdatePacket> for D1204State {
    fn from(packet: D1204StateUpdatePacket) -> Self {
        Self {
            tws_status: packet.tws_status,
            dual_battery: packet.dual_battery,
            case_battery: packet.case_battery,
            dual_firmware_version: packet.dual_firmware_version,
            serial_number: packet.serial_number,
            equalizer: packet.equalizer,
            buttons: packet.buttons,
            dolby_audio: packet.dolby_audio,
            easy_chat: packet.easy_chat,
            sound_mode: packet.sound_mode,
            manual_noise_canceling: packet.manual_noise_canceling,
            limit_high_volume: packet.limit_high_volume,
        }
    }
}
