use crate::ffi;
use crate::types::{Codec, ColorSpace, VideoFlags};
use crate::video_data_format::VideoDataFormat;

/// Video-specific accessors for a received media frame.
pub struct VideoFrame<'a> {
    raw: &'a ffi::OMTMediaFrame,
}

impl<'a> VideoFrame<'a> {
    pub(crate) fn new(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self { raw }
    }

    pub fn width(&self) -> i32 {
        self.raw.Width as i32
    }

    pub fn height(&self) -> i32 {
        self.raw.Height as i32
    }

    pub fn stride(&self) -> i32 {
        self.raw.Stride as i32
    }

    pub fn frame_rate(&self) -> (i32, i32) {
        (self.raw.FrameRateN as i32, self.raw.FrameRateD as i32)
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

    pub fn raw_data(&self) -> Option<&'a [u8]> {
        if self.raw.Data.is_null() || self.raw.DataLength <= 0 {
            return None;
        }
        let len = self.raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.Data as *const u8, len) })
    }

    pub fn data(&self, format: VideoDataFormat) -> Option<Vec<u8>> {
        let data = self.raw_data()?;
        let width = self.width() as usize;
        let height = self.height() as usize;
        let stride = self.stride() as usize;

        if width == 0 || height == 0 {
            return None;
        }

        let codec = Codec::from(self.raw.Codec);
        let flags = self.flags();
        let (out_has_alpha, out_premultiplied, out_high_bit_depth) = match format {
            VideoDataFormat::RGB => (false, false, false),
            VideoDataFormat::RGBAs => (true, false, false),
            VideoDataFormat::RGBAp => (true, true, false),
            VideoDataFormat::RGB16 => (false, false, true),
            VideoDataFormat::RGBAs16 => (true, false, true),
            VideoDataFormat::RGBAp16 => (true, true, true),
        };

        let out_channels = if out_has_alpha { 4 } else { 3 };
        let out_bytes = if out_high_bit_depth { 2 } else { 1 };
        let pixel_count = width.checked_mul(height)?;
        let out_len = pixel_count
            .checked_mul(out_channels)?
            .checked_mul(out_bytes)?;
        let mut out = vec![0u8; out_len];

        match codec {
            Codec::BGRA => self.data_bgra(
                data,
                width,
                height,
                stride,
                flags,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            Codec::UYVY | Codec::YUY2 => self.data_uyvy_yuy2(
                data,
                width,
                height,
                stride,
                codec,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            Codec::UYVA => self.data_uyva(
                data,
                width,
                height,
                stride,
                flags,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            Codec::NV12 => self.data_nv12(
                data,
                width,
                height,
                stride,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            Codec::YV12 => self.data_yv12(
                data,
                width,
                height,
                stride,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            Codec::P216 | Codec::PA16 => self.data_p216_pa16(
                data,
                width,
                height,
                stride,
                codec,
                flags,
                out_has_alpha,
                out_premultiplied,
                out_high_bit_depth,
                &mut out,
            )?,
            _ => return None,
        }

        Some(out)
    }

    fn data_bgra(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        flags: VideoFlags,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width * 4 || data.len() < stride * height {
            return None;
        }
        for y in 0..height {
            let row = &data[y * stride..y * stride + width * 4];
            for x in 0..width {
                let i = x * 4;
                let b = row[i];
                let g = row[i + 1];
                let r = row[i + 2];
                let mut a = row[i + 3];
                if !flags.contains(VideoFlags::ALPHA) {
                    a = 255;
                }

                if out_high_bit_depth {
                    let (r, g, b, a) = convert_alpha_u16(
                        upscale_u8_to_u16(r),
                        upscale_u8_to_u16(g),
                        upscale_u8_to_u16(b),
                        upscale_u8_to_u16(a),
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r, g, b, a);
                } else {
                    let (r, g, b, a) = convert_alpha_u8(
                        r,
                        g,
                        b,
                        a,
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u8(out, x + y * width, out_has_alpha, r, g, b, a);
                }
            }
        }
        Some(())
    }

    fn data_uyvy_yuy2(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        codec: Codec,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width * 2 || data.len() < stride * height {
            return None;
        }
        for y in 0..height {
            let row = &data[y * stride..y * stride + width * 2];
            let mut x = 0;
            while x + 1 < width {
                let i = x * 2;
                let (u, y0, v, y1) = if codec == Codec::UYVY {
                    (row[i], row[i + 1], row[i + 2], row[i + 3])
                } else {
                    (row[i + 1], row[i], row[i + 3], row[i + 2])
                };

                let (r0, g0, b0) = yuv_to_rgb_u8(y0, u, v, self.color_space());
                let (r1, g1, b1) = yuv_to_rgb_u8(y1, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r0, g0, b0, a0) = convert_alpha_u16(
                        upscale_u8_to_u16(r0),
                        upscale_u8_to_u16(g0),
                        upscale_u8_to_u16(b0),
                        65535,
                        false,
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r0, g0, b0, a0);

                    let (r1, g1, b1, a1) = convert_alpha_u16(
                        upscale_u8_to_u16(r1),
                        upscale_u8_to_u16(g1),
                        upscale_u8_to_u16(b1),
                        65535,
                        false,
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + 1 + y * width, out_has_alpha, r1, g1, b1, a1);
                } else {
                    let (r0, g0, b0, a0) =
                        convert_alpha_u8(r0, g0, b0, 255, false, out_premultiplied);
                    write_pixel_u8(out, x + y * width, out_has_alpha, r0, g0, b0, a0);

                    let (r1, g1, b1, a1) =
                        convert_alpha_u8(r1, g1, b1, 255, false, out_premultiplied);
                    write_pixel_u8(out, x + 1 + y * width, out_has_alpha, r1, g1, b1, a1);
                }

                x += 2;
            }

            if x < width {
                let i = x * 2;
                let (u, y0, v) = if codec == Codec::UYVY {
                    (row[i], row[i + 1], row[i + 2])
                } else {
                    (row[i + 1], row[i], row[i + 3])
                };
                let (r0, g0, b0) = yuv_to_rgb_u8(y0, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r0, g0, b0, a0) = convert_alpha_u16(
                        upscale_u8_to_u16(r0),
                        upscale_u8_to_u16(g0),
                        upscale_u8_to_u16(b0),
                        65535,
                        false,
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r0, g0, b0, a0);
                } else {
                    let (r0, g0, b0, a0) =
                        convert_alpha_u8(r0, g0, b0, 255, false, out_premultiplied);
                    write_pixel_u8(out, x + y * width, out_has_alpha, r0, g0, b0, a0);
                }
            }
        }
        Some(())
    }

    fn data_uyva(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        flags: VideoFlags,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width * 2 {
            return None;
        }
        let yuv_plane_size = stride * height;
        let alpha_stride = width;
        let alpha_plane_size = alpha_stride * height;
        if data.len() < yuv_plane_size + alpha_plane_size {
            return None;
        }
        let (yuv_plane, alpha_plane) = data.split_at(yuv_plane_size);

        for y in 0..height {
            let row = &yuv_plane[y * stride..y * stride + width * 2];
            let alpha_row = &alpha_plane[y * alpha_stride..y * alpha_stride + width];
            let mut x = 0;
            while x + 1 < width {
                let i = x * 2;
                let u = row[i];
                let y0 = row[i + 1];
                let v = row[i + 2];
                let y1 = row[i + 3];

                let a0 = if flags.contains(VideoFlags::ALPHA) {
                    alpha_row[x]
                } else {
                    255
                };
                let a1 = if flags.contains(VideoFlags::ALPHA) {
                    alpha_row[x + 1]
                } else {
                    255
                };

                let (r0, g0, b0) = yuv_to_rgb_u8(y0, u, v, self.color_space());
                let (r1, g1, b1) = yuv_to_rgb_u8(y1, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r0, g0, b0, a0) = convert_alpha_u16(
                        upscale_u8_to_u16(r0),
                        upscale_u8_to_u16(g0),
                        upscale_u8_to_u16(b0),
                        upscale_u8_to_u16(a0),
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r0, g0, b0, a0);

                    let (r1, g1, b1, a1) = convert_alpha_u16(
                        upscale_u8_to_u16(r1),
                        upscale_u8_to_u16(g1),
                        upscale_u8_to_u16(b1),
                        upscale_u8_to_u16(a1),
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + 1 + y * width, out_has_alpha, r1, g1, b1, a1);
                } else {
                    let (r0, g0, b0, a0) = convert_alpha_u8(
                        r0,
                        g0,
                        b0,
                        a0,
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u8(out, x + y * width, out_has_alpha, r0, g0, b0, a0);

                    let (r1, g1, b1, a1) = convert_alpha_u8(
                        r1,
                        g1,
                        b1,
                        a1,
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u8(out, x + 1 + y * width, out_has_alpha, r1, g1, b1, a1);
                }

                x += 2;
            }

            if x < width {
                let i = x * 2;
                let u = row[i];
                let y0 = row[i + 1];
                let v = row[i + 2];

                let a0 = if flags.contains(VideoFlags::ALPHA) {
                    alpha_row[x]
                } else {
                    255
                };
                let (r0, g0, b0) = yuv_to_rgb_u8(y0, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r0, g0, b0, a0) = convert_alpha_u16(
                        upscale_u8_to_u16(r0),
                        upscale_u8_to_u16(g0),
                        upscale_u8_to_u16(b0),
                        upscale_u8_to_u16(a0),
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r0, g0, b0, a0);
                } else {
                    let (r0, g0, b0, a0) = convert_alpha_u8(
                        r0,
                        g0,
                        b0,
                        a0,
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u8(out, x + y * width, out_has_alpha, r0, g0, b0, a0);
                }
            }
        }
        Some(())
    }

    fn data_nv12(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width || data.len() < stride * height {
            return None;
        }
        let y_plane_size = stride * height;
        let uv_plane_size = stride * (height / 2);
        if data.len() < y_plane_size + uv_plane_size {
            return None;
        }
        let (y_plane, uv_plane) = data.split_at(y_plane_size);

        for y in 0..height {
            let y_row = &y_plane[y * stride..y * stride + width];
            let uv_row = &uv_plane[(y / 2) * stride..(y / 2) * stride + width];
            for x in 0..width {
                let y0 = y_row[x];
                let uv_idx = (x / 2) * 2;
                let u = uv_row[uv_idx];
                let v = uv_row[uv_idx + 1];
                let (r, g, b) = yuv_to_rgb_u8(y0, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r, g, b, a) = convert_alpha_u16(
                        upscale_u8_to_u16(r),
                        upscale_u8_to_u16(g),
                        upscale_u8_to_u16(b),
                        65535,
                        false,
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r, g, b, a);
                } else {
                    let (r, g, b, a) = convert_alpha_u8(r, g, b, 255, false, out_premultiplied);
                    write_pixel_u8(out, x + y * width, out_has_alpha, r, g, b, a);
                }
            }
        }
        Some(())
    }

    fn data_yv12(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width || data.len() < stride * height {
            return None;
        }
        let y_plane_size = stride * height;
        let chroma_stride = stride / 2;
        let chroma_height = height / 2;
        let chroma_plane_size = chroma_stride * chroma_height;

        if data.len() < y_plane_size + chroma_plane_size * 2 {
            return None;
        }

        let y_plane = &data[..y_plane_size];
        let v_plane = &data[y_plane_size..y_plane_size + chroma_plane_size];
        let u_plane = &data[y_plane_size + chroma_plane_size..y_plane_size + chroma_plane_size * 2];

        for y in 0..height {
            let y_row = &y_plane[y * stride..y * stride + width];
            let u_row = &u_plane[(y / 2) * chroma_stride..(y / 2) * chroma_stride + (width / 2)];
            let v_row = &v_plane[(y / 2) * chroma_stride..(y / 2) * chroma_stride + (width / 2)];
            for x in 0..width {
                let y0 = y_row[x];
                let u = u_row[x / 2];
                let v = v_row[x / 2];
                let (r, g, b) = yuv_to_rgb_u8(y0, u, v, self.color_space());

                if out_high_bit_depth {
                    let (r, g, b, a) = convert_alpha_u16(
                        upscale_u8_to_u16(r),
                        upscale_u8_to_u16(g),
                        upscale_u8_to_u16(b),
                        65535,
                        false,
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r, g, b, a);
                } else {
                    let (r, g, b, a) = convert_alpha_u8(r, g, b, 255, false, out_premultiplied);
                    write_pixel_u8(out, x + y * width, out_has_alpha, r, g, b, a);
                }
            }
        }
        Some(())
    }

    fn data_p216_pa16(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        codec: Codec,
        flags: VideoFlags,
        out_has_alpha: bool,
        out_premultiplied: bool,
        out_high_bit_depth: bool,
        out: &mut [u8],
    ) -> Option<()> {
        if stride < width * 2 {
            return None;
        }
        let y_plane_size = stride * height;
        let uv_plane_size = stride * height;
        let alpha_plane_size = width * height * 2;

        let needs_alpha = codec == Codec::PA16 && flags.contains(VideoFlags::ALPHA);
        let total = if codec == Codec::PA16 {
            y_plane_size + uv_plane_size + alpha_plane_size
        } else {
            y_plane_size + uv_plane_size
        };

        if data.len() < total {
            return None;
        }

        let (y_plane, rest) = data.split_at(y_plane_size);
        let (uv_plane, alpha_plane) = if codec == Codec::PA16 {
            let (uv, a) = rest.split_at(uv_plane_size);
            (uv, Some(a))
        } else {
            (rest, None)
        };

        for y in 0..height {
            let y_row = &y_plane[y * stride..y * stride + width * 2];
            let uv_row = &uv_plane[y * stride..y * stride + width * 2];
            let alpha_row = if let Some(a) = alpha_plane {
                let alpha_stride = width * 2;
                Some(&a[y * alpha_stride..y * alpha_stride + width * 2])
            } else {
                None
            };

            for x in 0..width {
                let y_idx = x * 2;
                let u_idx = (x / 2) * 4;

                let y0 = u16::from_le_bytes([y_row[y_idx], y_row[y_idx + 1]]);
                let u = u16::from_le_bytes([uv_row[u_idx], uv_row[u_idx + 1]]);
                let v = u16::from_le_bytes([uv_row[u_idx + 2], uv_row[u_idx + 3]]);

                let (r8, g8, b8) = yuv_to_rgb_u8(
                    downscale_u16_to_u8(y0),
                    downscale_u16_to_u8(u),
                    downscale_u16_to_u8(v),
                    self.color_space(),
                );

                let a16 = if needs_alpha {
                    if let Some(a_row) = alpha_row {
                        let a_idx = x * 2;
                        u16::from_le_bytes([a_row[a_idx], a_row[a_idx + 1]])
                    } else {
                        65535
                    }
                } else {
                    65535
                };

                if out_high_bit_depth {
                    let (r, g, b, a) = convert_alpha_u16(
                        upscale_u8_to_u16(r8),
                        upscale_u8_to_u16(g8),
                        upscale_u8_to_u16(b8),
                        a16,
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u16(out, x + y * width, out_has_alpha, r, g, b, a);
                } else {
                    let (r, g, b, a) = convert_alpha_u8(
                        r8,
                        g8,
                        b8,
                        downscale_u16_to_u8(a16),
                        flags.contains(VideoFlags::PREMULTIPLIED),
                        out_premultiplied,
                    );
                    write_pixel_u8(out, x + y * width, out_has_alpha, r, g, b, a);
                }
            }
        }
        Some(())
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

fn write_pixel_u8(out: &mut [u8], index: usize, has_alpha: bool, r: u8, g: u8, b: u8, a: u8) {
    let channels = if has_alpha { 4 } else { 3 };
    let base = index * channels;
    out[base] = r;
    out[base + 1] = g;
    out[base + 2] = b;
    if has_alpha {
        out[base + 3] = a;
    }
}

fn write_pixel_u16(out: &mut [u8], index: usize, has_alpha: bool, r: u16, g: u16, b: u16, a: u16) {
    let channels = if has_alpha { 4 } else { 3 };
    let base = index * channels * 2;
    let rb = r.to_le_bytes();
    let gb = g.to_le_bytes();
    let bb = b.to_le_bytes();
    out[base] = rb[0];
    out[base + 1] = rb[1];
    out[base + 2] = gb[0];
    out[base + 3] = gb[1];
    out[base + 4] = bb[0];
    out[base + 5] = bb[1];
    if has_alpha {
        let ab = a.to_le_bytes();
        out[base + 6] = ab[0];
        out[base + 7] = ab[1];
    }
}

fn convert_alpha_u8(
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    src_premultiplied: bool,
    dst_premultiplied: bool,
) -> (u8, u8, u8, u8) {
    if src_premultiplied == dst_premultiplied {
        return (r, g, b, a);
    }
    if dst_premultiplied {
        if a == 0 {
            return (0, 0, 0, 0);
        }
        let a16 = a as u16;
        let r = ((r as u16 * a16 + 127) / 255) as u8;
        let g = ((g as u16 * a16 + 127) / 255) as u8;
        let b = ((b as u16 * a16 + 127) / 255) as u8;
        (r, g, b, a)
    } else {
        if a == 0 {
            return (0, 0, 0, 0);
        }
        let a16 = a as u16;
        let r = ((r as u16 * 255 + a16 / 2) / a16) as u8;
        let g = ((g as u16 * 255 + a16 / 2) / a16) as u8;
        let b = ((b as u16 * 255 + a16 / 2) / a16) as u8;
        (r, g, b, a)
    }
}

fn convert_alpha_u16(
    r: u16,
    g: u16,
    b: u16,
    a: u16,
    src_premultiplied: bool,
    dst_premultiplied: bool,
) -> (u16, u16, u16, u16) {
    if src_premultiplied == dst_premultiplied {
        return (r, g, b, a);
    }
    if dst_premultiplied {
        if a == 0 {
            return (0, 0, 0, 0);
        }
        let r = ((r as u32 * a as u32 + 32767) / 65535) as u16;
        let g = ((g as u32 * a as u32 + 32767) / 65535) as u16;
        let b = ((b as u32 * a as u32 + 32767) / 65535) as u16;
        (r, g, b, a)
    } else {
        if a == 0 {
            return (0, 0, 0, 0);
        }
        let r = ((r as u32 * 65535 + (a as u32 / 2)) / a as u32) as u16;
        let g = ((g as u32 * 65535 + (a as u32 / 2)) / a as u32) as u16;
        let b = ((b as u32 * 65535 + (a as u32 / 2)) / a as u32) as u16;
        (r, g, b, a)
    }
}

fn yuv_to_rgb_u8(y: u8, u: u8, v: u8, cs: ColorSpace) -> (u8, u8, u8) {
    let c = y as i32 - 16;
    let d = u as i32 - 128;
    let e = v as i32 - 128;

    let (r, g, b) = match cs {
        ColorSpace::BT709 => {
            let r = (298 * c + 459 * e + 128) / 256;
            let g = (298 * c - 55 * d - 136 * e + 128) / 256;
            let b = (298 * c + 541 * d + 128) / 256;
            (r, g, b)
        }
        _ => {
            let r = (298 * c + 409 * e + 128) / 256;
            let g = (298 * c - 100 * d - 208 * e + 128) / 256;
            let b = (298 * c + 516 * d + 128) / 256;
            (r, g, b)
        }
    };

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

fn clamp_u8(v: i32) -> u8 {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v as u8
    }
}

fn upscale_u8_to_u16(v: u8) -> u16 {
    (v as u16) * 257
}

fn downscale_u16_to_u8(v: u16) -> u8 {
    ((v as u32 + 128) / 257) as u8
}
