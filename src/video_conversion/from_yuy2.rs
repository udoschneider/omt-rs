//! YUY2 video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvPackedImage, YuvRange, YuvStandardMatrix};

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
