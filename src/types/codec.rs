//! Codec definitions for Open Media Transport (OMT).
//!
//! This module defines the `Codec` enum representing supported pixel formats
//! and codec identifiers. Each codec corresponds to a specific media format
//! for video, audio, or metadata transmission.
//!
//! For detailed protocol specifications, refer to
//! [`libomt.h`](https://github.com/openmediatransport/libomt/blob/main/libomt.h).
//!
//! # Codec Overview
//!
//! ## Video Codecs
//!
//! ### Compressed
//! - **VMX1**: Fast video compression. Supports compressed frames for recording/processing
//!   when using `IncludeCompressed` or `CompressedOnly` receive flags.
//!
//! ### Uncompressed (8-bit)
//! - **UYVY**: YUV 4:2:2, 16 bpp. Fastest format without alpha. Chroma co-sited with even luma.
//! - **YUY2**: YUV 4:2:2, 16 bpp. YUYV pixel order variant.
//! - **UYVA**: UYVY + alpha plane. Treated as UYVY when alpha flags not set.
//! - **BGRA**: RGBA, 32 bpp (ARGB32 on Win32). Treated as BGRX when alpha flags not set.
//! - **NV12**: Planar YUV 4:2:0. Y plane + interleaved UV. Common in hardware acceleration.
//! - **YV12**: Planar YUV 4:2:0. Y plane + separate U/V planes.
//!
//! ### High Bit Depth (10-bit+)
//! - **P216**: Planar YUV 4:2:2, 16-bit. Auto-sets `VideoFlags::HIGH_BIT_DEPTH`.
//! - **PA16**: P216 + 16-bit alpha plane. Auto-sets `VideoFlags::HIGH_BIT_DEPTH`.
//!
//! ## Audio Codec
//! - **FPA1**: Floating-point Planar Audio, 32-bit. Only supported audio format.
//!
//! # Examples
//!
//! ```rust
//! use omt::Codec;
//!
//! let video_codec = Codec::BGRA;
//! let audio_codec = Codec::FPA1;
//!
//! // Get FOURCC code (little-endian 32-bit integer)
//! assert_eq!(Codec::UYVY.fourcc(), 0x59565955); // 'UYVY'
//! ```

use crate::ffi;

/// Supported pixel formats and codec identifiers.
///
/// Maps directly to the `OMTCodec` enum in `libomt.h`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Codec {
    /// Fast video compression codec (compressed).
    VMX1,

    /// Floating-point Planar Audio, 32-bit (audio).
    FPA1,

    /// YUV 4:2:2, 16 bpp. Chroma co-sited with even luma samples.
    UYVY,

    /// YUV 4:2:2, 16 bpp with YUYV pixel order.
    YUY2,

    /// RGBA, 32 bpp (ARGB32 on Win32). Treated as BGRX without alpha flags.
    BGRA,

    /// Planar YUV 4:2:0. Y plane + interleaved UV.
    NV12,

    /// Planar YUV 4:2:0. Y plane + separate U/V planes.
    YV12,

    /// UYVY + alpha plane. Treated as UYVY without alpha flags.
    UYVA,

    /// Planar YUV 4:2:2, 16-bit precision. Auto-sets `HIGH_BIT_DEPTH`.
    P216,

    /// P216 + 16-bit alpha plane. Auto-sets `HIGH_BIT_DEPTH`.
    PA16,

    /// Unknown codec with raw FOURCC value (forward compatibility).
    Unknown(i32),
}

impl From<ffi::OMTCodec> for Codec {
    fn from(value: ffi::OMTCodec) -> Self {
        let raw = value as i32;
        match raw {
            x if x == ffi::OMTCodec::VMX1 as i32 => Codec::VMX1,
            x if x == ffi::OMTCodec::FPA1 as i32 => Codec::FPA1,
            x if x == ffi::OMTCodec::UYVY as i32 => Codec::UYVY,
            x if x == ffi::OMTCodec::YUY2 as i32 => Codec::YUY2,
            x if x == ffi::OMTCodec::BGRA as i32 => Codec::BGRA,
            x if x == ffi::OMTCodec::NV12 as i32 => Codec::NV12,
            x if x == ffi::OMTCodec::YV12 as i32 => Codec::YV12,
            x if x == ffi::OMTCodec::UYVA as i32 => Codec::UYVA,
            x if x == ffi::OMTCodec::P216 as i32 => Codec::P216,
            x if x == ffi::OMTCodec::PA16 as i32 => Codec::PA16,
            _ => Codec::Unknown(raw),
        }
    }
}

