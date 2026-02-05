//! Video frame format conversion utilities.
//!
//! This module provides functions to convert video frames between different pixel formats.
//! Note that only conversions which can be natively done using the `yuv` crate are actually
//! available. All other conversions simply return `None`. The reason is that `yuv` is using
//! SIMD/assembler optimized conversions. If you need something special (e.g. conversion from
//! a format w/o alpha to a format *with* alpha) either do it on your own or use functions in
//! `video_conversions.rs` as foundation and add your adaptations on top thereof. This is also
//! the reason why the return types all return `RGB8`/`RGBA8`/`RGB16`/`RGBA16` instead of `u8`.
//! This allows easier iterating/mapping over the results.

use crate::types::{Codec, ColorSpace, VideoFlags};
use crate::VideoFrame;
use rgb::bytemuck;
use rgb::*;
use yuv::{
    YuvBiPlanarImage, YuvConversionMode, YuvPackedImage, YuvPlanarImage, YuvRange,
    YuvStandardMatrix,
};

pub fn to_rgb8(frame: &VideoFrame) -> Option<Vec<RGB8>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    let raw_data = frame.raw_data()?;

    let yuv_range = get_yuv_range(frame);
    let yuv_matrix = get_yuv_matrix(frame);

    match frame.codec() {
        Codec::UYVY => uyvy_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::YUY2 => yuy2_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::NV12 => nv12_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::YV12 => yv12_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::BGRA => bgra_to_rgb8(raw_data, width, height, stride),
        Codec::UYVA => uyva_to_rgb8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::P216 | Codec::PA16 => None,
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => None,
    }
}

pub fn to_rgba8(frame: &VideoFrame) -> Option<Vec<RGBA8>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    let raw_data = frame.raw_data()?;

    let yuv_range = get_yuv_range(frame);
    let yuv_matrix = get_yuv_matrix(frame);

    match frame.codec() {
        Codec::UYVY => uyvy_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::YUY2 => yuy2_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::NV12 => nv12_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::YV12 => yv12_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::BGRA => bgra_to_rgba8(raw_data, width, height, stride),
        Codec::UYVA => uyva_to_rgba8(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::P216 | Codec::PA16 => None,
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => None,
    }
}

pub fn to_rgb16(_frame: &VideoFrame) -> Option<Vec<RGB16>> {
    None
}

pub fn to_rgba16(_frame: &VideoFrame) -> Option<Vec<RGBA16>> {
    None
}

