//! Video frame format conversion utilities.
//!
//! This module provides internal conversion functions used by the `MediaFrame` type.
//! Note that only conversions which can be natively done using the `yuv` crate are actually
//! available. All other conversions simply return `None`. The reason is that `yuv` is using
//! SIMD/assembler optimized conversions. If you need something special (e.g. conversion from
//! a format w/o alpha to a format *with* alpha) either do it on your own or use functions in
//! this module as foundation and add your adaptations on top thereof. This is also
//! the reason why the return types all return `RGB8`/`RGBA8`/`RGB16`/`RGBA16` instead of `u8`.
//! This allows easier iterating/mapping over the results.
//!
//! To convert video frames, use the methods on `MediaFrame`:
//! - [`MediaFrame::to_rgb8()`](crate::MediaFrame::to_rgb8)
//! - [`MediaFrame::to_rgba8()`](crate::MediaFrame::to_rgba8)
//! - [`MediaFrame::to_rgb16()`](crate::MediaFrame::to_rgb16)
//! - [`MediaFrame::to_rgba16()`](crate::MediaFrame::to_rgba16)
use crate::MediaFrame;
use crate::types::{ColorSpace, VideoFlags};
use yuv::{YuvRange, YuvStandardMatrix};

pub(crate) use from_bgra::*;
pub(crate) use from_nv12::*;
pub(crate) use from_p216::*;
pub(crate) use from_uyva::*;
pub(crate) use from_uyvy::*;
pub(crate) use from_yuy2::*;
pub(crate) use from_yv12::*;

mod from_bgra;
mod from_nv12;
mod from_p216;
mod from_uyva;
mod from_uyvy;
mod from_yuy2;
mod from_yv12;

#[cfg(test)]
mod test_utils;

/// Determines the appropriate YUV standard matrix for a video frame.
///
/// Returns the YUV standard matrix based on the frame's color space:
/// - `Bt709` for BT.709 color space or frames with width >= 1280 (HD and above)
/// - `Bt601` for BT.601 color space or frames with width < 1280 (SD)
pub(crate) fn get_yuv_matrix(frame: &MediaFrame) -> YuvStandardMatrix {
    match frame.color_space() {
        Some(ColorSpace::Bt709) => YuvStandardMatrix::Bt709,
        Some(ColorSpace::Bt601) => YuvStandardMatrix::Bt601,
        Some(ColorSpace::Undefined) | None => {
            if frame.width() >= 1280 {
                YuvStandardMatrix::Bt709
            } else {
                YuvStandardMatrix::Bt601
            }
        }
    }
}

/// Determines the appropriate YUV range for a video frame.
///
/// Returns `Full` range if the frame has high bit depth flag set,
/// otherwise returns `Limited` range.
pub(crate) fn get_yuv_range(frame: &MediaFrame) -> YuvRange {
    if frame.flags().contains(VideoFlags::HIGH_BIT_DEPTH) {
        YuvRange::Full
    } else {
        YuvRange::Limited
    }
}