impl Codec {
    /// Returns the FOURCC code as a little-endian `u32`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::Codec;
    ///
    /// assert_eq!(Codec::BGRA.fourcc(), 0x41524742); // 'BGRA'
    /// assert_eq!(Codec::UYVY.fourcc(), 0x59565955); // 'UYVY'
    /// ```
    pub fn fourcc(self) -> u32 {
        match self {
            Codec::VMX1 => ffi::OMTCodec::VMX1 as u32,
            Codec::FPA1 => ffi::OMTCodec::FPA1 as u32,
            Codec::UYVY => ffi::OMTCodec::UYVY as u32,
            Codec::YUY2 => ffi::OMTCodec::YUY2 as u32,
            Codec::BGRA => ffi::OMTCodec::BGRA as u32,
            Codec::NV12 => ffi::OMTCodec::NV12 as u32,
            Codec::YV12 => ffi::OMTCodec::YV12 as u32,
            Codec::UYVA => ffi::OMTCodec::UYVA as u32,
            Codec::P216 => ffi::OMTCodec::P216 as u32,
            Codec::PA16 => ffi::OMTCodec::PA16 as u32,
            Codec::Unknown(v) => v as u32,
        }
    }

    /// Converts to FFI `OMTCodec` representation.
    pub(crate) fn to_ffi(self) -> ffi::OMTCodec {
        match self {
            Codec::VMX1 => ffi::OMTCodec::VMX1,
            Codec::FPA1 => ffi::OMTCodec::FPA1,
            Codec::UYVY => ffi::OMTCodec::UYVY,
            Codec::YUY2 => ffi::OMTCodec::YUY2,
            Codec::BGRA => ffi::OMTCodec::BGRA,
            Codec::NV12 => ffi::OMTCodec::NV12,
            Codec::YV12 => ffi::OMTCodec::YV12,
            Codec::UYVA => ffi::OMTCodec::UYVA,
            Codec::P216 => ffi::OMTCodec::P216,
            Codec::PA16 => ffi::OMTCodec::PA16,
            Codec::Unknown(_) => ffi::OMTCodec::VMX1,
        }
    }

    /// Returns a human-readable string representation of the codec's FOURCC.
    ///
    /// Converts the codec's FOURCC value into a 4-character string.
    /// Non-printable characters are replaced with '.'.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::Codec;
    ///
    /// assert_eq!(Codec::BGRA.fourcc_string(), "BGRA");
    /// assert_eq!(Codec::UYVY.fourcc_string(), "UYVY");
    /// ```
    pub fn fourcc_string(self) -> String {
        fourcc_to_string(self.fourcc())
    }
}

