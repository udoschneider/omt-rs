//! Core types and enumerations for the OMT library.

use bitflags::bitflags;
use std::fmt;

use crate::MAX_STRING_LENGTH;
use crate::error::{Error, Result};

bitflags! {
    /// Type of media frame.
    ///
    /// This is a bitflags type that can be combined using bitwise OR operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameType: u32 {
        /// No frame type.
        const NONE = omt_sys::OMTFrameType_None;
        /// Metadata frame.
        const METADATA = omt_sys::OMTFrameType_Metadata;
        /// Video frame.
        const VIDEO = omt_sys::OMTFrameType_Video;
        /// Audio frame.
        const AUDIO = omt_sys::OMTFrameType_Audio;
    }
}

impl FrameType {
    /// All frame types (Video, Audio, and Metadata).
    pub const ALL: Self = Self::VIDEO.union(Self::AUDIO).union(Self::METADATA);

    /// Video and Audio frames.
    pub const VIDEO_AUDIO: Self = Self::VIDEO.union(Self::AUDIO);

    /// Creates a `FrameType` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        Self::from_bits(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.bits()
    }
}

/// Media codec types supported by OMT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Codec {
    /// VMX1 - Fast video codec.
    Vmx1 = omt_sys::OMTCodec_VMX1,
    /// FPA1 - Floating-point Planar Audio 32bit.
    Fpa1 = omt_sys::OMTCodec_FPA1,
    /// UYVY - 16bpp YUV format.
    Uyvy = omt_sys::OMTCodec_UYVY,
    /// YUY2 - 16bpp YUV format YUYV pixel order.
    Yuy2 = omt_sys::OMTCodec_YUY2,
    /// BGRA - 32bpp RGBA format (Same as ARGB32 on Win32).
    Bgra = omt_sys::OMTCodec_BGRA,
    /// NV12 - Planar 4:2:0 YUV format. Y plane followed by interleaved half height U/V plane.
    Nv12 = omt_sys::OMTCodec_NV12,
    /// YV12 - Planar 4:2:0 YUV format. Y plane followed by half height U and V planes.
    Yv12 = omt_sys::OMTCodec_YV12,
    /// UYVA - 16pp YUV format immediately followed by an alpha plane.
    Uyva = omt_sys::OMTCodec_UYVA,
    /// P216 - Planar 4:2:2 YUV format. 16bit Y plane followed by interlaved 16bit UV plane.
    P216 = omt_sys::OMTCodec_P216,
    /// PA16 - Same as P216 followed by an additional 16bit alpha plane.
    Pa16 = omt_sys::OMTCodec_PA16,
}

impl Codec {
    /// Creates a `Codec` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTCodec_VMX1 => Some(Self::Vmx1),
            omt_sys::OMTCodec_FPA1 => Some(Self::Fpa1),
            omt_sys::OMTCodec_UYVY => Some(Self::Uyvy),
            omt_sys::OMTCodec_YUY2 => Some(Self::Yuy2),
            omt_sys::OMTCodec_BGRA => Some(Self::Bgra),
            omt_sys::OMTCodec_NV12 => Some(Self::Nv12),
            omt_sys::OMTCodec_YV12 => Some(Self::Yv12),
            omt_sys::OMTCodec_UYVA => Some(Self::Uyva),
            omt_sys::OMTCodec_P216 => Some(Self::P216),
            omt_sys::OMTCodec_PA16 => Some(Self::Pa16),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }
}

/// Video encoding quality level.
///
/// If set to `Default`, the Sender is configured to allow suggestions from all Receivers.
/// The highest suggest amongst all receivers is then selected.
///
/// If a Receiver is set to `Default`, then it will defer the quality to whatever is set
/// amongst other Receivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Quality {
    /// Default quality (allows receiver suggestions).
    Default = omt_sys::OMTQuality_Default,
    /// Low quality encoding.
    Low = omt_sys::OMTQuality_Low,
    /// Medium quality encoding.
    Medium = omt_sys::OMTQuality_Medium,
    /// High quality encoding.
    High = omt_sys::OMTQuality_High,
}

impl Quality {
    /// Creates a `Quality` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTQuality_Default => Some(Self::Default),
            omt_sys::OMTQuality_Low => Some(Self::Low),
            omt_sys::OMTQuality_Medium => Some(Self::Medium),
            omt_sys::OMTQuality_High => Some(Self::High),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }
}

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

/// Flags for video frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoFlags(u32);

