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
use rgb::*;
use yuv::{YuvRange, YuvStandardMatrix};

pub use from_bgra::*;
pub use from_nv12::*;
pub use from_p216::*;
pub use from_uyva::*;
pub use from_uyvy::*;
pub use from_yuy2::*;
pub use from_yv12::*;

mod from_bgra;
mod from_nv12;
mod from_p216;
mod from_uyva;
mod from_uyvy;
mod from_yuy2;
mod from_yv12;

#[cfg(test)]
mod test_utils;

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

pub fn to_rgb16(frame: &VideoFrame) -> Option<Vec<RGB16>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    let raw_data = frame.raw_data()?;

    let yuv_range = get_yuv_range(frame);
    let yuv_matrix = get_yuv_matrix(frame);

    match frame.codec() {
        Codec::UYVY | Codec::YUY2 | Codec::NV12 | Codec::YV12 | Codec::BGRA => None,
        Codec::UYVA => None,
        Codec::P216 => p216_to_rgb16(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::PA16 => pa16_to_rgb16(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => None,
    }
}

pub fn to_rgba16(frame: &VideoFrame) -> Option<Vec<RGBA16>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    let raw_data = frame.raw_data()?;

    let yuv_range = get_yuv_range(frame);
    let yuv_matrix = get_yuv_matrix(frame);

    match frame.codec() {
        Codec::UYVY | Codec::YUY2 | Codec::NV12 | Codec::YV12 | Codec::BGRA => None,
        Codec::UYVA => None,
        Codec::P216 => p216_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::PA16 => pa16_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix),
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => None,
    }
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
