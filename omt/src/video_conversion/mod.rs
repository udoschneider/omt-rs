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
//!
//! # Input validation contract (read before adding a caller)
//!
//! The per-format converters here (`*_to_rgb8`, `*_to_rgba16`, …) **trust**
//! their `width`/`height`/`stride` arguments: they compute plane sizes and
//! slice offsets with plain (unchecked) arithmetic and assume the input slice
//! is large enough. On their own they are therefore *not* safe against
//! unvalidated, attacker-controlled dimensions — a hostile frame could provoke
//! an out-of-bounds slice or an overflowing allocation size.
//!
//! The single enforcement point is [`required_input_len`], which computes the
//! minimum buffer size with fully overflow-checked arithmetic. Every public
//! entry point — the four `MediaFrame::to_*` methods — calls it and rejects the
//! frame before dispatching, so *reaching a converter always implies the frame
//! passed that gate*. Those four methods are currently the only callers.
//!
//! **If you add another caller, it MUST run the same `required_input_len` check
//! first.** Do not call a converter directly on frame-supplied dimensions.
use crate::MediaFrame;
use crate::types::{Codec, ColorSpace};
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
/// When the frame carries an explicit color space, it is honored. For
/// `Undefined`/absent color space we mirror the OMT codec's own default,
/// documented in `libomt.h`: *"BT601 for heights < 720, BT709 for everything
/// else"* — i.e. the selection is **height**-based, not width-based. Matching
/// the library keeps the Rust-side RGB conversion in agreement with the matrix
/// the sender encoded with.
pub(crate) fn get_yuv_matrix(frame: &MediaFrame<'_>) -> YuvStandardMatrix {
    match frame.color_space() {
        Some(ColorSpace::Bt709) => YuvStandardMatrix::Bt709,
        Some(ColorSpace::Bt601) => YuvStandardMatrix::Bt601,
        Some(ColorSpace::Undefined) | None => {
            if frame.height() >= 720 {
                YuvStandardMatrix::Bt709
            } else {
                YuvStandardMatrix::Bt601
            }
        }
    }
}

/// Determines the appropriate YUV range for a video frame.
///
/// OMT transports studio/limited-range YUV for every pixel format and has **no**
/// full-range signaling in its ABI (`libomt.h` exposes only a color-space enum,
/// no range flag). In particular the `HIGH_BIT_DEPTH` flag denotes P216/PA16
/// encoding — a bit-depth property that is orthogonal to range and must not be
/// used to infer it. We therefore always decode as [`YuvRange::Limited`]; the
/// argument is kept for symmetry with [`get_yuv_matrix`] and to localize the
/// assumption should OMT ever gain range signaling.
pub(crate) fn get_yuv_range(_frame: &MediaFrame<'_>) -> YuvRange {
    YuvRange::Limited
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

    // Every packed/planar branch first requires `stride` to be at least the
    // pixel width times the format's per-pixel byte count. `stride` and `width`
    // are independent attacker-controlled values, and the converters allocate
    // their output from `width`; without this the gate could accept a frame with
    // a huge `width` but a tiny `stride` (small validated input) and the
    // converter would over-allocate its `width`-sized output. Requiring
    // `stride >= width * bpp` keeps the output bounded by the validated input.
    match codec {
        // Packed 4:2:2 (2 bpp) and packed 32bpp BGRA (4 bpp): one plane of rows.
        Codec::Uyvy | Codec::Yuy2 => {
            if stride < width.checked_mul(2)? {
                return None;
            }
            height.checked_mul(stride)
        }
        Codec::Bgra => {
            if stride < width.checked_mul(4)? {
                return None;
            }
            height.checked_mul(stride)
        }
        // NV12: full-res Y plane (1 bpp) + interleaved UV plane of ceil(height/2) rows.
        Codec::Nv12 => {
            if stride < width {
                return None;
            }
            let y = height.checked_mul(stride)?;
            let uv = stride.checked_mul(height.div_ceil(2))?;
            y.checked_add(uv)
        }
        // YV12: full-res Y plane (1 bpp) + two quarter-res chroma planes.
        Codec::Yv12 => {
            if stride < width {
                return None;
            }
            let y = height.checked_mul(stride)?;
            let uv = (height / 2).checked_mul(stride / 2)?.checked_mul(2)?;
            y.checked_add(uv)
        }
        // UYVA: packed UYVY plane (2 bpp) followed by a full-res 8-bit alpha plane.
        Codec::Uyva => {
            if stride < width.checked_mul(2)? {
                return None;
            }
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
///
/// P216 is fully strided, so every plane is sized from the luma row `stride`
/// rather than the pixel width. For conforming OMT frames `stride == width * 2`,
/// so each plane is `width * height` u16. `width` is used only to reject a
/// `stride` too small for the declared width (16-bit luma needs `stride >=
/// width * 2` bytes), matching the constraint the packed branches enforce.
fn p216_input_len(width: usize, height: usize, stride: usize, alpha: bool) -> Option<usize> {
    // 16-bit luma: each pixel is 2 bytes, so a row needs at least `width * 2`.
    if stride < width.checked_mul(2)? {
        return None;
    }
    // The Y plane, the interleaved UV plane and (for PA16) the alpha plane all
    // use the luma row stride. Counted in 16-bit (u16) elements, then doubled.
    let plane = (stride / 2).checked_mul(height)?;
    // Y plane + interleaved UV plane, both at the luma stride.
    let mut elements = plane.checked_mul(2)?;
    if alpha {
        elements = elements.checked_add(plane)?;
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

    #[test]
    fn rejects_stride_smaller_than_width() {
        // A stride too small for the declared width must be rejected so the
        // converters' width-sized output allocation stays bounded by the input.
        // Packed 4:2:2 needs stride >= width * 2.
        assert_eq!(required_input_len(Codec::Uyvy, 100_000, 1, 4), None);
        assert_eq!(required_input_len(Codec::Yuy2, 100_000, 1, 4), None);
        // BGRA needs stride >= width * 4.
        assert_eq!(required_input_len(Codec::Bgra, 4, 1, 8), None);
        // NV12/YV12 luma needs stride >= width.
        assert_eq!(required_input_len(Codec::Nv12, 8, 1, 4), None);
        assert_eq!(required_input_len(Codec::Yv12, 8, 1, 4), None);
        // UYVA UYVY portion needs stride >= width * 2.
        assert_eq!(required_input_len(Codec::Uyva, 8, 1, 4), None);
        // P216/PA16 16-bit luma needs stride (bytes) >= width * 2.
        assert_eq!(required_input_len(Codec::P216, 8, 1, 4), None);
        assert_eq!(required_input_len(Codec::Pa16, 8, 1, 4), None);
    }

    #[test]
    fn accepts_exact_dense_stride() {
        // The dense (stride == width * bpp) boundary must still be accepted.
        assert_eq!(required_input_len(Codec::Uyvy, 4, 2, 8), Some(16));
        assert_eq!(required_input_len(Codec::Bgra, 4, 2, 16), Some(32));
        // P216: Y + UV planes, each width*height u16 -> 2 * 4 * 2 * 2 bytes.
        assert_eq!(required_input_len(Codec::P216, 4, 2, 8), Some(32));
        // PA16 adds the alpha plane: 3 * 4 * 2 * 2 bytes.
        assert_eq!(required_input_len(Codec::Pa16, 4, 2, 8), Some(48));
    }
}
