//! Audio-specific methods for MediaFrame.

use crate::frame::MediaFrame;
use std::slice;

impl<'a> MediaFrame<'a> {
    /// Returns the sample rate (e.g., 48000, 44100).
    ///
    /// This method is only meaningful for audio frames.
    pub fn sample_rate(&self) -> i32 {
        self.ffi.SampleRate
    }

    /// Returns the number of audio channels (maximum 32).
    ///
    /// This method is only meaningful for audio frames.
    pub fn channels(&self) -> i32 {
        self.ffi.Channels
    }

    /// Returns the number of samples per channel.
    ///
    /// This method is only meaningful for audio frames.
    pub fn samples_per_channel(&self) -> i32 {
        self.ffi.SamplesPerChannel
    }

    /// Returns the audio data as f32 slices (one per channel).
    ///
    /// Each slice contains `samples_per_channel` samples.
    /// This method is only meaningful for audio frames.
    ///
    /// Returns `None` if the data is not properly aligned or sized for f32 conversion.
    ///
    /// The returned slices borrow from `self` and cannot outlive this frame.
    pub fn as_f32_planar(&self) -> Option<Vec<&[f32]>> {
        let data = self.data();
        let samples_per_channel = self.samples_per_channel();
        let channels = self.channels();

        // Reject negative counts before casting: a negative `i32` becomes a huge
        // `usize`, and the size products below must not wrap (mirrors the
        // overflow-checked gate the video path uses in `required_input_len`).
        if samples_per_channel < 0 || channels < 0 {
            return None;
        }
        let samples_per_channel = samples_per_channel as usize;
        let channels = channels as usize;
        let samples_per_plane = samples_per_channel.checked_mul(std::mem::size_of::<f32>())?;

        // Validate total data size
        let expected_size = channels.checked_mul(samples_per_plane)?;
        if data.len() != expected_size {
            return None;
        }

        // Validate alignment for f32 access
        if !(data.as_ptr() as usize).is_multiple_of(std::mem::align_of::<f32>()) {
            return None;
        }

        let mut result = Vec::with_capacity(channels);
        for ch in 0..channels {
            let offset = ch * samples_per_plane;
            if offset + samples_per_plane <= data.len() {
                let plane_data = &data[offset..offset + samples_per_plane];

                // SAFETY: We've validated:
                // 1. The data length matches expected size
                // 2. The pointer is properly aligned for f32
                // 3. The slice bounds are within the valid data range
                // 4. The result borrows `self` (via `data`), so the f32 slices
                //    cannot outlive the frame that owns/backs the byte buffer
                let f32_slice = unsafe {
                    slice::from_raw_parts(plane_data.as_ptr() as *const f32, samples_per_channel)
                };
                result.push(f32_slice);
            }
        }
        Some(result)
    }
}
