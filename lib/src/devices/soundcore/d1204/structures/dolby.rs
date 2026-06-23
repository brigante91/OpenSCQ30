use strum::FromRepr;

/// Dolby Audio / spatial audio mode on the Liberty 5 Pro Max.
///
/// Decoded from tag 44 of the state update packet, layout `[enabled, mode, 1]`.
/// The mode byte fully describes the state:
/// - `0` Off
/// - `1` Fixed (spatial audio, no head tracking)
/// - `2` Head Tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromRepr)]
#[repr(u8)]
pub enum DolbyAudio {
    #[default]
    Off = 0,
    Fixed = 1,
    HeadTracking = 2,
}

impl DolbyAudio {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        bytes
            .get(1)
            .and_then(|&v| Self::from_repr(v))
            .unwrap_or_default()
    }

    /// Tag 44 body: `[enabled, mode, 1]`.
    pub fn bytes(&self) -> [u8; 3] {
        [u8::from(*self != Self::Off), *self as u8, 1]
    }
}