/// Converts a FOURCC code to a human-readable string.
///
/// Takes a 32-bit FOURCC value and converts it to a 4-character string.
/// Non-printable characters (outside ASCII 32-126) are replaced with '.'.
fn fourcc_to_string(fourcc: u32) -> String {
    let bytes = [
        (fourcc & 0xff) as u8,
        ((fourcc >> 8) & 0xff) as u8,
        ((fourcc >> 16) & 0xff) as u8,
        ((fourcc >> 24) & 0xff) as u8,
    ];
    bytes
        .iter()
        .map(|&b| {
            if (32..=126).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi;

    #[test]
    fn test_codec_from_ffi() {
        assert_eq!(Codec::from(ffi::OMTCodec::VMX1), Codec::VMX1);
        assert_eq!(Codec::from(ffi::OMTCodec::FPA1), Codec::FPA1);
        assert_eq!(Codec::from(ffi::OMTCodec::UYVY), Codec::UYVY);
        assert_eq!(Codec::from(ffi::OMTCodec::YUY2), Codec::YUY2);
        assert_eq!(Codec::from(ffi::OMTCodec::BGRA), Codec::BGRA);
        assert_eq!(Codec::from(ffi::OMTCodec::NV12), Codec::NV12);
        assert_eq!(Codec::from(ffi::OMTCodec::YV12), Codec::YV12);
        assert_eq!(Codec::from(ffi::OMTCodec::UYVA), Codec::UYVA);
        assert_eq!(Codec::from(ffi::OMTCodec::P216), Codec::P216);
        assert_eq!(Codec::from(ffi::OMTCodec::PA16), Codec::PA16);

        assert_eq!(Codec::Unknown(0x12345678), Codec::Unknown(0x12345678));
    }

    #[test]
    fn test_codec_fourcc() {
        assert_eq!(Codec::VMX1.fourcc(), ffi::OMTCodec::VMX1 as u32);
        assert_eq!(Codec::FPA1.fourcc(), ffi::OMTCodec::FPA1 as u32);
        assert_eq!(Codec::UYVY.fourcc(), ffi::OMTCodec::UYVY as u32);
        assert_eq!(Codec::YUY2.fourcc(), ffi::OMTCodec::YUY2 as u32);
        assert_eq!(Codec::BGRA.fourcc(), ffi::OMTCodec::BGRA as u32);
        assert_eq!(Codec::NV12.fourcc(), ffi::OMTCodec::NV12 as u32);
        assert_eq!(Codec::YV12.fourcc(), ffi::OMTCodec::YV12 as u32);
        assert_eq!(Codec::UYVA.fourcc(), ffi::OMTCodec::UYVA as u32);
        assert_eq!(Codec::P216.fourcc(), ffi::OMTCodec::P216 as u32);
        assert_eq!(Codec::PA16.fourcc(), ffi::OMTCodec::PA16 as u32);

        let unknown_value = 0x12345678;
        assert_eq!(Codec::Unknown(unknown_value).fourcc(), unknown_value as u32);
    }

    #[test]
    fn test_codec_equality() {
        assert_eq!(Codec::BGRA, Codec::BGRA);
        assert_ne!(Codec::BGRA, Codec::UYVY);
        assert_ne!(Codec::VMX1, Codec::FPA1);
    }

    #[test]
    fn test_codec_debug() {
        assert_eq!(format!("{:?}", Codec::BGRA), "BGRA");
        assert_eq!(format!("{:?}", Codec::UYVY), "UYVY");
        assert_eq!(format!("{:?}", Codec::Unknown(123)), "Unknown(123)");
    }

    #[test]
    fn test_codec_clone() {
        let codec1 = Codec::BGRA;
        let codec2 = codec1.clone();
        assert_eq!(codec1, codec2);
    }

    #[test]
    fn test_codec_copy() {
        let codec1 = Codec::UYVY;
        let codec2 = codec1;
        assert_eq!(codec1, Codec::UYVY);
        assert_eq!(codec2, Codec::UYVY);
    }

    #[test]
    fn test_fourcc_string() {
        assert_eq!(Codec::BGRA.fourcc_string(), "BGRA");
        assert_eq!(Codec::UYVY.fourcc_string(), "UYVY");
        assert_eq!(Codec::VMX1.fourcc_string(), "VMX1");
        assert_eq!(Codec::FPA1.fourcc_string(), "FPA1");
    }

    #[test]
    fn test_fourcc_to_string() {
        use super::fourcc_to_string;

        // Test with printable ASCII characters
        assert_eq!(fourcc_to_string(0x41524742), "BGRA");
        assert_eq!(fourcc_to_string(0x59565955), "UYVY");

        // Test with non-printable characters (should be replaced with '.')
        assert_eq!(fourcc_to_string(0x00000000), "....");
        assert_eq!(fourcc_to_string(0x01020304), "....");
    }
}
