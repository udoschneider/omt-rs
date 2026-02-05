//! BGRA video frame conversion functions.

use rgb::bytemuck;
use rgb::*;

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

pub fn bgra_to_rgb16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
) -> Option<Vec<RGB16>> {
    None
}

pub fn bgra_to_rgba16(
    _raw_data: &[u8],
    _width: usize,
    _height: usize,
    _stride: usize,
) -> Option<Vec<RGBA16>> {
    None
}
