use async_trait::async_trait;
use openscq30_lib_has::Has;

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        },
        d1204::structures::EasyChat,
    },
};

/// Read-only view of the Liberty 5 Pro Max EasyChat feature (auto transparency
/// while talking). Writing requires the outbound "set" packet, not captured yet.
struct EasyChatSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for EasyChatSettingHandler
where
    T: Has<EasyChat> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        vec![SettingId::EasyChat]
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        if *setting_id != SettingId::EasyChat {
            return None;
        }
        let easy_chat: &EasyChat = state.get();
        let label = if easy_chat.0 { "On" } else { "Off" };
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
    T: Has<EasyChat> + Clone + Send + Sync + 'static,
{
    pub fn add_d1204_easy_chat_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::General, EasyChatSettingHandler);
    }
}
