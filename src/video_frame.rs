use crate::ffi;
use crate::types::{Codec, ColorSpace, VideoDataFormat, VideoFlags};
use crate::video_conversion;

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

    /// Converts the video frame to the specified output format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted pixel data, or `None` if the
    /// conversion is not supported.
    ///
    /// # Note
    ///
    /// Not all combinations of input codecs, flags, and output formats have been
    /// implemented or tested yet. This method may return `None` for unsupported
    /// conversions, particularly for:
    /// - 16-bit output formats (`RGB16`, `RGBA16`)
    /// - Certain codec/flag combinations
    /// - Alpha channel handling for formats like `UYVA`
    /// - Premultiplied alpha (`PREMULTIPLIED` flag) - not currently handled
    pub fn data(&self, format: VideoDataFormat) -> Option<Vec<u8>> {
        video_conversion::convert(self, format)
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
