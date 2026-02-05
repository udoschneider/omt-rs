//! Video frame format conversion utilities.
//!
//! This module provides functions to convert video frames between different pixel formats.
//! Note that not all combinations of input codecs, flags, and output formats have been
//! implemented or tested yet. Some conversions may return `None`, particularly for:
//! - 16-bit output formats (`RGB16`, `RGBA16`)
//! - Certain codec/flag combinations
//! - Alpha channel handling for formats like `UYVA`
//! - 16-bit input formats (`P216`, `PA16`)
//! - Premultiplied alpha (`PREMULTIPLIED` flag) - not currently handled

use crate::types::{Codec, ColorSpace, VideoDataFormat, VideoFlags};
use crate::VideoFrame;
use rgb::bytemuck;
use rgb::*;
use yuv::{
    YuvBiPlanarImage, YuvConversionMode, YuvPackedImage, YuvPlanarImage, YuvRange,
    YuvStandardMatrix,
};

/// Converts a video frame to the specified output format.
///
/// # Arguments
///
/// * `frame` - The video frame to convert
/// * `format` - The desired output format
///
/// # Returns
///
/// Returns `Some(Vec<u8>)` containing the converted pixel data, or `None` if the
/// conversion is not supported.
///
/// # Note
///
/// Not all combinations of input codecs, flags, and output formats have been
/// implemented or tested yet. This function may return `None` for unsupported
/// conversions, particularly for:
/// - 16-bit output formats (`RGB16`, `RGBA16`)
/// - Certain codec/flag combinations
/// - Alpha channel handling for formats like `UYVA`
/// - 16-bit input formats (`P216`, `PA16`)
/// - Premultiplied alpha (`PREMULTIPLIED` flag) - not currently handled
pub fn convert(frame: &VideoFrame, format: VideoDataFormat) -> Option<Vec<u8>> {
    match format {
        VideoDataFormat::RGB => {
            rgb8_data(frame, format).map(|data| bytemuck::cast_slice(&data).to_vec())
        }
        VideoDataFormat::RGBA => {
            rgba8_data(frame, format).map(|data| bytemuck::cast_slice(&data).to_vec())
        }
        VideoDataFormat::RGB16 => {
            rgb16_data(frame, format).map(|data| bytemuck::cast_slice(&data).to_vec())
        }
        VideoDataFormat::RGBA16 => {
            rgba16_data(frame, format).map(|data| bytemuck::cast_slice(&data).to_vec())
        }
    }
}

