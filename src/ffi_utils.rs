//! Shared FFI helper utilities.

use crate::ffi;
use crate::receiver::{Statistics, Tally};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Converts a fixed-size C char array to a Rust `String`.
pub(crate) fn c_char_array_to_string(arr: &[c_char]) -> String {
    // SAFETY: c_char is a byte-sized type; we only read the provided slice length.
    let bytes = unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len()) };
    match CStr::from_bytes_until_nul(bytes) {
        Ok(cstr) => cstr.to_string_lossy().into_owned(),
        Err(_) => String::new(),
    }
}

/// Writes a Rust string into a fixed-size C char array with NUL termination.
pub(crate) fn write_c_char_array(dst: &mut [c_char], value: &str) {
    if dst.is_empty() {
        return;
    }
    dst.fill(0);
    let bytes = value.as_bytes();
    let max = dst.len() - 1;
    let copy_len = bytes.len().min(max);
    for (dst_byte, src_byte) in dst.iter_mut().take(copy_len).zip(bytes.iter()) {
        *dst_byte = *src_byte as c_char;
    }
    dst[copy_len] = 0;
}

impl From<&ffi::OMTStatistics> for Statistics {
    fn from(stats: &ffi::OMTStatistics) -> Self {
        Statistics {
            bytes_sent: stats.BytesSent as i64,
            bytes_received: stats.BytesReceived as i64,
            bytes_sent_since_last: stats.BytesSentSinceLast as i64,
            bytes_received_since_last: stats.BytesReceivedSinceLast as i64,
            frames: stats.Frames as i64,
            frames_since_last: stats.FramesSinceLast as i64,
            frames_dropped: stats.FramesDropped as i64,
            codec_time: stats.CodecTime as i64,
            codec_time_since_last: stats.CodecTimeSinceLast as i64,
            reserved1: stats.Reserved1 as i64,
            reserved2: stats.Reserved2 as i64,
            reserved3: stats.Reserved3 as i64,
            reserved4: stats.Reserved4 as i64,
            reserved5: stats.Reserved5 as i64,
            reserved6: stats.Reserved6 as i64,
            reserved7: stats.Reserved7 as i64,
        }
    }
}

impl From<&ffi::OMTTally> for Tally {
    fn from(tally: &ffi::OMTTally) -> Self {
        Tally {
            preview: tally.preview != 0,
            program: tally.program != 0,
        }
    }
}
