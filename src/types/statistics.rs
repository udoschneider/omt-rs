//! Transport and codec statistics for audio or video streams.

use crate::ffi;

#[derive(Clone, Debug, Default)]
/// Transport and codec statistics for audio or video streams.
///
/// Retrieved via [`crate::Receiver::get_video_statistics`], [`crate::Receiver::get_audio_statistics`],
/// [`crate::Sender::get_video_statistics`], or [`crate::Sender::get_audio_statistics`].
///
/// Statistics track both cumulative totals and deltas since the last statistics query,
/// allowing for rate calculations and monitoring.
pub struct Statistics {
    /// Total bytes sent (cumulative)
    pub bytes_sent: i64,
    /// Total bytes received (cumulative)
    pub bytes_received: i64,
    /// Bytes sent since last statistics query
    pub bytes_sent_since_last: i64,
    /// Bytes received since last statistics query
    pub bytes_received_since_last: i64,
    /// Total number of frames (cumulative)
    pub frames: i64,
    /// Number of frames since last statistics query
    pub frames_since_last: i64,
    /// Total number of frames dropped (cumulative)
    pub frames_dropped: i64,
    /// Time in milliseconds spent encoding/decoding (cumulative).
    /// Divide by `frames` to get average per-frame codec time.
    pub codec_time: i64,
    /// Time in milliseconds spent on the last frame encoded/decoded
    pub codec_time_since_last: i64,
    /// Reserved for future use
    pub reserved1: i64,
    /// Reserved for future use
    pub reserved2: i64,
    /// Reserved for future use
    pub reserved3: i64,
    /// Reserved for future use
    pub reserved4: i64,
    /// Reserved for future use
    pub reserved5: i64,
    /// Reserved for future use
    pub reserved6: i64,
    /// Reserved for future use
    pub reserved7: i64,
}

