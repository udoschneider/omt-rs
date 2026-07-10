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
use crate::types::{Codec, ColorSpace, VideoFlags};
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
pub(crate) fn get_yuv_matrix(frame: &MediaFrame<'_>) -> YuvStandardMatrix {
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
pub(crate) fn get_yuv_range(frame: &MediaFrame<'_>) -> YuvRange {
    if frame.flags().contains(VideoFlags::HIGH_BIT_DEPTH) {
        YuvRange::Full
    } else {
        YuvRange::Limited
    }
}

/// Minimum number of input bytes required to decode a video frame of `codec`
/// with the given dimensions and row stride.
///
/// The `width`, `height` and `stride` values come straight off the network and
/// are attacker-controlled, so every product is computed with checked
/// arithmetic. Returns `None` when a dimension is degenerate (zero), when the
/// computation would overflow `usize`, or when the codec is not decodable to
/// RGB (`Vmx1`/`Fpa1`). Callers must treat `None` as "reject this frame".
///
/// The returned length is a conservative lower bound: it is always at least as
/// large as the highest byte index the corresponding converter in this module
/// will slice, so validating against it before dispatching guarantees the
/// converters can neither index out of bounds nor over-allocate their output.
pub(crate) fn required_input_len(
    codec: Codec,
    width: usize,
    height: usize,
    stride: usize,
) -> Option<usize> {
    if width == 0 || height == 0 || stride == 0 {
        return None;
    }

    match codec {
        // Packed 4:2:2 and packed 32bpp: a single plane of `height` rows.
        Codec::Uyvy | Codec::Yuy2 | Codec::Bgra => height.checked_mul(stride),
        // NV12: full-res Y plane + interleaved UV plane of ceil(height/2) rows.
        Codec::Nv12 => {
            let y = height.checked_mul(stride)?;
            let uv = stride.checked_mul(height.div_ceil(2))?;
            y.checked_add(uv)
        }
        // YV12: full-res Y plane + two quarter-res chroma planes.
        Codec::Yv12 => {
            let y = height.checked_mul(stride)?;
            let uv = (height / 2).checked_mul(stride / 2)?.checked_mul(2)?;
            y.checked_add(uv)
        }
        // UYVA: packed UYVY plane followed by a full-res 8-bit alpha plane.
        Codec::Uyva => {
            let uyvy = height.checked_mul(stride)?;
            let alpha = width.checked_mul(height)?;
            uyvy.checked_add(alpha)
        }
        // P216/PA16: 16-bit planar 4:2:2, optionally with a 16-bit alpha plane.
        Codec::P216 => p216_input_len(width, height, stride, false),
        Codec::Pa16 => p216_input_len(width, height, stride, true),
        Codec::Vmx1 | Codec::Fpa1 => None,
    }
}

/// Byte length required by the P216/PA16 layout (see [`required_input_len`]).
fn p216_input_len(width: usize, height: usize, stride: usize, alpha: bool) -> Option<usize> {
    // Everything below is counted in 16-bit (u16) elements, then doubled.
    let y = (stride / 2).checked_mul(height)?;
    // Interleaved UV: ceil(width/2) pairs per row, two u16 per pair, `height` rows.
    let uv = width.div_ceil(2).checked_mul(2)?.checked_mul(height)?;
    let mut elements = y.checked_add(uv)?;
    if alpha {
        elements = elements.checked_add(width.checked_mul(height)?)?;
    }
    elements.checked_mul(2)
}

#[cfg(test)]
mod required_input_len_tests {
    use super::*;

    #[test]
    fn rejects_degenerate_dimensions() {
        assert_eq!(required_input_len(Codec::Uyvy, 0, 1080, 3840), None);
        assert_eq!(required_input_len(Codec::Uyvy, 1920, 0, 3840), None);
        assert_eq!(required_input_len(Codec::Uyvy, 1920, 1080, 0), None);
    }

    #[test]
    fn rejects_overflowing_dimensions() {
        // Mirrors a negative i32 dimension cast to usize by the caller.
        let huge = usize::MAX;
        assert_eq!(required_input_len(Codec::Uyvy, huge, huge, huge), None);
        assert_eq!(required_input_len(Codec::Nv12, 4, huge, huge), None);
    }

    #[test]
    fn matches_known_layouts() {
        // Packed UYVY: height * stride.
        assert_eq!(required_input_len(Codec::Uyvy, 4, 2, 8), Some(16));
        // NV12: y (2*8) + uv (8 * ceil(2/2)) = 16 + 8.
        assert_eq!(required_input_len(Codec::Nv12, 4, 2, 8), Some(24));
        // YV12: y (2*8) + 2 * ((2/2) * (8/2)) = 16 + 8.
        assert_eq!(required_input_len(Codec::Yv12, 4, 2, 8), Some(24));
        // UYVA: uyvy (2*8) + alpha (4*2) = 16 + 8.
        assert_eq!(required_input_len(Codec::Uyva, 4, 2, 8), Some(24));
    }

    #[test]
    fn compressed_and_audio_codecs_are_rejected() {
        assert_eq!(required_input_len(Codec::Vmx1, 4, 2, 8), None);
        assert_eq!(required_input_len(Codec::Fpa1, 4, 2, 8), None);
    }
}
