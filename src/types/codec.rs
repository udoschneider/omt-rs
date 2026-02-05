//! Codec definitions for Open Media Transport (OMT).
//!
//! This module defines the `Codec` enum which represents supported pixel formats
//! and codec identifiers used by OMT. Each codec corresponds to a specific media
//! format for video, audio, or metadata transmission.
//!
//! For detailed protocol specifications and usage guidelines, refer to
//! [`libomt.h`](https://github.com/openmediatransport/libomt/blob/main/libomt.h).
//!
//! # Supported Codecs
//!
//! Based on `libomt.h` documentation:
//!
//! ## Video Codecs
//!
//! - **VMX1**: Fast video compression codec. When sending, supports uncompressed
//!   formats that get compressed to VMX1. When receiving with `IncludeCompressed`
//!   or `CompressedOnly` flags, provides original compressed VMX1 frames.
//!
//! - **UYVY**: 16 bits per pixel YUV 4:2:2 format. This is the fastest uncompressed
//!   format when no alpha channel is required. Chroma samples are co-sited with
//!   even luma samples.
//!
//! - **YUY2**: 16 bits per pixel YUV 4:2:2 format with YUYV pixel order.
//!
//! - **UYVA**: 16 bits per pixel YUV 4:2:2 format immediately followed by an
//!   alpha plane. When alpha flags are not set, UYVA is treated as UYVY.
//!
//! - **NV12**: Planar 4:2:0 YUV format. Y plane followed by interleaved half-height
//!   U/V plane. Commonly used in hardware acceleration APIs.
//!
//! - **YV12**: Planar 4:2:0 YUV format. Y plane followed by separate half-height
//!   U and V planes.
//!
//! - **BGRA**: 32 bits per pixel RGBA format (same as ARGB32 on Win32). When alpha
//!   flags are not set, BGRA is treated as BGRX (alpha channel ignored).
//!
//! - **P216**: Planar 4:2:2 YUV format with 16-bit precision. 16-bit Y plane
//!   followed by interleaved 16-bit UV plane. Used for high bit depth content.
//!
//! - **PA16**: Same as P216 followed by an additional 16-bit alpha plane.
//!   Provides high bit depth with alpha support.
//!
//! ## Audio Codec
//!
//! - **FPA1**: Floating-point Planar Audio, 32-bit precision. The only supported
//!   audio format for both sending and receiving.
//!
//! # Usage Notes
//!
//! ## Sending Frames
//!
//! When sending video frames, the following codecs are supported:
//! - Uncompressed: `UYVY`, `YUY2`, `NV12`, `YV12`, `BGRA`, `UYVA`
//! - Compressed: `VMX1`
//!
//! BGRA will be treated as BGRX and UYVA as UYVY when alpha flags are not set.
//!
//! ## Receiving Frames
//!
//! When receiving video frames, the following uncompressed formats are supported:
//! - `UYVY`, `UYVA`, `BGRA`, `BGRX`
//!
//! The format can be selected using `PreferredVideoFormat` to balance performance
//! and feature requirements.
//!
//! ## High Bit Depth Support
//!
//! For high bit depth content (10-bit or more):
//! - Use `P216` for YUV 4:2:2 without alpha
//! - Use `PA16` for YUV 4:2:2 with alpha
//! - Set `VideoFlags::HIGH_BIT_DEPTH` flag automatically added by sender for P216/PA16
//! - For VMX1 compressed data originally from P216/PA16, set `HIGH_BIT_DEPTH` manually
//!
//! # Examples
//!
//! ```rust
//! use omt::Codec;
//!
//! // Creating a video frame with BGRA codec
//! let video_codec = Codec::BGRA;
//!
//! // Creating an audio frame with FPA1 codec
//! let audio_codec = Codec::FPA1;
//!
//! // Getting the FOURCC code
//! let fourcc = Codec::UYVY.fourcc();
//! assert_eq!(fourcc, 0x59565955); // 'UYVY' in little-endian
//!
//! // Converting from FFI representation (requires internal ffi module)
//! // Note: The ffi module is private, this is for illustration only
//! // let ffi_codec = ffi::OMTCodec::NV12;
//! // let codec = Codec::from(ffi_codec);
//! // assert_eq!(codec, Codec::NV12);
//! ```

use crate::ffi;

