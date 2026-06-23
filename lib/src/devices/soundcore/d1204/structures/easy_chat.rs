/// Bit set in tag 27 byte 1 when EasyChat is enabled.
const EASY_CHAT_BIT: u8 = 0x40;

/// EasyChat ("automatically switch to transparency when you start talking") on
/// the Liberty 5 Pro Max.
///
/// Decoded from tag 27, a small feature bitfield with layout `[8, flags, dolby]`.
/// EasyChat is bit `0x40` of the flags byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EasyChat(pub bool);

impl EasyChat {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.get(1).copied().unwrap_or(0) & EASY_CHAT_BIT != 0)
    }
}
