use async_trait::async_trait;
use openscq30_lib_has::Has;
use strum::IntoEnumIterator;

use crate::{
    api::settings::{Setting, SettingId, Value},
    devices::soundcore::common::{
        settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        structures::DualBattery,
    },
    i18n::fl,
};

use super::BatterySetting;

pub struct BatterySettingHandler {
    max_level: u8,
}

impl BatterySettingHandler {
    pub fn new(max_level: u8) -> Self {
        Self { max_level }
    }
}

#[async_trait]
impl<T> SettingHandler<T> for BatterySettingHandler
where
    T: Has<DualBattery> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        BatterySetting::iter().map(Into::into).collect()
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let battery = state.get();
        let setting: BatterySetting = (*setting_id).try_into().ok()?;
        Some(match setting {
            BatterySetting::IsChargingLeft => Setting::Information {
                value: battery.left.is_charging.to_string(),
                translated_value: if battery.left.is_charging.into() {
                    fl!("charging")
                } else {
                    fl!("not-charging")
                },
            },
            BatterySetting::IsChargingRight => Setting::Information {
                value: battery.right.is_charging.to_string(),
                translated_value: if battery.right.is_charging.into() {
                    fl!("charging")
                } else {
                    fl!("not-charging")
                },
            },
            BatterySetting::BatteryLevelLeft => {
                let percentage = (battery.left.level.0 as f32 / self.max_level as f32 * 100.0).round() as u8;
                Setting::Information {
                    value: battery.left.level.0.to_string(),
                    translated_value: format!("{}/{} ({}%)", battery.left.level.0, self.max_level, percentage),
                }
            },
            BatterySetting::BatteryLevelRight => {
                let percentage = (battery.right.level.0 as f32 / self.max_level as f32 * 100.0).round() as u8;
                Setting::Information {
                    value: battery.right.level.0.to_string(),
                    translated_value: format!("{}/{} ({}%)", battery.right.level.0, self.max_level, percentage),
                }
            },
        })
    }

    async fn set(
        &self,
        _state: &mut T,
        _setting_id: &SettingId,
        _value: Value,
    ) -> SettingHandlerResult<()> {
        Err(SettingHandlerError::ReadOnly)
    }
}