fn rgb8_data(frame: &VideoFrame, _format: VideoDataFormat) -> Option<Vec<RGB8>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    // Get raw data
    let raw_data = frame.raw_data()?;

    match frame.codec() {
        Codec::UYVY => {
            // UYVY is packed YUV 4:2:2
            let yuy_stride = stride as u32;

            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride, // UYVY stride is in bytes
                width: width as u32,
                height: height as u32,
            };

            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel

            // Convert UYVY to RGB
            yuv::uyvy422_to_rgb(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgb_data),
                rgb_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgb_data)
        }
        Codec::YUY2 => {
            // YUY2 is packed YUV 4:2:2 (YUVY order)
            let yuy_stride = stride as u32;

            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride, // YUY2 stride is in bytes
                width: width as u32,
                height: height as u32,
            };

            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel

            // Convert YUYV to RGB (YUY2 is YUYV format)
            yuv::yuyv422_to_rgb(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgb_data),
                rgb_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgb_data)
        }
        Codec::NV12 => {
            // NV12 is bi-planar YUV 4:2:0
            let y_plane = &raw_data[0..height * stride];
            let uv_plane = &raw_data[height * stride..];

            let bi_planar_image = YuvBiPlanarImage {
                y_plane,
                y_stride: stride as u32,
                uv_plane,
                uv_stride: stride as u32, // UV plane has same stride as Y plane
                width: width as u32,
                height: height as u32,
            };

            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel

            // Convert NV12 to RGB
            yuv::yuv_nv12_to_rgb(
                &bi_planar_image,
                bytemuck::cast_slice_mut(&mut rgb_data),
                rgb_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
                YuvConversionMode::Balanced,
            )
            .ok()?;

            Some(rgb_data)
        }
        Codec::YV12 => {
            // YV12 is planar YUV 4:2:0 with Y, V, U planes
            let y_size = height * stride;
            let uv_size = (height / 2) * (stride / 2);

            let y_plane = &raw_data[0..y_size];
            let v_plane = &raw_data[y_size..y_size + uv_size];
            let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

            let _planar_image = YuvPlanarImage {
                y_plane,
                y_stride: stride as u32,
                u_plane,
                u_stride: (stride / 2) as u32,
                v_plane,
                v_stride: (stride / 2) as u32,
                width: width as u32,
                height: height as u32,
            };

            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel

            // Convert YV12 to RGB (YV12 is YVU order, but yuv420_to_rgb expects YUV)
            // We need to swap U and V planes
            let swapped_image = YuvPlanarImage {
                y_plane,
                y_stride: stride as u32,
                u_plane: v_plane, // Swap U and V
                u_stride: (stride / 2) as u32,
                v_plane: u_plane, // Swap V and U
                v_stride: (stride / 2) as u32,
                width: width as u32,
                height: height as u32,
            };

            yuv::yuv420_to_rgb(
                &swapped_image,
                bytemuck::cast_slice_mut(&mut rgb_data),
                rgb_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgb_data)
        }
        Codec::BGRA => {
            // BGRA is already in RGB format, just need to convert BGRA to RGB
            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];

            // Convert BGRA to RGB
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel
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
        Codec::UYVA => {
            // UYVA is packed YUV 4:2:2 with alpha
            // For RGB8, we ignore alpha
            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride: stride as u32, // UYVA stride is in bytes (ignoring alpha for now)
                width: width as u32,
                height: height as u32,
            };

            let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel

            // Convert UYVY to RGB (ignoring alpha channel)
            yuv::uyvy422_to_rgb(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgb_data),
                rgb_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgb_data)
        }
        Codec::P216 | Codec::PA16 => {
            // P216 and PA16 are 16-bit formats, not supported for RGB8
            None
        }
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => {
            // Unknown or unsupported codecs
            None
        }
    }
}

