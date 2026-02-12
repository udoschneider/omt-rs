//! Preferred video format definitions for receiver negotiation in Open Media Transport.
//!
//! Defines the pixel format preferences that receivers can specify when connecting to senders.

use crate::ffi;

/// Preferred pixel format for receiver negotiation.
///
/// Specifies the preferred uncompressed video format of decoded frames.
///
/// # Performance Notes
///
/// UYVY is always the fastest if no alpha channel is required.
///
/// # Variants
///
/// * `UYVY` - Always receive UYVY format
/// * `UYVYorBGRA` - Provides BGRA only when alpha channel is present, otherwise UYVY
/// * `BGRA` - Always convert back to BGRA
/// * `UYVYorUYVA` - Provides UYVA only when alpha channel is present, otherwise UYVY
/// * `UYVYorUYVAorP216orPA16` - Provides P216 if sender encoded with high bit depth, or PA16
///   if sender encoded with high bit depth and alpha. Otherwise same as UYVYorUYVA.
/// * `P216` - Always receive P216 format
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PreferredVideoFormat {
    /// UYVY format only
    UYVY,
    /// UYVY or BGRA (BGRA when alpha present)
    UYVYorBGRA,
    /// BGRA format only
    BGRA,
    /// UYVY or UYVA (UYVA when alpha present)
    UYVYorUYVA,
    /// UYVY/UYVA/P216/PA16 based on bit depth and alpha
    UYVYorUYVAorP216orPA16,
    /// P216 format only
    P216,
}

impl From<ffi::OMTPreferredVideoFormat> for PreferredVideoFormat {
    fn from(value: ffi::OMTPreferredVideoFormat) -> Self {
        match value {
            ffi::OMTPreferredVideoFormat::UYVYorBGRA => PreferredVideoFormat::UYVYorBGRA,
            ffi::OMTPreferredVideoFormat::BGRA => PreferredVideoFormat::BGRA,
            ffi::OMTPreferredVideoFormat::UYVYorUYVA => PreferredVideoFormat::UYVYorUYVA,
            ffi::OMTPreferredVideoFormat::UYVYorUYVAorP216orPA16 => {
                PreferredVideoFormat::UYVYorUYVAorP216orPA16
            }
            ffi::OMTPreferredVideoFormat::P216 => PreferredVideoFormat::P216,
            _ => PreferredVideoFormat::UYVY,
        }
    }
}

impl From<PreferredVideoFormat> for ffi::OMTPreferredVideoFormat {
    fn from(value: PreferredVideoFormat) -> Self {
        match value {
            PreferredVideoFormat::UYVYorBGRA => ffi::OMTPreferredVideoFormat::UYVYorBGRA,
            PreferredVideoFormat::BGRA => ffi::OMTPreferredVideoFormat::BGRA,
            PreferredVideoFormat::UYVYorUYVA => ffi::OMTPreferredVideoFormat::UYVYorUYVA,
            PreferredVideoFormat::UYVYorUYVAorP216orPA16 => {
                ffi::OMTPreferredVideoFormat::UYVYorUYVAorP216orPA16
            }
            PreferredVideoFormat::P216 => ffi::OMTPreferredVideoFormat::P216,
            PreferredVideoFormat::UYVY => ffi::OMTPreferredVideoFormat::UYVY,
        }
    }
}
