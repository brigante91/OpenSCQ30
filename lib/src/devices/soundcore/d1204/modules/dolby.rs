use async_trait::async_trait;
use openscq30_lib_has::Has;

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        },
        d1204::structures::DolbyAudio,
    },
};

/// Read-only view of the Liberty 5 Pro Max Dolby Audio / spatial audio mode.
///
/// Writing requires the outbound "set" packet format, not captured yet.
struct DolbyAudioSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for DolbyAudioSettingHandler
where
    T: Has<DolbyAudio> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        vec![SettingId::DolbyAudio]
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        if *setting_id != SettingId::DolbyAudio {
            return None;
        }
        let mode: &DolbyAudio = state.get();
        let label = match mode {
            DolbyAudio::Off => "Off",
            DolbyAudio::Fixed => "Fixed",
            DolbyAudio::HeadTracking => "Head Tracking",
        };
        Some(Setting::Information {
            value: label.to_string(),
            translated_value: label.to_string(),
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

impl<T> ModuleCollection<T>
where
    T: Has<DolbyAudio> + Clone + Send + Sync + 'static,
{
    pub fn add_d1204_dolby_audio_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::General, DolbyAudioSettingHandler);
    }
}
