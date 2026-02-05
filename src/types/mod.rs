//! High-level types for Open Media Transport (OMT).
//!
//! These types provide ergonomic enums, flags, and frame views on top of the
//! low-level FFI structs (including `OMTMediaFrame` from `libomt.h`).
//! Frames are borrowed views whose lifetime is limited to the next receive
//! call on the same sender/receiver, matching the header's ownership rules.
//!
//! Timestamps use the OMT timebase (10,000,000 ticks per second) and should
//! represent the original capture time for correct synchronization. A timestamp
//! of `-1` tells the sender to generate timestamps and pace delivery by the
//! frame or sample rate.
//!
//! Metadata frames and per-frame metadata payloads are UTF-8 XML strings with a
//! terminating null; lengths include the null byte.
//!
//! For protocol context, see: <https://github.com/openmediatransport>

use crate::ffi;
pub use crate::receiver::AudioFrame;
pub use crate::video_frame::VideoFrame;
use bitflags::bitflags;
use std::time::Duration;

mod address;
pub use address::Address;

mod name;
pub use name::Name;

/// Standard timeout type used by the safe API.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timeout(Duration);

impl Timeout {
    pub fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    pub fn as_duration(self) -> Duration {
        self.0
    }

    pub fn as_millis_i32(self) -> i32 {
        self.0.as_millis().min(u128::from(i32::MAX as u32)) as i32
    }
}

impl From<Duration> for Timeout {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<Timeout> for Duration {
    fn from(value: Timeout) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Stream type selector for receive/send operations.
pub enum FrameType {
    None,
    Metadata,
    Video,
    Audio,
}

impl From<ffi::OMTFrameType> for FrameType {
    fn from(value: ffi::OMTFrameType) -> Self {
        match value {
            ffi::OMTFrameType::Metadata => FrameType::Metadata,
            ffi::OMTFrameType::Video => FrameType::Video,
            ffi::OMTFrameType::Audio => FrameType::Audio,
            _ => FrameType::None,
        }
    }
}

impl From<FrameType> for ffi::OMTFrameType {
    fn from(value: FrameType) -> Self {
        match value {
            FrameType::Metadata => ffi::OMTFrameType::Metadata,
            FrameType::Video => ffi::OMTFrameType::Video,
            FrameType::Audio => ffi::OMTFrameType::Audio,
            FrameType::None => ffi::OMTFrameType::None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Supported pixel formats and codec identifiers used by OMT.
pub enum Codec {
    VMX1,
    FPA1,
    UYVY,
    YUY2,
    BGRA,
    NV12,
    YV12,
    UYVA,
    P216,
    PA16,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Sender quality selection.
pub enum Quality {
    Default,
    Low,
    Medium,
    High,
}

impl From<ffi::OMTQuality> for Quality {
    fn from(value: ffi::OMTQuality) -> Self {
        match value {
            ffi::OMTQuality::Low => Quality::Low,
            ffi::OMTQuality::Medium => Quality::Medium,
            ffi::OMTQuality::High => Quality::High,
            _ => Quality::Default,
        }
    }
}

impl From<Quality> for ffi::OMTQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Low => ffi::OMTQuality::Low,
            Quality::Medium => ffi::OMTQuality::Medium,
            Quality::High => ffi::OMTQuality::High,
            Quality::Default => ffi::OMTQuality::Default,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Color space metadata for video frames.
pub enum ColorSpace {
    Undefined,
    BT601,
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

bitflags! {
    /// Bitflags describing video frame properties (alpha, interlaced, etc.).
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[allow(non_snake_case)]
    pub struct VideoFlags: i32 {
        const NONE = 0;
        const INTERLACED = 1;
        const ALPHA = 2;
        const PREMULTIPLIED = 4;
        const PREVIEW = 8;
        const HIGH_BIT_DEPTH = 16;
    }
}

impl From<ffi::OMTVideoFlags> for VideoFlags {
    fn from(value: ffi::OMTVideoFlags) -> Self {
        VideoFlags::from_bits_truncate(value)
    }
}

impl From<VideoFlags> for i32 {
    fn from(value: VideoFlags) -> Self {
        value.bits()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Preferred pixel format negotiation for receivers.
pub enum PreferredVideoFormat {
    UYVY,
    UYVYorBGRA,
    BGRA,
    UYVYorUYVA,
    UYVYorUYVAorP216orPA16,
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

bitflags! {
    /// Receiver configuration flags (preview/compressed delivery).
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[allow(non_snake_case)]
    pub struct ReceiveFlags: i32 {
        const NONE = 0;
        const PREVIEW = 1;
        const INCLUDE_COMPRESSED = 2;
        const COMPRESSED_ONLY = 4;
    }
}

impl From<ffi::OMTReceiveFlags> for ReceiveFlags {
    fn from(value: ffi::OMTReceiveFlags) -> Self {
        ReceiveFlags::from_bits_truncate(value)
    }
}

impl From<ReceiveFlags> for i32 {
    fn from(value: ReceiveFlags) -> Self {
        value.bits()
    }
}

/// Requested output format for video data conversion.
pub enum VideoDataFormat {
    /// 8-bit per component RGB.
    RGB,
    /// 8-bit per component RGBA, straight alpha.
    RGBA,
    /// 16-bit per component RGB.
    RGB16,
    /// 16-bit per component RGBA, straight alpha.
    RGBA16,
}

/// Borrowed view of a received media frame.
///
/// Valid only until the next receive call on the same receiver/sender.
pub struct FrameRef<'a> {
    raw: &'a ffi::OMTMediaFrame,
}

impl<'a> FrameRef<'a> {
    pub(crate) fn new(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self { raw }
    }

    /// Returns the OMT frame type (video/audio/metadata).
    pub fn frame_type(&self) -> FrameType {
        self.raw.Type.into()
    }

    /// Returns the frame timestamp in OMT ticks (10,000,000 per second).
    pub fn timestamp(&self) -> i64 {
        self.raw.Timestamp
    }

    pub fn codec(&self) -> Codec {
        self.raw.Codec.into()
    }

    pub fn video(&self) -> Option<VideoFrame<'a>> {
        if self.frame_type() != FrameType::Video {
            return None;
        }
        Some(VideoFrame::new(self.raw))
    }

    pub fn audio(&self) -> Option<AudioFrame<'a>> {
        if self.frame_type() != FrameType::Audio {
            return None;
        }
        Some(AudioFrame::new(self.raw))
    }

    pub fn metadata(&self) -> Option<&'a [u8]> {
        if self.frame_type() != FrameType::Metadata {
            return None;
        }
        if self.raw.Data.is_null() || self.raw.DataLength <= 0 {
            return None;
        }
        let len = self.raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.Data as *const u8, len) })
    }
}