impl VideoFlags {
    /// No flags set.
    pub const NONE: Self = Self(omt_sys::OMTVideoFlags_None);
    /// Frames are interlaced.
    pub const INTERLACED: Self = Self(omt_sys::OMTVideoFlags_Interlaced);
    /// Frames contain an alpha channel.
    pub const ALPHA: Self = Self(omt_sys::OMTVideoFlags_Alpha);
    /// Alpha channel is premultiplied (when combined with ALPHA).
    pub const PRE_MULTIPLIED: Self = Self(omt_sys::OMTVideoFlags_PreMultiplied);
    /// Frame is a special 1/8th preview frame.
    pub const PREVIEW: Self = Self(omt_sys::OMTVideoFlags_Preview);
    /// High bit depth frame (P216 or PA16 formats).
    pub const HIGH_BIT_DEPTH: Self = Self(omt_sys::OMTVideoFlags_HighBitDepth);

    /// Creates flags from raw bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Checks if the flags contain the given flag.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Creates flags from FFI value.
    pub(crate) fn from_ffi(value: u32) -> Self {
        Self(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for VideoFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for VideoFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for VideoFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

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

/// Flags for receiver configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReceiveFlags(u32);

impl ReceiveFlags {
    /// No flags set.
    pub const NONE: Self = Self(omt_sys::OMTReceiveFlags_None);
    /// Receive only a 1/8th preview of the video.
    pub const PREVIEW: Self = Self(omt_sys::OMTReceiveFlags_Preview);
    /// Include a copy of the compressed VMX1 video frames.
    pub const INCLUDE_COMPRESSED: Self = Self(omt_sys::OMTReceiveFlags_IncludeCompressed);
    /// Include only the compressed VMX1 video frame without decoding.
    pub const COMPRESSED_ONLY: Self = Self(omt_sys::OMTReceiveFlags_CompressedOnly);

    /// Creates flags from raw bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Checks if the flags contain the given flag.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Creates flags from FFI value.
    pub(crate) fn from_ffi(value: u32) -> Self {
        Self(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for ReceiveFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ReceiveFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ReceiveFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Information describing the sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderInfo {
    /// Product name.
    pub product_name: String,
    /// Manufacturer name.
    pub manufacturer: String,
    /// Version string.
    pub version: String,
}

impl SenderInfo {
    /// Creates a new `SenderInfo`.
    pub fn new(product_name: String, manufacturer: String, version: String) -> Self {
        Self {
            product_name,
            manufacturer,
            version,
        }
    }

    /// Creates from FFI struct.
    pub(crate) fn from_ffi(ffi: &omt_sys::OMTSenderInfo) -> Result<Self> {
        Ok(Self {
            product_name: Self::c_array_to_string(&ffi.ProductName)?,
            manufacturer: Self::c_array_to_string(&ffi.Manufacturer)?,
            version: Self::c_array_to_string(&ffi.Version)?,
        })
    }

    /// Converts to FFI struct.
    pub(crate) fn to_ffi(&self) -> Result<omt_sys::OMTSenderInfo> {
        let mut ffi = omt_sys::OMTSenderInfo {
            ProductName: [0; MAX_STRING_LENGTH],
            Manufacturer: [0; MAX_STRING_LENGTH],
            Version: [0; MAX_STRING_LENGTH],
            Reserved1: [0; MAX_STRING_LENGTH],
            Reserved2: [0; MAX_STRING_LENGTH],
            Reserved3: [0; MAX_STRING_LENGTH],
        };

        Self::string_to_c_array(&self.product_name, &mut ffi.ProductName)?;
        Self::string_to_c_array(&self.manufacturer, &mut ffi.Manufacturer)?;
        Self::string_to_c_array(&self.version, &mut ffi.Version)?;

        Ok(ffi)
    }

    fn c_array_to_string(arr: &[i8; MAX_STRING_LENGTH]) -> Result<String> {
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len()) };
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..null_pos])
            .map(|s| s.to_string())
            .map_err(|_| Error::InvalidUtf8)
    }

    fn string_to_c_array(s: &str, arr: &mut [i8; MAX_STRING_LENGTH]) -> Result<()> {
        let bytes = s.as_bytes();
        if bytes.len() >= MAX_STRING_LENGTH {
            return Err(Error::BufferTooSmall {
                required: bytes.len() + 1,
                provided: MAX_STRING_LENGTH,
            });
        }

        for (i, &byte) in bytes.iter().enumerate() {
            arr[i] = byte as i8;
        }
        arr[bytes.len()] = 0;

        Ok(())
    }
}

impl Default for SenderInfo {
    fn default() -> Self {
        Self {
            product_name: String::new(),
            manufacturer: String::new(),
            version: String::new(),
        }
    }
}

impl fmt::Display for SenderInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} by {} (v{})",
            self.product_name, self.manufacturer, self.version
        )
    }
}
