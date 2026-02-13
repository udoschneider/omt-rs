//! Color space definitions for video encoding in Open Media Transport.
//!
//! Defines the color space standards used for YUV to RGB conversion and video processing.

use crate::ffi;

/// Video color space standard.
///
/// Defines the color space used for video encoding/decoding.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColorSpace {
    /// Undefined or unknown color space
    Undefined,
    /// ITU-R BT.601 (standard definition)
    BT601,
    /// ITU-R BT.709 (high definition)
    BT709,
}

impl From<ffi::OMTColorSpace> for ColorSpace {
    fn from(value: ffi::OMTColorSpace) -> Self {
        match value {
            ffi::OMTColorSpace::BT601 => ColorSpace::BT601,
            ffi::OMTColorSpace::BT709 => ColorSpace::BT709,
            _ => ColorSpace::Undefined,
        }
    }
}

impl From<ColorSpace> for ffi::OMTColorSpace {
    fn from(value: ColorSpace) -> Self {
        match value {
            ColorSpace::BT601 => ffi::OMTColorSpace::BT601,
            ColorSpace::BT709 => ffi::OMTColorSpace::BT709,
            ColorSpace::Undefined => ffi::OMTColorSpace::Undefined,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_bt601() {
        let cs = ColorSpace::from(ffi::OMTColorSpace::BT601);
        assert_eq!(cs, ColorSpace::BT601);
    }

    #[test]
    fn test_from_ffi_bt709() {
        let cs = ColorSpace::from(ffi::OMTColorSpace::BT709);
        assert_eq!(cs, ColorSpace::BT709);
    }

    #[test]
    fn test_from_ffi_undefined() {
        let cs = ColorSpace::from(ffi::OMTColorSpace::Undefined);
        assert_eq!(cs, ColorSpace::Undefined);
    }

    #[test]
    fn test_to_ffi_bt601() {
        let ffi_cs: ffi::OMTColorSpace = ColorSpace::BT601.into();
        assert_eq!(ffi_cs as i32, ffi::OMTColorSpace::BT601 as i32);
    }

    #[test]
    fn test_to_ffi_bt709() {
        let ffi_cs: ffi::OMTColorSpace = ColorSpace::BT709.into();
        assert_eq!(ffi_cs as i32, ffi::OMTColorSpace::BT709 as i32);
    }

    #[test]
    fn test_to_ffi_undefined() {
        let ffi_cs: ffi::OMTColorSpace = ColorSpace::Undefined.into();
        assert_eq!(ffi_cs as i32, ffi::OMTColorSpace::Undefined as i32);
    }

    #[test]
    fn test_clone() {
        let cs1 = ColorSpace::BT709;
        let cs2 = cs1.clone();
        assert_eq!(cs1, cs2);
    }

    #[test]
    fn test_copy() {
        let cs1 = ColorSpace::BT601;
        let cs2 = cs1;
        assert_eq!(cs1, ColorSpace::BT601);
        assert_eq!(cs2, ColorSpace::BT601);
    }

    #[test]
    fn test_eq() {
        assert_eq!(ColorSpace::BT601, ColorSpace::BT601);
        assert_eq!(ColorSpace::BT709, ColorSpace::BT709);
        assert_eq!(ColorSpace::Undefined, ColorSpace::Undefined);
        assert_ne!(ColorSpace::BT601, ColorSpace::BT709);
        assert_ne!(ColorSpace::BT601, ColorSpace::Undefined);
        assert_ne!(ColorSpace::BT709, ColorSpace::Undefined);
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", ColorSpace::BT601), "BT601");
        assert_eq!(format!("{:?}", ColorSpace::BT709), "BT709");
        assert_eq!(format!("{:?}", ColorSpace::Undefined), "Undefined");
    }
}
