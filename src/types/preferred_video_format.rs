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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_uyvy() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::UYVY);
        assert_eq!(fmt, PreferredVideoFormat::UYVY);
    }

    #[test]
    fn test_from_ffi_uyvy_or_bgra() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::UYVYorBGRA);
        assert_eq!(fmt, PreferredVideoFormat::UYVYorBGRA);
    }

    #[test]
    fn test_from_ffi_bgra() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::BGRA);
        assert_eq!(fmt, PreferredVideoFormat::BGRA);
    }

    #[test]
    fn test_from_ffi_uyvy_or_uyva() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::UYVYorUYVA);
        assert_eq!(fmt, PreferredVideoFormat::UYVYorUYVA);
    }

    #[test]
    fn test_from_ffi_uyvy_or_uyva_or_p216_or_pa16() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::UYVYorUYVAorP216orPA16);
        assert_eq!(fmt, PreferredVideoFormat::UYVYorUYVAorP216orPA16);
    }

    #[test]
    fn test_from_ffi_p216() {
        let fmt = PreferredVideoFormat::from(ffi::OMTPreferredVideoFormat::P216);
        assert_eq!(fmt, PreferredVideoFormat::P216);
    }

    #[test]
    fn test_to_ffi_uyvy() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat = PreferredVideoFormat::UYVY.into();
        assert_eq!(ffi_fmt as i32, ffi::OMTPreferredVideoFormat::UYVY as i32);
    }

    #[test]
    fn test_to_ffi_uyvy_or_bgra() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat = PreferredVideoFormat::UYVYorBGRA.into();
        assert_eq!(
            ffi_fmt as i32,
            ffi::OMTPreferredVideoFormat::UYVYorBGRA as i32
        );
    }

    #[test]
    fn test_to_ffi_bgra() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat = PreferredVideoFormat::BGRA.into();
        assert_eq!(ffi_fmt as i32, ffi::OMTPreferredVideoFormat::BGRA as i32);
    }

    #[test]
    fn test_to_ffi_uyvy_or_uyva() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat = PreferredVideoFormat::UYVYorUYVA.into();
        assert_eq!(
            ffi_fmt as i32,
            ffi::OMTPreferredVideoFormat::UYVYorUYVA as i32
        );
    }

    #[test]
    fn test_to_ffi_uyvy_or_uyva_or_p216_or_pa16() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat =
            PreferredVideoFormat::UYVYorUYVAorP216orPA16.into();
        assert_eq!(
            ffi_fmt as i32,
            ffi::OMTPreferredVideoFormat::UYVYorUYVAorP216orPA16 as i32
        );
    }

    #[test]
    fn test_to_ffi_p216() {
        let ffi_fmt: ffi::OMTPreferredVideoFormat = PreferredVideoFormat::P216.into();
        assert_eq!(ffi_fmt as i32, ffi::OMTPreferredVideoFormat::P216 as i32);
    }

    #[test]
    fn test_clone() {
        let fmt1 = PreferredVideoFormat::BGRA;
        let fmt2 = fmt1.clone();
        assert_eq!(fmt1, fmt2);
    }

    #[test]
    fn test_copy() {
        let fmt1 = PreferredVideoFormat::UYVY;
        let fmt2 = fmt1;
        assert_eq!(fmt1, PreferredVideoFormat::UYVY);
        assert_eq!(fmt2, PreferredVideoFormat::UYVY);
    }

    #[test]
    fn test_eq() {
        assert_eq!(PreferredVideoFormat::UYVY, PreferredVideoFormat::UYVY);
        assert_eq!(PreferredVideoFormat::BGRA, PreferredVideoFormat::BGRA);
        assert_ne!(PreferredVideoFormat::UYVY, PreferredVideoFormat::BGRA);
        assert_ne!(
            PreferredVideoFormat::UYVYorBGRA,
            PreferredVideoFormat::UYVYorUYVA
        );
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", PreferredVideoFormat::UYVY), "UYVY");
        assert_eq!(
            format!("{:?}", PreferredVideoFormat::UYVYorBGRA),
            "UYVYorBGRA"
        );
        assert_eq!(format!("{:?}", PreferredVideoFormat::BGRA), "BGRA");
        assert_eq!(
            format!("{:?}", PreferredVideoFormat::UYVYorUYVA),
            "UYVYorUYVA"
        );
        assert_eq!(
            format!("{:?}", PreferredVideoFormat::UYVYorUYVAorP216orPA16),
            "UYVYorUYVAorP216orPA16"
        );
        assert_eq!(format!("{:?}", PreferredVideoFormat::P216), "P216");
    }
}
