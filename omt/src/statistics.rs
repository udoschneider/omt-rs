//! Statistics tracking for OMT senders and receivers.

use std::time::Duration;

/// Statistics for video or audio transmission/reception.
///
/// Provides metrics about data transfer, frame counts, codec performance,
/// and other operational statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Statistics {
    /// Total bytes sent.
    pub bytes_sent: i64,
    /// Total bytes received.
    pub bytes_received: i64,
    /// Bytes sent since last call.
    pub bytes_sent_since_last: i64,
    /// Bytes received since last call.
    pub bytes_received_since_last: i64,
    /// Total frames processed.
    pub frames: i64,
    /// Frames processed since last call.
    pub frames_since_last: i64,
    /// Total frames dropped.
    pub frames_dropped: i64,
    /// Time in milliseconds spent encoding/decoding so far.
    ///
    /// Can be divided by `frames` to calculate average per-frame time.
    pub codec_time: i64,
    /// Time in milliseconds spent on the last frame.
    pub codec_time_since_last: i64,
}

impl Statistics {
    /// Creates a new statistics instance with all values set to zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the codec time as a `Duration`.
    pub fn codec_duration(&self) -> Duration {
        Duration::from_millis(self.codec_time as u64)
    }

    /// Returns the last codec time as a `Duration`.
    pub fn codec_duration_since_last(&self) -> Duration {
        Duration::from_millis(self.codec_time_since_last as u64)
    }

    /// Returns the average codec time per frame in milliseconds.
    ///
    /// Returns `None` if no frames have been processed.
    pub fn average_codec_time_ms(&self) -> Option<f64> {
        if self.frames > 0 {
            Some(self.codec_time as f64 / self.frames as f64)
        } else {
            None
        }
    }

    /// Returns the total bytes transferred (sent + received).
    pub fn total_bytes(&self) -> i64 {
        self.bytes_sent + self.bytes_received
    }

    /// Returns the bytes transferred since last call (sent + received).
    pub fn bytes_since_last(&self) -> i64 {
        self.bytes_sent_since_last + self.bytes_received_since_last
    }

    /// Returns the frame drop rate as a percentage.
    ///
    /// Returns `None` if no frames have been processed.
    pub fn drop_rate(&self) -> Option<f64> {
        let total = self.frames + self.frames_dropped;
        if total > 0 {
            Some((self.frames_dropped as f64 / total as f64) * 100.0)
        } else {
            None
        }
    }

    /// Converts from FFI representation.
    pub(crate) fn from_ffi(ffi: &omt_sys::OMTStatistics) -> Self {
        Self {
            bytes_sent: ffi.BytesSent,
            bytes_received: ffi.BytesReceived,
            bytes_sent_since_last: ffi.BytesSentSinceLast,
            bytes_received_since_last: ffi.BytesReceivedSinceLast,
            frames: ffi.Frames,
            frames_since_last: ffi.FramesSinceLast,
            frames_dropped: ffi.FramesDropped,
            codec_time: ffi.CodecTime,
            codec_time_since_last: ffi.CodecTimeSinceLast,
        }
    }
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Frames: {} (dropped: {}), Bytes: sent={}, recv={}, Codec time: {}ms",
            self.frames, self.frames_dropped, self.bytes_sent, self.bytes_received, self.codec_time
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_new() {
        let stats = Statistics::new();
        assert_eq!(stats.frames, 0);
        assert_eq!(stats.bytes_sent, 0);
    }

    #[test]
    fn test_average_codec_time() {
        let mut stats = Statistics::new();
        assert_eq!(stats.average_codec_time_ms(), None);

        stats.codec_time = 1000;
        stats.frames = 10;
        assert_eq!(stats.average_codec_time_ms(), Some(100.0));
    }

    #[test]
    fn test_total_bytes() {
        let mut stats = Statistics::new();
        stats.bytes_sent = 1000;
        stats.bytes_received = 500;
        assert_eq!(stats.total_bytes(), 1500);
    }

    #[test]
    fn test_drop_rate() {
        let mut stats = Statistics::new();
        assert_eq!(stats.drop_rate(), None);

        stats.frames = 90;
        stats.frames_dropped = 10;
        assert_eq!(stats.drop_rate(), Some(10.0));
    }

    #[test]
    fn test_codec_duration() {
        let mut stats = Statistics::new();
        stats.codec_time = 5000;
        stats.codec_time_since_last = 100;

        assert_eq!(stats.codec_duration(), Duration::from_millis(5000));
        assert_eq!(
            stats.codec_duration_since_last(),
            Duration::from_millis(100)
        );
    }
}