pub fn uyvy_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let yuy_stride = stride as u32;

    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::uyvy422_to_rgb(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn yuy2_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let yuy_stride = stride as u32;

    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::yuyv422_to_rgb(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn nv12_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let y_plane = &raw_data[0..height * stride];
    let uv_plane = &raw_data[height * stride..];

    let bi_planar_image = YuvBiPlanarImage {
        y_plane,
        y_stride: stride as u32,
        uv_plane,
        uv_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::yuv_nv12_to_rgb(
        &bi_planar_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
        YuvConversionMode::Balanced,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn yv12_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let y_size = height * stride;
    let uv_size = (height / 2) * (stride / 2);

    let y_plane = &raw_data[0..y_size];
    let v_plane = &raw_data[y_size..y_size + uv_size];
    let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    let swapped_image = YuvPlanarImage {
        y_plane,
        y_stride: stride as u32,
        u_plane: v_plane,
        u_stride: (stride / 2) as u32,
        v_plane: u_plane,
        v_stride: (stride / 2) as u32,
        width: width as u32,
        height: height as u32,
    };

    yuv::yuv420_to_rgb(
        &swapped_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn bgra_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    _stride: usize,
) -> Option<Vec<RGB8>> {
    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];

    let rgba_stride = (width * 4) as u32;
    let rgb_stride = (width * 3) as u32;
    yuv::bgra_to_rgb(
        raw_data,
        rgba_stride,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        width as u32,
        height as u32,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn uyva_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::uyvy422_to_rgb(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn uyvy_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::uyvy422_to_rgba(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn yuy2_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::yuyv422_to_rgba(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn nv12_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let y_plane = &raw_data[0..height * stride];
    let uv_plane = &raw_data[height * stride..];

    let bi_planar_image = YuvBiPlanarImage {
        y_plane,
        y_stride: stride as u32,
        uv_plane,
        uv_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::yuv_nv12_to_rgba(
        &bi_planar_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
        YuvConversionMode::Balanced,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn yv12_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let y_size = height * stride;
    let uv_size = (height / 2) * (stride / 2);

    let y_plane = &raw_data[0..y_size];
    let v_plane = &raw_data[y_size..y_size + uv_size];
    let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    let swapped_image = YuvPlanarImage {
        y_plane,
        y_stride: stride as u32,
        u_plane: v_plane,
        u_stride: (stride / 2) as u32,
        v_plane: u_plane,
        v_stride: (stride / 2) as u32,
        width: width as u32,
        height: height as u32,
    };

    yuv::yuv420_to_rgba(
        &swapped_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn bgra_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    _stride: usize,
) -> Option<Vec<RGBA8>> {
    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];

    let bgra_stride = (width * 4) as u32;
    let rgba_stride = (width * 4) as u32;
    yuv::bgra_to_rgba(
        raw_data,
        bgra_stride,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        width as u32,
        height as u32,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn uyva_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::uyvy422_to_rgba(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgba_data)
}

pub fn get_yuv_matrix(frame: &VideoFrame) -> YuvStandardMatrix {
    match frame.color_space() {
        ColorSpace::BT709 => YuvStandardMatrix::Bt709,
        ColorSpace::BT601 => YuvStandardMatrix::Bt601,
        ColorSpace::Undefined => {
            if frame.width() >= 1280 {
                YuvStandardMatrix::Bt709
            } else {
                YuvStandardMatrix::Bt601
            }
        }
    }
}

pub fn get_yuv_range(frame: &VideoFrame) -> YuvRange {
    if frame.flags().contains(VideoFlags::HIGH_BIT_DEPTH) {
        YuvRange::Full
    } else {
        YuvRange::Limited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const K8: RGB8 = RGB8 { r: 0, g: 0, b: 0 };
    const R8: RGB8 = RGB8 { r: 255, g: 0, b: 0 };
    const G8: RGB8 = RGB8 { r: 0, g: 255, b: 0 };
    const Y8: RGB8 = RGB8 {
        r: 255,
        g: 255,
        b: 0,
    };
    const B8: RGB8 = RGB8 { r: 0, g: 0, b: 255 };
    const M8: RGB8 = RGB8 {
        r: 255,
        g: 0,
        b: 255,
    };
    const C8: RGB8 = RGB8 {
        r: 0,
        g: 255,
        b: 255,
    };
    const W8: RGB8 = RGB8 {
        r: 255,
        g: 255,
        b: 255,
    };

    fn rgb8_colors() -> Vec<RGB8> {
        vec![K8, R8, G8, Y8, B8, M8, C8, W8]
    }

    fn rgba8_colors() -> Vec<RGBA8> {
        return rgb8_colors()
            .iter()
            .enumerate()
            .map(|(index, color)| color.with_alpha((index * 32) as u8))
            .collect();
    }

    fn rgb16_colors() -> Vec<RGB16> {
        return rgb8_colors()
            .iter()
            .map(|color| RGB16 {
                r: color.r as u16 * 257,
                g: color.g as u16 * 257,
                b: color.b as u16 * 257,
            })
            .collect();
    }

    fn rgba16_colors() -> Vec<RGBA16> {
        return rgb16_colors()
            .iter()
            .enumerate()
            .map(|(index, color)| color.with_alpha((index * 8192) as u16))
            .collect();
    }

    #[test]
    fn test_bgr_to_rgb8() {
        let colors: Vec<Bgra<u8>> = rgba8_colors()
            .iter()
            .map(|rgba| Bgra {
                r: rgba.r,
                g: rgba.g,
                b: rgba.b,
                a: rgba.a,
            })
            .collect();
        let bgra_bytes = rgb::bytemuck::cast_slice(&colors[..]);
        assert_eq!(
            rgb8_colors(),
            bgra_to_rgb8(&bgra_bytes[..], 4, 2, 4).unwrap()
        )
    }
}