impl From<&ffi::OMTStatistics> for Statistics {
    fn from(stats: &ffi::OMTStatistics) -> Self {
        Statistics {
            bytes_sent: stats.BytesSent,
            bytes_received: stats.BytesReceived,
            bytes_sent_since_last: stats.BytesSentSinceLast,
            bytes_received_since_last: stats.BytesReceivedSinceLast,
            frames: stats.Frames,
            frames_since_last: stats.FramesSinceLast,
            frames_dropped: stats.FramesDropped,
            codec_time: stats.CodecTime,
            codec_time_since_last: stats.CodecTimeSinceLast,
            reserved1: stats.Reserved1,
            reserved2: stats.Reserved2,
            reserved3: stats.Reserved3,
            reserved4: stats.Reserved4,
            reserved5: stats.Reserved5,
            reserved6: stats.Reserved6,
            reserved7: stats.Reserved7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_default() {
        let stats = Statistics::default();
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.bytes_sent_since_last, 0);
        assert_eq!(stats.bytes_received_since_last, 0);
        assert_eq!(stats.frames, 0);
        assert_eq!(stats.frames_since_last, 0);
        assert_eq!(stats.frames_dropped, 0);
        assert_eq!(stats.codec_time, 0);
        assert_eq!(stats.codec_time_since_last, 0);
    }

    #[test]
    fn test_from_ffi_conversion() {
        let ffi_stats = ffi::OMTStatistics {
            BytesSent: 1000,
            BytesReceived: 2000,
            BytesSentSinceLast: 100,
            BytesReceivedSinceLast: 200,
            Frames: 30,
            FramesSinceLast: 5,
            FramesDropped: 2,
            CodecTime: 500,
            CodecTimeSinceLast: 10,
            Reserved1: 0,
            Reserved2: 0,
            Reserved3: 0,
            Reserved4: 0,
            Reserved5: 0,
            Reserved6: 0,
            Reserved7: 0,
        };

        let stats = Statistics::from(&ffi_stats);
        assert_eq!(stats.bytes_sent, 1000);
        assert_eq!(stats.bytes_received, 2000);
        assert_eq!(stats.bytes_sent_since_last, 100);
        assert_eq!(stats.bytes_received_since_last, 200);
        assert_eq!(stats.frames, 30);
        assert_eq!(stats.frames_since_last, 5);
        assert_eq!(stats.frames_dropped, 2);
        assert_eq!(stats.codec_time, 500);
        assert_eq!(stats.codec_time_since_last, 10);
    }

    #[test]
    fn test_from_ffi_with_reserved_fields() {
        let ffi_stats = ffi::OMTStatistics {
            BytesSent: 0,
            BytesReceived: 0,
            BytesSentSinceLast: 0,
            BytesReceivedSinceLast: 0,
            Frames: 0,
            FramesSinceLast: 0,
            FramesDropped: 0,
            CodecTime: 0,
            CodecTimeSinceLast: 0,
            Reserved1: 111,
            Reserved2: 222,
            Reserved3: 333,
            Reserved4: 444,
            Reserved5: 555,
            Reserved6: 666,
            Reserved7: 777,
        };

        let stats = Statistics::from(&ffi_stats);
        assert_eq!(stats.reserved1, 111);
        assert_eq!(stats.reserved2, 222);
        assert_eq!(stats.reserved3, 333);
        assert_eq!(stats.reserved4, 444);
        assert_eq!(stats.reserved5, 555);
        assert_eq!(stats.reserved6, 666);
        assert_eq!(stats.reserved7, 777);
    }

    #[test]
    fn test_clone() {
        let stats1 = Statistics {
            bytes_sent: 1000,
            bytes_received: 2000,
            ..Default::default()
        };
        let stats2 = stats1.clone();
        assert_eq!(stats1.bytes_sent, stats2.bytes_sent);
        assert_eq!(stats1.bytes_received, stats2.bytes_received);
    }

    #[test]
    fn test_debug() {
        let stats = Statistics {
            bytes_sent: 1000,
            frames: 30,
            ..Default::default()
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("Statistics"));
        assert!(debug_str.contains("1000"));
        assert!(debug_str.contains("30"));
    }

    #[test]
    fn test_negative_values() {
        let ffi_stats = ffi::OMTStatistics {
            BytesSent: -100,
            BytesReceived: -200,
            BytesSentSinceLast: -10,
            BytesReceivedSinceLast: -20,
            Frames: -5,
            FramesSinceLast: -1,
            FramesDropped: -2,
            CodecTime: -50,
            CodecTimeSinceLast: -5,
            Reserved1: 0,
            Reserved2: 0,
            Reserved3: 0,
            Reserved4: 0,
            Reserved5: 0,
            Reserved6: 0,
            Reserved7: 0,
        };

        let stats = Statistics::from(&ffi_stats);
        assert_eq!(stats.bytes_sent, -100);
        assert_eq!(stats.bytes_received, -200);
        assert_eq!(stats.frames, -5);
    }

    #[test]
    fn test_large_values() {
        let ffi_stats = ffi::OMTStatistics {
            BytesSent: i64::MAX,
            BytesReceived: i64::MAX / 2,
            BytesSentSinceLast: 1_000_000_000,
            BytesReceivedSinceLast: 2_000_000_000,
            Frames: 1_000_000,
            FramesSinceLast: 60,
            FramesDropped: 100,
            CodecTime: 500_000,
            CodecTimeSinceLast: 16,
            Reserved1: 0,
            Reserved2: 0,
            Reserved3: 0,
            Reserved4: 0,
            Reserved5: 0,
            Reserved6: 0,
            Reserved7: 0,
        };

        let stats = Statistics::from(&ffi_stats);
        assert_eq!(stats.bytes_sent, i64::MAX);
        assert_eq!(stats.bytes_received, i64::MAX / 2);
        assert_eq!(stats.frames, 1_000_000);
    }
}
