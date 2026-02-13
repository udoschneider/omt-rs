//! Video-specific methods for MediaFrame.

use crate::frame::MediaFrame;
use crate::types::{Codec, ColorSpace, VideoFlags};
use crate::video_conversion::{
    bgra_to_rgb8, bgra_to_rgba8, get_yuv_matrix, get_yuv_range, nv12_to_rgb8, nv12_to_rgba8,
    p216_to_rgb16, p216_to_rgba16, pa16_to_rgb16, pa16_to_rgba16, uyva_to_rgb8, uyva_to_rgba8,
    uyvy_to_rgb8, uyvy_to_rgba8, yuy2_to_rgb8, yuy2_to_rgba8, yv12_to_rgb8, yv12_to_rgba8,
};
use rgb::{RGB8, RGB16, RGBA8, RGBA16};

impl<'a> MediaFrame<'a> {
    /// Returns the video width in pixels.
    ///
    /// This method is only meaningful for video frames.
    pub fn width(&self) -> i32 {
        self.ffi.Width
    }

    /// Returns the video height in pixels.
    ///
    /// This method is only meaningful for video frames.
    pub fn height(&self) -> i32 {
        self.ffi.Height
    }

    /// Returns the stride (row pitch) in bytes.
    ///
    /// This method is only meaningful for video frames.
    pub fn stride(&self) -> i32 {
        self.ffi.Stride
    }

    /// Returns the video flags.
    ///
    /// This method is only meaningful for video frames.
    pub fn flags(&self) -> VideoFlags {
        VideoFlags::from_ffi(self.ffi.Flags)
    }

    /// Returns the frame rate numerator.
    ///
    /// This method is only meaningful for video frames.
    pub fn frame_rate_numerator(&self) -> i32 {
        self.ffi.FrameRateN
    }

    /// Returns the frame rate denominator.
    ///
    /// This method is only meaningful for video frames.
    pub fn frame_rate_denominator(&self) -> i32 {
        self.ffi.FrameRateD
    }

    /// Returns the frame rate as a floating point value.
    ///
    /// This method is only meaningful for video frames.
    pub fn frame_rate(&self) -> f64 {
        if self.ffi.FrameRateD != 0 {
            self.ffi.FrameRateN as f64 / self.ffi.FrameRateD as f64
        } else {
            0.0
        }
    }

    /// Returns the display aspect ratio.
    ///
    /// This method is only meaningful for video frames.
    pub fn aspect_ratio(&self) -> f32 {
        self.ffi.AspectRatio
    }

    /// Returns the color space.
    ///
    /// This method is only meaningful for video frames.
    pub fn color_space(&self) -> Option<ColorSpace> {
        ColorSpace::from_ffi(self.ffi.ColorSpace)
    }

    /// Converts the video frame to RGB8 format.
    ///
    /// Returns a vector of RGB8 pixels if the conversion is supported for the frame's codec,
    /// or `None` if the codec doesn't support conversion to RGB8.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::MediaFrame;
    /// # fn example(frame: &MediaFrame) {
    /// if let Some(rgb_pixels) = frame.to_rgb8() {
    ///     // Process RGB8 pixels
    /// }
    /// # }
    /// ```
    pub fn to_rgb8(&self) -> Option<Vec<RGB8>> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let stride = self.stride() as usize;

        let raw_data = self.data();

        let yuv_range = get_yuv_range(self);
        let yuv_matrix = get_yuv_matrix(self);

        match self.codec()? {
            Codec::Uyvy => uyvy_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Yuy2 => yuy2_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Nv12 => nv12_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Yv12 => yv12_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Bgra => bgra_to_rgb8(raw_data, width, height, stride),
            Codec::Uyva => uyva_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::P216 | Codec::Pa16 => None,
            Codec::Vmx1 | Codec::Fpa1 => None,
        }
    }

    /// Converts the video frame to RGBA8 format.
    ///
    /// Returns a vector of RGBA8 pixels if the conversion is supported for the frame's codec,
    /// or `None` if the codec doesn't support conversion to RGBA8.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::MediaFrame;
    /// # fn example(frame: &MediaFrame) {
    /// if let Some(rgba_pixels) = frame.to_rgba8() {
    ///     // Process RGBA8 pixels
    /// }
    /// # }
    /// ```
    pub fn to_rgba8(&self) -> Option<Vec<RGBA8>> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let stride = self.stride() as usize;

        let raw_data = self.data();

        let yuv_range = get_yuv_range(self);
        let yuv_matrix = get_yuv_matrix(self);

        match self.codec()? {
            Codec::Uyvy => uyvy_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Yuy2 => yuy2_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Nv12 => nv12_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Yv12 => yv12_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Bgra => bgra_to_rgba8(raw_data, width, height, stride),
            Codec::Uyva => uyva_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::P216 | Codec::Pa16 => None,
            Codec::Vmx1 | Codec::Fpa1 => None,
        }
    }

    /// Converts the video frame to RGB16 format (16-bit per channel).
    ///
    /// Returns a vector of RGB16 pixels if the conversion is supported for the frame's codec,
    /// or `None` if the codec doesn't support conversion to RGB16.
    ///
    /// Currently supports P216 and PA16 codecs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::MediaFrame;
    /// # fn example(frame: &MediaFrame) {
    /// if let Some(rgb16_pixels) = frame.to_rgb16() {
    ///     // Process RGB16 pixels
    /// }
    /// # }
    /// ```
    pub fn to_rgb16(&self) -> Option<Vec<RGB16>> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let stride = self.stride() as usize;

        let raw_data = self.data();

        let yuv_range = get_yuv_range(self);
        let yuv_matrix = get_yuv_matrix(self);

        match self.codec()? {
            Codec::Uyvy | Codec::Yuy2 | Codec::Nv12 | Codec::Yv12 | Codec::Bgra => None,
            Codec::Uyva => None,
            Codec::P216 => p216_to_rgb16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Pa16 => pa16_to_rgb16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Vmx1 | Codec::Fpa1 => None,
        }
    }

    /// Converts the video frame to RGBA16 format (16-bit per channel).
    ///
    /// Returns a vector of RGBA16 pixels if the conversion is supported for the frame's codec,
    /// or `None` if the codec doesn't support conversion to RGBA16.
    ///
    /// Currently supports P216 and PA16 codecs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::MediaFrame;
    /// # fn example(frame: &MediaFrame) {
    /// if let Some(rgba16_pixels) = frame.to_rgba16() {
    ///     // Process RGBA16 pixels
    /// }
    /// # }
    /// ```
    pub fn to_rgba16(&self) -> Option<Vec<RGBA16>> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let stride = self.stride() as usize;

        let raw_data = self.data();

        let yuv_range = get_yuv_range(self);
        let yuv_matrix = get_yuv_matrix(self);

        match self.codec()? {
            Codec::Uyvy | Codec::Yuy2 | Codec::Nv12 | Codec::Yv12 | Codec::Bgra => None,
            Codec::Uyva => None,
            Codec::P216 => p216_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Pa16 => pa16_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Vmx1 | Codec::Fpa1 => None,
        }
    }
}
