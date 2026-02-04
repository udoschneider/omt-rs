use crate::ffi;

/// Audio-specific accessors for a received media frame.
pub struct AudioFrame<'a> {
    raw: &'a ffi::OMTMediaFrame,
}

impl<'a> AudioFrame<'a> {
    pub(crate) fn new(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self { raw }
    }

    pub fn sample_rate(&self) -> i32 {
        self.raw.SampleRate as i32
    }

    pub fn channels(&self) -> i32 {
        self.raw.Channels as i32
    }

    pub fn samples_per_channel(&self) -> i32 {
        self.raw.SamplesPerChannel as i32
    }

    pub fn raw_data(&self) -> Option<&'a [u8]> {
        if self.raw.Data.is_null() || self.raw.DataLength <= 0 {
            return None;
        }
        let len = self.raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.Data as *const u8, len) })
    }

    pub fn data(&self) -> Option<Vec<Vec<f32>>> {
        let data = self.raw_data()?;
        let channels = self.channels();
        let samples_per_channel = self.samples_per_channel();

        if channels <= 0 || samples_per_channel <= 0 {
            return None;
        }

        let channels = channels as usize;
        let samples_per_channel = samples_per_channel as usize;
        let total_samples = channels.checked_mul(samples_per_channel)?;
        let expected_len = total_samples.checked_mul(4)?;
        if data.len() != expected_len {
            return None;
        }

        let mut out = vec![vec![0f32; samples_per_channel]; channels];

        for ch in 0..channels {
            let plane_base = ch * samples_per_channel * 4;
            for sample_idx in 0..samples_per_channel {
                let i = plane_base + sample_idx * 4;
                let bytes = [data[i], data[i + 1], data[i + 2], data[i + 3]];
                out[ch][sample_idx] = f32::from_le_bytes(bytes);
            }
        }

        Some(out)
    }

    pub fn compressed_data(&self) -> Option<&'a [u8]> {
        if self.raw.CompressedData.is_null() || self.raw.CompressedLength <= 0 {
            return None;
        }
        let len = self.raw.CompressedLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.CompressedData as *const u8, len) })
    }

    pub fn metadata(&self) -> Option<&'a [u8]> {
        if self.raw.FrameMetadata.is_null() || self.raw.FrameMetadataLength <= 0 {
            return None;
        }
        let len = self.raw.FrameMetadataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.FrameMetadata as *const u8, len) })
    }
}
