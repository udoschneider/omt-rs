//! Audio-specific methods for MediaFrame.

use crate::frame::MediaFrame;
use std::slice;

impl MediaFrame {
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
    pub fn as_f32_planar(&self) -> Vec<&[f32]> {
        let data = self.data();
        let samples_per_channel = self.samples_per_channel() as usize;
        let channels = self.channels() as usize;
        let samples_per_plane = samples_per_channel * std::mem::size_of::<f32>();

        let mut result = Vec::with_capacity(channels);
        for ch in 0..channels {
            let offset = ch * samples_per_plane;
            if offset + samples_per_plane <= data.len() {
                let plane_data = &data[offset..offset + samples_per_plane];
                let f32_slice = unsafe {
                    slice::from_raw_parts(plane_data.as_ptr() as *const f32, samples_per_channel)
                };
                result.push(f32_slice);
            }
        }
        result
    }
}
