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