fn rgba8_data(frame: &VideoFrame, _format: VideoDataFormat) -> Option<Vec<RGBA8>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride() as usize;

    // Get raw data
    let raw_data = frame.raw_data()?;

    match frame.codec() {
        Codec::UYVY => {
            // UYVY is packed YUV 4:2:2
            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride: stride as u32, // UYVY stride is in bytes
                width: width as u32,
                height: height as u32,
            };

            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel

            // Convert UYVY to RGBA
            yuv::uyvy422_to_rgba(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgba_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::YUY2 => {
            // YUY2 is packed YUV 4:2:2 (YUVY order)
            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride: stride as u32, // YUY2 stride is in bytes
                width: width as u32,
                height: height as u32,
            };

            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel

            // Convert YUYV to RGBA (YUY2 is YUYV format)
            yuv::yuyv422_to_rgba(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgba_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::NV12 => {
            // NV12 is bi-planar YUV 4:2:0
            let y_plane = &raw_data[0..height * stride];
            let uv_plane = &raw_data[height * stride..];

            let bi_planar_image = YuvBiPlanarImage {
                y_plane,
                y_stride: stride as u32,
                uv_plane,
                uv_stride: stride as u32, // UV plane has same stride as Y plane
                width: width as u32,
                height: height as u32,
            };

            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel

            // Convert NV12 to RGBA
            yuv::yuv_nv12_to_rgba(
                &bi_planar_image,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgba_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
                YuvConversionMode::Balanced,
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::YV12 => {
            // YV12 is planar YUV 4:2:0 with Y, V, U planes
            let y_size = height * stride;
            let uv_size = (height / 2) * (stride / 2);

            let y_plane = &raw_data[0..y_size];
            let v_plane = &raw_data[y_size..y_size + uv_size];
            let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

            let _planar_image = YuvPlanarImage {
                y_plane,
                y_stride: stride as u32,
                u_plane,
                u_stride: (stride / 2) as u32,
                v_plane,
                v_stride: (stride / 2) as u32,
                width: width as u32,
                height: height as u32,
            };

            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel

            // Convert YV12 to RGBA (YV12 is YVU order, but yuv420_to_rgba expects YUV)
            // We need to swap U and V planes
            let swapped_image = YuvPlanarImage {
                y_plane,
                y_stride: stride as u32,
                u_plane: v_plane, // Swap U and V
                u_stride: (stride / 2) as u32,
                v_plane: u_plane, // Swap V and U
                v_stride: (stride / 2) as u32,
                width: width as u32,
                height: height as u32,
            };

            yuv::yuv420_to_rgba(
                &swapped_image,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgba_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::BGRA => {
            // BGRA is already in RGBA format, just need to convert BGRA to RGBA
            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];

            // Convert BGRA to RGBA
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel
            let rgb_stride = (width * 3) as u32; // RGB has 3 bytes per pixel
            yuv::bgra_to_rgba(
                raw_data,
                rgba_stride,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgb_stride,
                width as u32,
                height as u32,
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::UYVA => {
            // UYVA is packed YUV 4:2:2 with alpha
            // For RGBA8, we need to handle alpha
            // Note: UYVA format needs special handling, for now treat as UYVY with alpha=255
            let packed_image = YuvPackedImage {
                yuy: raw_data,
                yuy_stride: stride as u32, // UYVA stride is in bytes (ignoring alpha for now)
                width: width as u32,
                height: height as u32,
            };

            let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
            let rgba_stride = (width * 4) as u32; // RGBA has 4 bytes per pixel

            // Convert UYVY to RGBA (ignoring alpha channel for now)
            yuv::uyvy422_to_rgba(
                &packed_image,
                bytemuck::cast_slice_mut(&mut rgba_data),
                rgba_stride,
                get_yuv_range(frame),
                get_yuv_matrix(frame),
            )
            .ok()?;

            Some(rgba_data)
        }
        Codec::P216 | Codec::PA16 => {
            // P216 and PA16 are 16-bit formats, not supported for RGBA8
            None
        }
        Codec::VMX1 | Codec::FPA1 | Codec::Unknown(_) => {
            // Unknown or unsupported codecs
            None
        }
    }
}

#[allow(dead_code)] // TODO: Implement RGB16 conversion
fn rgb16_data(_frame: &VideoFrame, _format: VideoDataFormat) -> Option<Vec<RGB16>> {
    // 16-bit conversion requires different yuv crate functions
    // For now, return None as this requires more complex handling
    None
}

#[allow(dead_code)] // TODO: Implement RGBA16 conversion
fn rgba16_data(_frame: &VideoFrame, _format: VideoDataFormat) -> Option<Vec<RGBA16>> {
    // 16-bit conversion requires different yuv crate functions
    // For now, return None as this requires more complex handling
    None
}

/// Helper function to get YUV matrix from frame color space
fn get_yuv_matrix(frame: &VideoFrame) -> YuvStandardMatrix {
    match frame.color_space() {
        ColorSpace::BT601 => YuvStandardMatrix::Bt601,
        ColorSpace::BT709 => YuvStandardMatrix::Bt709,
        ColorSpace::Undefined => {
            // Default to BT.601 for SD, BT.709 for HD
            if frame.width() >= 1280 {
                YuvStandardMatrix::Bt709
            } else {
                YuvStandardMatrix::Bt601
            }
        }
    }
}

/// Helper function to get YUV range from frame flags
fn get_yuv_range(frame: &VideoFrame) -> YuvRange {
    // Check if frame has high bit depth flag
    if frame.flags().contains(VideoFlags::HIGH_BIT_DEPTH) {
        YuvRange::Full
    } else {
        // Default to limited range for 8-bit video
        YuvRange::Limited
    }
}
