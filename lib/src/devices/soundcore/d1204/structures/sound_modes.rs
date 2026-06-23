use strum::FromRepr;

/// Current sound mode of the Liberty 5 Pro Max.
///
/// On this device the mode is a single flattened value (tag 36, byte 0) rather
/// than the separate ambient/noise-canceling fields used by older devices:
/// - `0` Noise Canceling, manual strength
/// - `1` Transparency
/// - `2` Normal
/// - `3` Noise Canceling, adaptive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromRepr)]
#[repr(u8)]
pub enum SoundMode {
    NoiseCancelingManual = 0,
    Transparency = 1,
    #[default]
    Normal = 2,
    NoiseCancelingAdaptive = 3,
}

/// Manual noise canceling strength level (1-5), from tag 37 byte 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ManualNoiseCanceling(pub u8);
