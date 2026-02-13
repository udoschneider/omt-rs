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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_from_millis() {
        let timeout = Timeout::from_millis(500);
        assert_eq!(timeout.as_duration(), Duration::from_millis(500));
    }

    #[test]
    fn test_timeout_from_secs() {
        let timeout = Timeout::from_secs(10);
        assert_eq!(timeout.as_duration(), Duration::from_secs(10));
    }

    #[test]
    fn test_timeout_from_duration() {
        let duration = Duration::from_millis(1500);
        let timeout = Timeout::from_duration(duration);
        assert_eq!(timeout.as_duration(), duration);
    }

    #[test]
    fn test_timeout_as_duration() {
        let timeout = Timeout::from_millis(250);
        let duration = timeout.as_duration();
        assert_eq!(duration, Duration::from_millis(250));
    }

    #[test]
    fn test_timeout_as_millis_i32() {
        let timeout = Timeout::from_millis(1000);
        assert_eq!(timeout.as_millis_i32(), 1000);
    }

    #[test]
    fn test_timeout_as_millis_i32_zero() {
        let timeout = Timeout::from_millis(0);
        assert_eq!(timeout.as_millis_i32(), 0);
    }

    #[test]
    fn test_timeout_as_millis_i32_max_capped() {
        // Test that very large values are capped at i32::MAX
        let timeout = Timeout::from_secs(u64::MAX / 1000);
        let result = timeout.as_millis_i32();
        assert!(result <= i32::MAX);
    }

    #[test]
    fn test_timeout_from_duration_trait() {
        let duration = Duration::from_millis(750);
        let timeout: Timeout = duration.into();
        assert_eq!(timeout.as_duration(), Duration::from_millis(750));
    }

    #[test]
    fn test_timeout_to_duration_trait() {
        let timeout = Timeout::from_millis(500);
        let duration: Duration = timeout.into();
        assert_eq!(duration, Duration::from_millis(500));
    }

    #[test]
    fn test_timeout_clone() {
        let timeout1 = Timeout::from_millis(100);
        let timeout2 = timeout1.clone();
        assert_eq!(timeout1, timeout2);
    }

    #[test]
    fn test_timeout_copy() {
        let timeout1 = Timeout::from_secs(5);
        let timeout2 = timeout1;
        assert_eq!(timeout1.as_duration(), Duration::from_secs(5));
        assert_eq!(timeout2.as_duration(), Duration::from_secs(5));
    }

    #[test]
    fn test_timeout_eq() {
        let timeout1 = Timeout::from_millis(100);
        let timeout2 = Timeout::from_millis(100);
        let timeout3 = Timeout::from_millis(200);
        assert_eq!(timeout1, timeout2);
        assert_ne!(timeout1, timeout3);
    }

    #[test]
    fn test_timeout_ord() {
        let timeout1 = Timeout::from_millis(100);
        let timeout2 = Timeout::from_millis(200);
        let timeout3 = Timeout::from_millis(300);
        assert!(timeout1 < timeout2);
        assert!(timeout2 < timeout3);
        assert!(timeout1 < timeout3);
    }

    #[test]
    fn test_timeout_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let timeout1 = Timeout::from_millis(100);
        let timeout2 = Timeout::from_millis(200);
        let timeout3 = Timeout::from_millis(100);

        set.insert(timeout1);
        set.insert(timeout2);
        set.insert(timeout3);

        assert_eq!(set.len(), 2); // timeout1 and timeout3 are equal
    }

    #[test]
    fn test_timeout_debug() {
        let timeout = Timeout::from_millis(500);
        let debug_str = format!("{:?}", timeout);
        assert!(debug_str.contains("Timeout"));
    }

    #[test]
    fn test_timeout_from_millis_large_value() {
        let timeout = Timeout::from_millis(1_000_000);
        assert_eq!(timeout.as_duration(), Duration::from_millis(1_000_000));
    }

    #[test]
    fn test_timeout_from_secs_large_value() {
        let timeout = Timeout::from_secs(3600);
        assert_eq!(timeout.as_duration(), Duration::from_secs(3600));
    }

    #[test]
    fn test_timeout_as_millis_i32_small_values() {
        assert_eq!(Timeout::from_millis(1).as_millis_i32(), 1);
        assert_eq!(Timeout::from_millis(10).as_millis_i32(), 10);
        assert_eq!(Timeout::from_millis(100).as_millis_i32(), 100);
    }
}
