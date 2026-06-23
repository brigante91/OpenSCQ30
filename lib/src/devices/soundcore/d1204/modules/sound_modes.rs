use async_trait::async_trait;
use openscq30_lib_has::Has;

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        },
        d1204::structures::{ManualNoiseCanceling, SoundMode},
    },
    i18n::fl,
};

/// Read-only view of the Liberty 5 Pro Max sound mode.
///
/// Writing sound modes requires knowing the outbound "set" packet format, which
/// has not been captured yet. Until then these are exposed as read-only
/// information. Once the outbound format is known, replace this with a settable
/// module (see `common::modules::sound_modes_v2`).
struct SoundModesSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for SoundModesSettingHandler
where
    T: Has<SoundMode> + Has<ManualNoiseCanceling> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        vec![SettingId::AmbientSoundMode, SettingId::ManualNoiseCanceling]
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        match setting_id {
            SettingId::AmbientSoundMode => {
                let mode: &SoundMode = state.get();
                let translated_value = match mode {
                    SoundMode::NoiseCancelingManual => {
                        format!("{} ({})", fl!("noise-canceling"), fl!("manual"))
                    }
                    SoundMode::Transparency => fl!("transparency"),
                    SoundMode::Normal => fl!("normal"),
                    SoundMode::NoiseCancelingAdaptive => {
                        format!("{} ({})", fl!("noise-canceling"), fl!("adaptive"))
                    }
                };
                Some(Setting::Information {
                    value: format!("{mode:?}"),
                    translated_value,
                })
            }
            SettingId::ManualNoiseCanceling => {
                let level: &ManualNoiseCanceling = state.get();
                Some(Setting::Information {
                    value: level.0.to_string(),
                    translated_value: level.0.to_string(),
                })
            }
            _ => None,
        }
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

impl<T> ModuleCollection<T>
where
    T: Has<SoundMode> + Has<ManualNoiseCanceling> + Clone + Send + Sync + 'static,
{
    pub fn add_d1204_sound_modes_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::SoundModes, SoundModesSettingHandler);
    }
}
