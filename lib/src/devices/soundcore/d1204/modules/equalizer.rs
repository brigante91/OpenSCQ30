use async_trait::async_trait;
use openscq30_lib_has::Has;

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        },
        d1204::structures::D1204Equalizer,
    },
};

/// Read-only view of the Liberty 5 Pro Max equalizer.
///
/// Exposes the selected preset (tag 11) and, when a custom curve is active, the
/// per-band gains in dB (tag 12). Writing the equalizer requires the outbound
/// "set" packet format, which has not been captured yet.
struct EqualizerSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for EqualizerSettingHandler
where
    T: Has<D1204Equalizer> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        vec![
            SettingId::PresetEqualizerProfile,
            SettingId::VolumeAdjustments,
        ]
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let equalizer: &D1204Equalizer = state.get();
        match setting_id {
            SettingId::PresetEqualizerProfile => {
                let label = if equalizer.is_custom() {
                    "Custom".to_string()
                } else {
                    format!("Preset {}", equalizer.preset_id)
                };
                Some(Setting::Information {
                    value: label.clone(),
                    translated_value: label,
                })
            }
            SettingId::VolumeAdjustments => {
                let bands = equalizer
                    .bands_db()
                    .iter()
                    .map(|db| format!("{db:.1}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(Setting::Information {
                    value: bands.clone(),
                    translated_value: bands,
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
    T: Has<D1204Equalizer> + Clone + Send + Sync + 'static,
{
    pub fn add_d1204_equalizer_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::Equalizer, EqualizerSettingHandler);
    }
}
