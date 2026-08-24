//! Video-specific methods for MediaFrame.

use crate::frame::MediaFrame;
use crate::types::{Codec, ColorSpace, FrameRate, VideoFlags};
use crate::video_conversion::{
    bgra_to_rgb8, bgra_to_rgba8, get_yuv_matrix, get_yuv_range, nv12_to_rgb8, nv12_to_rgba8,
    p216_to_rgb16, p216_to_rgba16, pa16_to_rgb16, pa16_to_rgba16, required_input_len, uyva_to_rgb8,
    uyva_to_rgba8, uyvy_to_rgb8, uyvy_to_rgba8, yuy2_to_rgb8, yuy2_to_rgba8, yv12_to_rgb8,
    yv12_to_rgba8,
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

    /// Returns the frame rate as a rational [`FrameRate`], if valid.
    ///
    /// Returns `None` if the frame's numerator or denominator are not positive
    /// (e.g. for audio or metadata frames, or frames with an unset frame rate).
    ///
    /// This method is only meaningful for video frames.
    pub fn frame_rate_rational(&self) -> Option<FrameRate> {
        FrameRate::new(self.ffi.FrameRateN, self.ffi.FrameRateD).ok()
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

        let codec = self.codec()?;

        // Reject frames whose declared dimensions do not fit the data buffer.
        // `width`/`height`/`stride` are attacker-controlled, so this guard (with
        // its overflow-checked arithmetic) is what keeps the converters below
        // from indexing out of bounds or over-allocating on a malformed frame.
        if raw_data.len() < required_input_len(codec, width, height, stride)? {
            return None;
        }

        match codec {
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

        let codec = self.codec()?;

        // Reject frames whose declared dimensions do not fit the data buffer.
        // `width`/`height`/`stride` are attacker-controlled, so this guard (with
        // its overflow-checked arithmetic) is what keeps the converters below
        // from indexing out of bounds or over-allocating on a malformed frame.
        if raw_data.len() < required_input_len(codec, width, height, stride)? {
            return None;
        }

        match codec {
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

        let codec = self.codec()?;

        // Reject frames whose declared dimensions do not fit the data buffer.
        // `width`/`height`/`stride` are attacker-controlled, so this guard (with
        // its overflow-checked arithmetic) is what keeps the converters below
        // from indexing out of bounds or over-allocating on a malformed frame.
        if raw_data.len() < required_input_len(codec, width, height, stride)? {
            return None;
        }

        match codec {
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

        let codec = self.codec()?;

        // Reject frames whose declared dimensions do not fit the data buffer.
        // `width`/`height`/`stride` are attacker-controlled, so this guard (with
        // its overflow-checked arithmetic) is what keeps the converters below
        // from indexing out of bounds or over-allocating on a malformed frame.
        if raw_data.len() < required_input_len(codec, width, height, stride)? {
            return None;
        }

        match codec {
            Codec::Uyvy | Codec::Yuy2 | Codec::Nv12 | Codec::Yv12 | Codec::Bgra => None,
            Codec::Uyva => None,
            Codec::P216 => p216_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Pa16 => pa16_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
            Codec::Vmx1 | Codec::Fpa1 => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frame_builder::VideoFrameBuilder;
    use crate::types::Codec;

    /// A generously sized neutral-gray buffer, so these tests exercise the
    /// conversion path rather than the builder's minimum-size validation.
    fn frame_data(height: i32, stride: i32) -> Vec<u8> {
        vec![128u8; (stride * height) as usize * 4]
    }

    // Regression: the two 4:2:0 codecs must agree on odd dimensions. YV12's
    // input gate floored both chroma dimensions while NV12's rounded the height
    // up, so `yv12_to_rgb8` sliced its chroma planes one row short and the
    // `yuv` crate rejected the mismatch — an odd-height YV12 frame silently
    // converted to `None` while the equivalent NV12 frame converted fine.
    #[test]
    fn odd_height_420_frames_convert() {
        const WIDTH: i32 = 4;
        const HEIGHT: i32 = 3;
        const STRIDE: i32 = 4;

        for codec in [Codec::Nv12, Codec::Yv12] {
            let owned = VideoFrameBuilder::new()
                .codec(codec)
                .dimensions(WIDTH, HEIGHT)
                .stride(STRIDE)
                .data(frame_data(HEIGHT, STRIDE))
                .build()
                .expect("valid odd-height 4:2:0 frame");
            let frame = owned.as_media_frame();

            let pixels = frame
                .to_rgb8()
                .unwrap_or_else(|| panic!("{codec} odd-height frame failed to convert"));
            assert_eq!(pixels.len(), (WIDTH * HEIGHT) as usize, "{codec}");
        }
    }

    // Regression: a padded row pitch must still convert. The `yuv` crate's
    // packed 4:2:2 check requires an exactly-dense plane and ignores the stride
    // it is handed, so passing a `height * stride` slice straight through made
    // every padded UYVY/YUY2/UYVA frame fail — silently, as a bare `None`, in
    // the crate's own default receive format.
    #[test]
    fn padded_stride_packed_422_frames_convert() {
        const WIDTH: i32 = 8;
        const HEIGHT: i32 = 4;
        const DENSE: i32 = WIDTH * 2;

        for codec in [Codec::Uyvy, Codec::Yuy2, Codec::Uyva] {
            for stride in [DENSE, DENSE + 4, DENSE + 32] {
                let owned = VideoFrameBuilder::new()
                    .codec(codec)
                    .dimensions(WIDTH, HEIGHT)
                    .stride(stride)
                    .data(frame_data(HEIGHT, stride))
                    .build()
                    .expect("valid packed 4:2:2 frame");
                let frame = owned.as_media_frame();

                let pixels = frame.to_rgb8().unwrap_or_else(|| {
                    panic!("{codec} at stride {stride} (dense is {DENSE}) failed to convert")
                });
                assert_eq!(
                    pixels.len(),
                    (WIDTH * HEIGHT) as usize,
                    "{codec} @ {stride}"
                );
                assert!(frame.to_rgba8().is_some(), "{codec} @ {stride} rgba8");
            }
        }
    }

    // The even-dimension path — the only one conforming senders produce — must
    // keep working unchanged after the rounding fix.
    #[test]
    fn even_dimension_420_frames_convert() {
        const WIDTH: i32 = 4;
        const HEIGHT: i32 = 4;
        const STRIDE: i32 = 4;

        for codec in [Codec::Nv12, Codec::Yv12] {
            let owned = VideoFrameBuilder::new()
                .codec(codec)
                .dimensions(WIDTH, HEIGHT)
                .stride(STRIDE)
                .data(frame_data(HEIGHT, STRIDE))
                .build()
                .expect("valid 4:2:0 frame");
            let frame = owned.as_media_frame();

            let pixels = frame
                .to_rgb8()
                .unwrap_or_else(|| panic!("{codec} frame failed to convert"));
            assert_eq!(pixels.len(), (WIDTH * HEIGHT) as usize, "{codec}");
        }
    }
}
