//! NV12 video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvBiPlanarImage, YuvConversionMode, YuvRange, YuvStandardMatrix};

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

pub fn nv12_to_rgb16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
    _yuv_range: YuvRange,
    _yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB16>> {
    None
}

pub fn nv12_to_rgba16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
    _yuv_range: YuvRange,
    _yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA16>> {
    None
}
