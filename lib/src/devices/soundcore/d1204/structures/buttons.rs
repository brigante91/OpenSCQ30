use crate::devices::soundcore::common::modules::button_configuration::COMMON_ACTIONS_WITH_GAME_MODE;

/// Action ID used by the device to mean "no action / disabled".
pub const ACTION_DISABLED: u8 = 0xF;

/// A single button gesture on the Liberty 5 Pro Max (D1204).
///
/// Each gesture tag in the state update packet carries two action IDs. Mirroring
/// the older Liberty 5 (A3957), the first byte is the action used while the
/// earbuds are connected as a TWS pair and the second is the action used when
/// only one earbud is in use. `0xF` means the gesture is disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct D1204ButtonAction {
    pub connected: u8,
    pub disconnected: u8,
}

impl Default for D1204ButtonAction {
    fn default() -> Self {
        Self {
            connected: ACTION_DISABLED,
            disconnected: ACTION_DISABLED,
        }
    }
}

impl D1204ButtonAction {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            connected: bytes.first().copied().unwrap_or(ACTION_DISABLED),
            disconnected: bytes.get(1).copied().unwrap_or(ACTION_DISABLED),
        }
    }

    pub fn bytes(&self) -> [u8; 2] {
        [self.connected, self.disconnected]
    }

    /// The action currently in effect (the earbuds are normally connected as a
    /// TWS pair, so this is the `connected` slot).
    pub fn current(&self) -> u8 {
        self.connected
    }

    /// English, stable name of the current action (e.g. `"PlayPause"`), or
    /// `None` when the gesture is disabled or the action ID is unknown.
    pub fn name(&self) -> Option<&'static str> {
        action_name(self.current())
    }

    /// Localized name of the current action, or `None` when disabled/unknown.
    pub fn localized_name(&self) -> Option<String> {
        let id = self.current();
        COMMON_ACTIONS_WITH_GAME_MODE
            .iter()
            .find(|action| action.id == id)
            .map(|action| (action.localized_name)())
    }
}

fn action_name(id: u8) -> Option<&'static str> {
    COMMON_ACTIONS_WITH_GAME_MODE
        .iter()
        .find(|action| action.id == id)
        .map(|action| action.name)
}

/// Button gesture configuration parsed from the D1204 state update packet.
///
/// Tag mapping (decoded from a real device, cross-checked against the A3957
/// action table):
/// - tag 13 / 14: single press, left / right
/// - tag 15 / 16: double press, left / right
/// - tag 17 / 18: triple press, left / right
/// - tag 19 / 20: long press, left / right
///
/// - tag 21 / 22: swipe up, left / right (default VolumeUp)
/// - tag 23 / 24: swipe down, left / right (default VolumeDown)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct D1204Buttons {
    pub left_single: D1204ButtonAction,
    pub right_single: D1204ButtonAction,
    pub left_double: D1204ButtonAction,
    pub right_double: D1204ButtonAction,
    pub left_triple: D1204ButtonAction,
    pub right_triple: D1204ButtonAction,
    pub left_long: D1204ButtonAction,
    pub right_long: D1204ButtonAction,
    pub left_swipe_up: D1204ButtonAction,
    pub right_swipe_up: D1204ButtonAction,
    pub left_swipe_down: D1204ButtonAction,
    pub right_swipe_down: D1204ButtonAction,
}
