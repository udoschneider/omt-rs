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
