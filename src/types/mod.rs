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
//! Metadata frames and per-frame metadata payloads are UTF-8 XML strings.
//!
//! **Important:** Although `libomt.h` explicitly states that metadata strings must
//! *include* the null terminator, this high-level Rust wrapper handles this automatically.
//! Functions accepting metadata strings ensure that the passed string does *not* include
//! a null character and add it behind the scenes. The length passed to the C API includes
//! the null byte, but Rust users should provide normal Rust strings without null terminators.
//!
//! For protocol context, see: <https://github.com/openmediatransport>

use crate::ffi;
pub use crate::media_frame::MediaFrame;
use bitflags::bitflags;
use std::time::Duration;

mod address;
pub use address::Address;

mod codec;
pub use codec::Codec;

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
