use std::collections::HashMap;

use crate::devices::soundcore::{
    common::{
        self,
        device::fetch_state_from_state_update_packet,
        macros::soundcore_device,
        packet::outbound::{RequestState, ToPacket},
    },
    d1204::{packets::inbound::D1204StateUpdatePacket, state::D1204State},
};

mod modules;
mod packets;
mod state;
mod structures;

// Soundcore Liberty 5 Pro Max (D1204).
//
// Unlike the older "A" series devices, the D1204 encodes its state update packet
// using a Tag-Length-Value layout (see `common::packet::parsing::take_tlv_fields`
// and `packets::inbound::D1204StateUpdatePacket`).
//
// Only the tags that have been decoded so far are wired up. To add more settings
// (sound modes, equalizer, button configuration, etc.), decode the relevant tag
// using the differential capture method and enable the matching builder module.
soundcore_device!(
    D1204State,
    async |packet_io| {
        fetch_state_from_state_update_packet::<D1204State, D1204StateUpdatePacket>(packet_io).await
    },
    async |builder| {
        builder.module_collection().add_state_update();
        // Read-only sound mode (ANC / Transparency / Normal) + manual ANC level.
        builder.module_collection().add_d1204_sound_modes_info();
        // Read-only touch gesture (button) configuration.
        builder.module_collection().add_d1204_buttons_info();
        // Read-only Dolby Audio mode (Off / Fixed / Head Tracking).
        builder.module_collection().add_d1204_dolby_audio_info();
        // Read-only EasyChat (auto transparency while talking).
        builder.module_collection().add_d1204_easy_chat_info();
        // Read-only equalizer (preset id + custom curve, 8 bands in dB).
        builder.module_collection().add_d1204_equalizer_info();
        builder.serial_number_and_dual_firmware_version();
        builder.tws_status();
        // Battery is reported as a percentage (0-100) on this device.
        builder.dual_battery(100);
        // Charging case battery, also reported as a percentage (0-100).
        builder.module_collection().add_case_battery_level(
            common::modules::case_battery_level::CaseBatteryLevelConfiguration {
                max_level: 100,
                level_offset: 0,
            },
        );
        // TODO: limit_high_volume is decoded in the state packet, but the outbound
        // "set" packet format for the D1204 has not been verified yet. Enable once
        // confirmed:
        // builder.limit_high_volume();
    },
    {
        HashMap::from([(
            RequestState::COMMAND,
            D1204StateUpdatePacket::default().to_packet(),
        )])
    },
);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        DeviceModel,
        devices::soundcore::common::{
            device::{SoundcoreDeviceConfig, test_utils::TestSoundcoreDevice},
            packet,
        },
        settings::SettingId,
    };

    #[tokio::test(start_paused = true)]
    async fn it_parses_serial_and_firmware_from_real_packet() {
        let device = TestSoundcoreDevice::new(
            super::device_registry,
            DeviceModel::SoundcoreD1204,
            HashMap::from([(
                packet::Command([1, 1]),
                packet::Inbound::new(
                    packet::Command([1, 1]),
                    vec![
                        1, 1, 1, 2, 1, 1, 3, 2, 0, 99, 4, 2, 0, 99, 5, 5, 48, 51, 46, 52, 48, 6, 5,
                        48, 51, 46, 52, 48, 7, 17, 49, 50, 48, 52, 55, 67, 69, 57, 49, 51, 56, 66,
                        53, 52, 68, 67, 0, 8, 2, 128, 77, 9, 5, 48, 49, 46, 51, 56, 10, 1, 49, 11,
                        2, 2, 0, 12, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 2, 15, 15, 15, 2, 3, 6, 17, 2,
                        15, 15, 19, 2, 4, 4, 21, 2, 0, 0, 23, 2, 1, 1, 14, 2, 15, 15, 16, 2, 6, 6,
                        18, 2, 15, 15, 20, 2, 4, 4, 22, 2, 0, 0, 24, 2, 1, 1, 25, 2, 1, 2, 42, 1,
                        1, 26, 1, 255, 27, 3, 8, 65, 2, 28, 1, 1, 35, 92, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 3, 2, 0, 0, 37, 2, 5, 1, 38, 2,
                        0, 0, 39, 3, 0, 90, 0, 41, 6, 0, 127, 29, 19, 86, 169, 44, 3, 1, 1, 1, 46,
                        1, 99, 48, 1, 2, 49, 1, 0, 50, 2, 0, 0, 51, 6, 0, 0, 0, 0, 0, 0, 52, 1, 0,
                        53, 1, 0, 54, 2, 1, 1, 68, 1, 0,
                    ],
                ),
            )]),
            SoundcoreDeviceConfig::default(),
        )
        .await;

        device.assert_setting_values([
            (SettingId::SerialNumber, "12047CE9138B54DC".into()),
            (SettingId::FirmwareVersionLeft, "03.40".into()),
            (SettingId::FirmwareVersionRight, "03.40".into()),
            (SettingId::BatteryLevelLeft, "99/100".into()),
            (SettingId::BatteryLevelRight, "99/100".into()),
            (SettingId::CaseBatteryLevel, "77/100".into()),
            (SettingId::AmbientSoundMode, "Normal".into()),
            (SettingId::ManualNoiseCanceling, "5".into()),
            (SettingId::LeftDoublePress, "NextSong".into()),
            (SettingId::RightDoublePress, "PlayPause".into()),
            (SettingId::LeftLongPress, "AmbientSoundMode".into()),
            (SettingId::RightLongPress, "AmbientSoundMode".into()),
            (SettingId::SwipeUp, "VolumeUp".into()),
            (SettingId::SwipeDown, "VolumeDown".into()),
            (SettingId::DolbyAudio, "Fixed".into()),
            (SettingId::EasyChat, "On".into()),
            (SettingId::PresetEqualizerProfile, "Preset 2".into()),
            (
                SettingId::VolumeAdjustments,
                "0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0".into(),
            ),
        ]);
    }
}
