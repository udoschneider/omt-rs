//! High-level types for Open Media Transport (OMT).
//!
//! Provides ergonomic Rust types wrapping the low-level FFI structs from `libomt.h`.
//!
//! ## Frame Lifetimes
//! Frame views are borrowed and valid only until the next receive call on the same sender/receiver.
//!
//! ## Timestamps
//! Uses OMT timebase: 10,000,000 ticks/second (1 second = 10,000,000 ticks).
//!
//! Timestamps should represent the accurate time the frame or audio sample was generated at the
//! original source and be used on the receiving end to synchronize and record to file as a
//! presentation timestamp (pts).
//!
//! A special value of `-1` can be specified to tell the Sender to generate timestamps and
//! throttle as required to maintain the specified frame rate or sample rate.
//!
//! ## Metadata
//! Metadata frames and per-frame payloads are UTF-8 XML strings. Null terminators are
//! handled automatically—provide standard Rust strings without null bytes.
//!
//! See: <https://github.com/openmediatransport>

pub use crate::media_frame::MediaFrame;
use std::time::Duration;

mod address;
pub use address::Address;

mod codec;
pub use codec::Codec;

mod color_space;
pub use color_space::ColorSpace;

mod frame_rate;
pub use frame_rate::FrameRate;

mod frame_type;
pub use frame_type::FrameType;

mod name;
pub use name::Name;

mod quality;
pub use quality::Quality;

mod preferred_video_format;
pub use preferred_video_format::PreferredVideoFormat;

mod video_flags;
pub use video_flags::VideoFlags;

mod receive_flags;
pub use receive_flags::ReceiveFlags;

mod sender_info;
pub use sender_info::SenderInfo;

mod statistics;
pub use statistics::Statistics;

mod tally;
pub use tally::Tally;

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
