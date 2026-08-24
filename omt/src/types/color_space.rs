//! Color space definitions for video frames.

/// Color space for video frames.
///
/// Used to determine the color space for YUV<>RGB conversions internally.
/// If undefined, the codec will assume BT601 for heights < 720, BT709 for everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ColorSpace {
    /// Undefined color space (automatic selection).
    Undefined = omt_sys::OMTColorSpace_Undefined,
    /// BT.601 color space.
    Bt601 = omt_sys::OMTColorSpace_BT601,
    /// BT.709 color space.
    Bt709 = omt_sys::OMTColorSpace_BT709,
}

impl ColorSpace {
    /// Returns the conventional display name of this color space.
    pub fn name(&self) -> &'static str {
        match self {
            ColorSpace::Undefined => "undefined",
            ColorSpace::Bt601 => "BT.601",
            ColorSpace::Bt709 => "BT.709",
        }
    }

    /// Creates a `ColorSpace` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTColorSpace_Undefined => Some(Self::Undefined),
            omt_sys::OMTColorSpace_BT601 => Some(Self::Bt601),
            omt_sys::OMTColorSpace_BT709 => Some(Self::Bt709),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_name() {
        assert_eq!(ColorSpace::Bt601.to_string(), "BT.601");
        assert_eq!(ColorSpace::Bt709.to_string(), "BT.709");
        assert_eq!(ColorSpace::Undefined.to_string(), "undefined");
    }

    #[test]
    fn test_ffi_roundtrip() {
        for cs in [ColorSpace::Undefined, ColorSpace::Bt601, ColorSpace::Bt709] {
            assert_eq!(ColorSpace::from_ffi(cs.to_ffi()), Some(cs));
        }
    }
}
