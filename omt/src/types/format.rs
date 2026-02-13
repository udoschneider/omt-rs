//! Preferred video format definitions for receiving.

/// Preferred video format for receiving.
///
/// - `Uyvy` is always the fastest, if no alpha channel is required.
/// - `UyvyOrBgra` will provide BGRA only when alpha channel is present.
/// - `Bgra` will always convert back to BGRA.
/// - `UyvyOrUyva` will provide UYVA only when alpha channel is present.
/// - `UyvyOrUyvaOrP216OrPa16` will provide P216 if sender encoded with high bit depth,
///   or PA16 if sender encoded with high bit depth and alpha. Otherwise same as `UyvyOrUyva`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PreferredVideoFormat {
    /// Always receive UYVY.
    Uyvy = omt_sys::OMTPreferredVideoFormat_UYVY,
    /// Receive UYVY or BGRA (when alpha present).
    UyvyOrBgra = omt_sys::OMTPreferredVideoFormat_UYVYorBGRA,
    /// Always receive BGRA.
    Bgra = omt_sys::OMTPreferredVideoFormat_BGRA,
    /// Receive UYVY or UYVA (when alpha present).
    UyvyOrUyva = omt_sys::OMTPreferredVideoFormat_UYVYorUYVA,
    /// Receive UYVY, UYVA, P216, or PA16 based on sender format.
    UyvyOrUyvaOrP216OrPa16 = omt_sys::OMTPreferredVideoFormat_UYVYorUYVAorP216orPA16,
    /// Always receive P216.
    P216 = omt_sys::OMTPreferredVideoFormat_P216,
}

impl PreferredVideoFormat {
    /// Creates from FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTPreferredVideoFormat_UYVY => Some(Self::Uyvy),
            omt_sys::OMTPreferredVideoFormat_UYVYorBGRA => Some(Self::UyvyOrBgra),
            omt_sys::OMTPreferredVideoFormat_BGRA => Some(Self::Bgra),
            omt_sys::OMTPreferredVideoFormat_UYVYorUYVA => Some(Self::UyvyOrUyva),
            omt_sys::OMTPreferredVideoFormat_UYVYorUYVAorP216orPA16 => {
                Some(Self::UyvyOrUyvaOrP216OrPa16)
            }
            omt_sys::OMTPreferredVideoFormat_P216 => Some(Self::P216),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }
}