/// Supported pixel formats and codec identifiers used by OMT.
///
/// Each variant represents a specific media format for video, audio, or metadata
/// transmission. The enum maps directly to the `OMTCodec` enum in `libomt.h`.
///
/// # Variants
///
/// ## Video Codecs
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Codec {
    /// Fast video compression codec.
    ///
    /// When sending: Supports uncompressed formats that get compressed to VMX1.
    /// When receiving with `IncludeCompressed` or `CompressedOnly` flags:
    /// Provides original compressed VMX1 frames for recording or processing.
    VMX1,

    /// Floating-point Planar Audio, 32-bit precision.
    ///
    /// The only supported audio format for both sending and receiving.
    /// Represents planar 32-bit floating point audio samples.
    FPA1,

    /// 16 bits per pixel YUV 4:2:2 format.
    ///
    /// Chroma samples are co-sited with even luma samples.
    /// This is the fastest uncompressed format when no alpha channel is required.
    UYVY,

    /// 16 bits per pixel YUV 4:2:2 format with YUYV pixel order.
    YUY2,

    /// 32 bits per pixel RGBA format (same as ARGB32 on Win32).
    ///
    /// When alpha flags are not set, BGRA is treated as BGRX (alpha channel ignored).
    BGRA,

    /// Planar 4:2:0 YUV format.
    ///
    /// Y plane followed by interleaved half-height U/V plane.
    /// Commonly used in hardware acceleration APIs.
    NV12,

    /// Planar 4:2:0 YUV format.
    ///
    /// Y plane followed by separate half-height U and V planes.
    YV12,

    /// 16 bits per pixel YUV 4:2:2 format with alpha plane.
    ///
    /// UYVY format immediately followed by an alpha plane.
    /// When alpha flags are not set, UYVA is treated as UYVY.
    UYVA,

    /// Planar 4:2:2 YUV format with 16-bit precision.
    ///
    /// 16-bit Y plane followed by interleaved 16-bit UV plane.
    /// Used for high bit depth content (10-bit or more).
    /// Sender automatically adds `VideoFlags::HIGH_BIT_DEPTH` for this format.
    P216,

    /// Planar 4:2:2 YUV format with 16-bit precision and alpha.
    ///
    /// Same as P216 followed by an additional 16-bit alpha plane.
    /// Provides high bit depth with alpha support.
    /// Sender automatically adds `VideoFlags::HIGH_BIT_DEPTH` for this format.
    PA16,

    /// Unknown codec with raw integer value.
    ///
    /// Used for forward compatibility with future codec additions.
    /// The raw value is the FOURCC code as a 32-bit integer.
    Unknown(i32),
}

impl From<ffi::OMTCodec> for Codec {
    /// Converts from the FFI `OMTCodec` representation.
    ///
    /// This conversion handles all known codec values from `libomt.h` and
    /// preserves unknown values in the `Unknown` variant for forward compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::Codec;
    ///
    /// // The From<ffi::OMTCodec> implementation is used internally
    /// // when converting from FFI structures. In user code, you typically
    /// // work with Codec values directly or get them from frame objects.
    ///
    /// // Example of using Codec values from frame accessors:
    /// // let frame = receiver.receive(Timeout::from_millis(100)).unwrap();
    /// // let codec = frame.codec();
    /// // match codec {
    /// //     Codec::BGRA => println!("Received BGRA frame"),
    /// //     Codec::UYVY => println!("Received UYVY frame"),
    /// //     _ => println!("Received other format: {:?}", codec),
    /// // }
    /// ```
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
    /// Returns the FOURCC code for this codec as a `u32`.
    ///
    /// FOURCC (Four Character Code) is a sequence of four bytes used to
    /// identify data formats. In OMT, these codes are represented as
    /// little-endian 32-bit integers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::Codec;
    ///
    /// // BGRA FOURCC: 'B'=0x42, 'G'=0x47, 'R'=0x52, 'A'=0x41
    /// // Little-endian: 0x41_52_47_42 = 0x41524742
    /// assert_eq!(Codec::BGRA.fourcc(), 0x41524742);
    ///
    /// // UYVY FOURCC: 'U'=0x55, 'Y'=0x59, 'V'=0x56, 'Y'=0x59
    /// // Little-endian: 0x59_56_59_55 = 0x59565955
    /// assert_eq!(Codec::UYVY.fourcc(), 0x59565955);
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

        // Test unknown codec - we can't safely create an invalid ffi::OMTCodec,
        // but we can test that Unknown variant works correctly
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

        // Test unknown codec
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
        // Both should be usable after assignment (Copy trait)
        assert_eq!(codec1, Codec::UYVY);
        assert_eq!(codec2, Codec::UYVY);
    }
}
