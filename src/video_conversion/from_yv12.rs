//! YV12 video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvPlanarImage, YuvRange, YuvStandardMatrix};

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

pub fn yv12_to_rgb16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
    _yuv_range: YuvRange,
    _yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB16>> {
    None
}

pub fn yv12_to_rgba16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
    _yuv_range: YuvRange,
    _yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA16>> {
    None
}
