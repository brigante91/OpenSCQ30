/// Preset ID used by the device to mean "custom" (drawn) equalizer.
pub const CUSTOM_PRESET_ID: u16 = 0xFEFE;

/// Number of equalizer bands on the Liberty 5 Pro Max.
pub const EQUALIZER_BANDS: usize = 8;

/// Equalizer state of the Liberty 5 Pro Max.
///
/// - `preset_id` comes from tag 11 (u16 little-endian). `0xFEFE` means a custom
///   curve is selected; other values select a built-in preset.
/// - `curve` is tag 12: [`EQUALIZER_BANDS`] consecutive little-endian `f32`
///   values, each a gain in dB. It is only populated when the custom preset is
///   active; built-in presets leave it zeroed.
///
/// The raw bytes are stored (rather than `f32`s) so the struct can derive `Eq`
/// and `Hash` like the rest of the device state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct D1204Equalizer {
    pub preset_id: u16,
    curve: [u8; EQUALIZER_BANDS * 4],
}

impl Default for D1204Equalizer {
    fn default() -> Self {
        Self {
            preset_id: 0,
            curve: [0; EQUALIZER_BANDS * 4],
        }
    }
}

impl D1204Equalizer {
    pub fn new(preset_id: u16, curve_bytes: &[u8]) -> Self {
        let mut curve = [0u8; EQUALIZER_BANDS * 4];
        let len = curve_bytes.len().min(curve.len());
        curve[..len].copy_from_slice(&curve_bytes[..len]);
        Self { preset_id, curve }
    }

    pub fn is_custom(&self) -> bool {
        self.preset_id == CUSTOM_PRESET_ID
    }

    /// Per-band gains in dB, decoded from the little-endian `f32` curve.
    pub fn bands_db(&self) -> [f32; EQUALIZER_BANDS] {
        let mut bands = [0.0f32; EQUALIZER_BANDS];
        for (band, chunk) in bands.iter_mut().zip(self.curve.chunks_exact(4)) {
            *band = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        bands
    }

    pub fn preset_id_bytes(&self) -> [u8; 2] {
        self.preset_id.to_le_bytes()
    }

    pub fn curve_bytes(&self) -> [u8; EQUALIZER_BANDS * 4] {
        self.curve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes_le_f32_bands() {
        // Real tag 12 capture: custom EQ with the first band set to maximum.
        let curve = [
            0, 0, 192, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let eq = D1204Equalizer::new(CUSTOM_PRESET_ID, &curve);
        assert!(eq.is_custom());
        assert_eq!(eq.bands_db(), [6.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }
}
