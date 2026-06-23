use async_trait::async_trait;
use openscq30_lib_has::Has;

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        common::{
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        },
        d1204::structures::{D1204ButtonAction, D1204Buttons},
    },
};

/// Read-only view of the Liberty 5 Pro Max button (touch gesture) configuration.
///
/// Writing button actions requires the outbound "set" packet format, which has
/// not been captured yet, so the gestures are exposed as read-only information.
struct ButtonsSettingHandler;

impl ButtonsSettingHandler {
    fn action_for(buttons: &D1204Buttons, setting_id: &SettingId) -> Option<D1204ButtonAction> {
        Some(match setting_id {
            SettingId::LeftSinglePress => buttons.left_single,
            SettingId::RightSinglePress => buttons.right_single,
            SettingId::LeftDoublePress => buttons.left_double,
            SettingId::RightDoublePress => buttons.right_double,
            SettingId::LeftTriplePress => buttons.left_triple,
            SettingId::RightTriplePress => buttons.right_triple,
            SettingId::LeftLongPress => buttons.left_long,
            SettingId::RightLongPress => buttons.right_long,
            _ => return None,
        })
    }
}

#[async_trait]
impl<T> SettingHandler<T> for ButtonsSettingHandler
where
    T: Has<D1204Buttons> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        vec![
            SettingId::LeftSinglePress,
            SettingId::RightSinglePress,
            SettingId::LeftDoublePress,
            SettingId::RightDoublePress,
            SettingId::LeftTriplePress,
            SettingId::RightTriplePress,
            SettingId::LeftLongPress,
            SettingId::RightLongPress,
        ]
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let buttons: &D1204Buttons = state.get();
        let action = Self::action_for(buttons, setting_id)?;
        Some(Setting::Information {
            value: action.name().unwrap_or("Disabled").to_string(),
            translated_value: action.localized_name().unwrap_or_else(|| "Disabled".to_string()),
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
    T: Has<D1204Buttons> + Clone + Send + Sync + 'static,
{
    pub fn add_d1204_buttons_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::ButtonConfiguration, ButtonsSettingHandler);
    }
}
