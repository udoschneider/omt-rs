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
