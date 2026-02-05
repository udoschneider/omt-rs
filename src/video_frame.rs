use crate::ffi;
use crate::types::{Codec, ColorSpace, VideoFlags};
use crate::video_conversion;
use rgb::bytemuck;

/// Video-specific accessors for a received media frame.
pub struct VideoFrame<'a> {
    raw: &'a ffi::OMTMediaFrame,
}

impl<'a> VideoFrame<'a> {
    pub(crate) fn new(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self { raw }
    }

    pub fn width(&self) -> i32 {
        self.raw.Width
    }

    pub fn height(&self) -> i32 {
        self.raw.Height
    }

    pub fn stride(&self) -> i32 {
        self.raw.Stride
    }

    pub fn frame_rate(&self) -> (i32, i32) {
        (self.raw.FrameRateN, self.raw.FrameRateD)
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.raw.AspectRatio
    }

    pub fn color_space(&self) -> ColorSpace {
        self.raw.ColorSpace.into()
    }

    pub fn flags(&self) -> VideoFlags {
        self.raw.Flags.into()
    }

    pub fn codec(&self) -> Codec {
        Codec::from(self.raw.Codec)
    }

    pub fn raw_data(&self) -> Option<&'a [u8]> {
        if self.raw.Data.is_null() || self.raw.DataLength <= 0 {
            return None;
        }
        let len = self.raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.Data as *const u8, len) })
    }

    /// Converts the video frame to 8-bit RGB format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGB pixel data (3 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// Not all combinations of input codecs and flags have been implemented or tested yet.
    pub fn rgb8_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgb8(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 8-bit RGBA format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGBA pixel data (4 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// Not all combinations of input codecs and flags have been implemented or tested yet.
    pub fn rgba8_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgba8(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 16-bit RGB format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGB16 pixel data (6 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// 16-bit output formats are not currently implemented and will always return `None`.
    pub fn rgb16_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgb16(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 16-bit RGBA format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGBA16 pixel data (8 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// 16-bit output formats are not currently implemented and will always return `None`.
    pub fn rgba16_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgba16(self).map(|data| bytemuck::cast_slice(&data).to_vec())
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
