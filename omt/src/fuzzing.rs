//! Fuzzing hooks for this crate's untrusted-input handling.
//!
//! **Not part of the public API.** Everything here is `#[doc(hidden)]`, gated
//! behind the off-by-default `unstable-fuzzing` feature (plus `cfg(test)`), and
//! exempt from semver. It exists so `fuzz/` can drive the parts of the crate
//! that are unreachable through the safe public API — building a frame whose
//! *header* is hostile.
//!
//! # What is being fuzzed, and why it needs a hook
//!
//! A received `MediaFrame` pairs a buffer libomt genuinely owns with header
//! fields — `Codec`, `Width`, `Height`, `Stride`, `Channels`,
//! `SamplesPerChannel` — that came off the network and are **attacker
//! controlled**. `video_conversion::required_input_len` is the single gate that
//! stops those fields from driving an out-of-bounds slice or an unbounded
//! allocation in the converters (see that module's docs).
//!
//! The public constructors cannot reproduce this: `VideoFrameBuilder` validates
//! its dimensions, so it can never produce the negative, absurd, or
//! self-inconsistent headers that matter. [`fuzz_media_frame`] builds the frame
//! directly instead.
//!
//! # What stays honest
//!
//! The data pointer and its length always describe the real payload slice. Only
//! the *interpretation* fields are fuzzed. That mirrors reality — libomt hands
//! over a buffer it really allocated, and the header is what must not be trusted
//! — and it keeps the harness itself sound: a lying `DataLength` would make the
//! harness read out of bounds and report its own bug as a crash in the crate.

use crate::frame::MediaFrame;
use crate::types::{Codec, FrameType};

/// Bytes consumed as the fuzzed frame header; the remainder is the payload.
const HEADER_LEN: usize = 24;

/// Reads a little-endian `i32` at `offset`, which must be in bounds.
fn i32_at(data: &[u8], offset: usize) -> i32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&data[offset..offset + 4]);
    i32::from_le_bytes(buf)
}

/// Maps a selector byte onto a codec's raw FFI value.
///
/// One selector out of every eleven yields a value that is *not* a valid
/// `OMTCodec`, so the `Codec::from_ffi` -> `None` path (an unknown codec from a
/// newer or hostile sender) is exercised too.
fn codec_value(selector: u8) -> u32 {
    match selector % 11 {
        0 => Codec::Vmx1.to_ffi(),
        1 => Codec::Fpa1.to_ffi(),
        2 => Codec::Uyvy.to_ffi(),
        3 => Codec::Yuy2.to_ffi(),
        4 => Codec::Bgra.to_ffi(),
        5 => Codec::Nv12.to_ffi(),
        6 => Codec::Yv12.to_ffi(),
        7 => Codec::Uyva.to_ffi(),
        8 => Codec::P216.to_ffi(),
        9 => Codec::Pa16.to_ffi(),
        _ => 0xDEAD_BEEF, // not a valid OMTCodec
    }
}

/// Whether `codec` has an 8-bit RGB decode path, i.e. whether a frame the input
/// gate accepts must convert successfully.
///
/// The 16-bit-only codecs (P216/PA16) and the ones with no pixel layout at all
/// (VMX1/FPA1) legitimately return `None` from `to_rgb8`, so they are excluded
/// from the gate-agreement check below.
fn decodes_to_rgb8(codec: Codec) -> bool {
    matches!(
        codec,
        Codec::Uyvy | Codec::Yuy2 | Codec::Nv12 | Codec::Yv12 | Codec::Bgra | Codec::Uyva
    )
}

/// Drives every untrusted-input path on [`MediaFrame`] with a hostile header.
///
/// Interprets the first [`HEADER_LEN`] bytes of `data` as frame header fields
/// and the rest as the payload, then exercises the accessors and all four RGB
/// conversions, asserting the invariants that must hold for *any* input:
///
/// * No path panics, indexes out of bounds, or allocates unboundedly.
/// * A successful conversion returns exactly `width * height` pixels — the
///   property that catches both an over-allocating converter and one that
///   silently produces a short buffer.
/// * A successful `as_f32_planar` returns exactly `channels` planes of
///   `samples_per_channel` samples each.
/// * Every reported buffer length stays within the payload that backs it.
/// * `to_static` deep-copies faithfully, even from a nonsense header.
///
/// Inputs shorter than the header are ignored.
#[doc(hidden)]
pub fn fuzz_media_frame(data: &[u8]) {
    if data.len() < HEADER_LEN {
        return;
    }
    let (header, payload) = data.split_at(HEADER_LEN);

    // `DataLength` is an `i32`; a payload past that is not representable and is
    // not what this harness is testing.
    let Ok(payload_len) = i32::try_from(payload.len()) else {
        return;
    };

    let frame_type = match header[1] % 4 {
        0 => FrameType::VIDEO,
        1 => FrameType::AUDIO,
        2 => FrameType::METADATA,
        _ => FrameType::from_bits_retain(u32::from(header[1])),
    };

    let width = i32_at(header, 4);
    let height = i32_at(header, 8);
    let channels = i32_at(header, 16);
    let samples_per_channel = i32_at(header, 20);

    let ffi = omt_sys::OMTMediaFrame {
        Type: frame_type.to_ffi(),
        Timestamp: -1,
        Codec: codec_value(header[0]),
        Width: width,
        Height: height,
        Stride: i32_at(header, 12),
        Flags: u32::from(header[3]),
        FrameRateN: 30,
        FrameRateD: 1,
        AspectRatio: 16.0 / 9.0,
        ColorSpace: u32::from(header[2]),
        SampleRate: 48_000,
        Channels: channels,
        SamplesPerChannel: samples_per_channel,
        // Truthful: this really is the payload and really is that long.
        Data: payload.as_ptr() as *mut _,
        DataLength: payload_len,
        // A null pointer paired with a positive length is a *normal* state for
        // received frames (see `MediaFrame::from_ffi_ptr`), so fuzz it: the
        // accessors must mask it and `to_static` must not copy it forward.
        CompressedData: std::ptr::null_mut(),
        CompressedLength: i32_at(header, 12),
        // Also truthful — aimed at the same payload, exercising the NUL
        // trimming and UTF-8 handling on arbitrary bytes.
        FrameMetadata: payload.as_ptr() as *mut _,
        FrameMetadataLength: payload_len,
    };

    // SAFETY: every field is initialized. The `Data` and `FrameMetadata`
    // pointers aim at `payload`, which is borrowed from `data` and outlives
    // `frame` (both are confined to this function body), so the frame cannot
    // dangle. `DataLength`/`FrameMetadataLength` describe that slice exactly.
    let frame = unsafe { MediaFrame::from_owned_ffi(ffi) };

    // --- Accessors must stay inside the payload that backs them -------------
    assert!(frame.data().len() <= payload.len());
    assert!(frame.compressed_data().is_empty(), "null buffer must mask");
    if let Some(metadata) = frame.frame_metadata_bytes() {
        assert!(metadata.len() <= payload.len());
        assert!(
            !metadata.contains(&0),
            "metadata must be trimmed at the NUL"
        );
    }
    let _ = frame.try_frame_metadata();
    let _ = frame.frame_metadata();
    let _ = frame.as_utf8();
    let _ = frame.frame_type();
    let _ = frame.codec();
    let _ = frame.color_space();
    let _ = frame.flags();
    let _ = frame.frame_rate_rational();

    // --- Conversions: bounded output, or none ------------------------------
    // A successful conversion must yield exactly one pixel per declared pixel.
    // The gate guarantees the input buffer covers that, so this also bounds the
    // allocation by the (validated) input rather than by the raw header.
    let expected_pixels = usize::try_from(width)
        .ok()
        .zip(usize::try_from(height).ok())
        .and_then(|(w, h)| w.checked_mul(h));

    macro_rules! check_conversion {
        ($call:expr, $what:literal) => {
            if let Some(pixels) = $call {
                assert_eq!(
                    Some(pixels.len()),
                    expected_pixels,
                    "{} produced {} pixels for {}x{}",
                    $what,
                    pixels.len(),
                    width,
                    height
                );
            }
        };
    }

    let rgb8 = frame.to_rgb8();
    let rgba8 = frame.to_rgba8();
    check_conversion!(rgb8.as_ref(), "to_rgb8");
    check_conversion!(rgba8.as_ref(), "to_rgba8");
    check_conversion!(frame.to_rgb16().as_ref(), "to_rgb16");
    check_conversion!(frame.to_rgba16().as_ref(), "to_rgba16");

    // --- The gate and the converters must agree ----------------------------
    // `required_input_len` accepting a frame is a promise that the converter for
    // that codec can decode it. When the two disagree the frame is rejected with
    // a bare `None`, indistinguishable from "this codec has no 8-bit path" — the
    // exact shape of two real bugs: YV12 flooring its chroma dimensions, and the
    // packed 4:2:2 converters handing `yuv` a padded plane it refuses.
    if let Some(codec) = frame.codec()
        && decodes_to_rgb8(codec)
        && let Some(required) = crate::video_conversion::required_input_len(
            codec,
            width as usize,
            height as usize,
            frame.stride() as usize,
        )
        && frame.data().len() >= required
    {
        assert!(
            rgb8.is_some() && rgba8.is_some(),
            "gate accepted {codec} {width}x{height} stride {} ({required} bytes required, \
             {} available) but conversion returned None",
            frame.stride(),
            frame.data().len(),
        );
    }

    // --- Audio planes ------------------------------------------------------
    if let Some(planes) = frame.as_f32_planar() {
        assert_eq!(planes.len(), channels as usize);
        for plane in planes {
            assert_eq!(plane.len(), samples_per_channel as usize);
        }
    }

    // --- Deep copy must be faithful and self-consistent --------------------
    let detached = frame.to_static();
    assert_eq!(detached.data(), frame.data());
    assert_eq!(
        detached.frame_metadata_bytes(),
        frame.frame_metadata_bytes()
    );
    // A copy must never inherit a pointer it did not duplicate, nor advertise a
    // buffer it does not have — that is what makes `MediaFrame<'static>` sound.
    assert!(detached.as_ffi().CompressedData.is_null());
    assert_eq!(detached.as_ffi().CompressedLength, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic sweep over adversarial headers, so the invariants
    /// [`fuzz_media_frame`] asserts are checked by ordinary `cargo test` on
    /// stable — not only when someone runs `cargo fuzz` on nightly.
    ///
    /// Every `width`/`height`/`stride` combination of [`DIMENSIONS`] is tried
    /// against every codec and payload size. The full cross product matters:
    /// the interesting failures live in the *relationships* between the three
    /// (a stride narrower than the width, an odd height against an even stride),
    /// not in any one of them alone.
    ///
    /// The value set is kept small enough to stay a couple of seconds in a debug
    /// build. Depth beyond that is the `fuzz/` target's job.
    #[test]
    fn adversarial_headers_uphold_frame_invariants() {
        /// Each entry earns its place:
        /// `i32::MIN`/`i32::MAX` overflow the size arithmetic, negatives become
        /// huge `usize`s when cast, `0` is degenerate, `3`/`17` are odd (how the
        /// YV12 chroma-rounding bug slipped through), `4`/`64` are the even,
        /// aligned cases.
        const DIMENSIONS: &[i32] = &[i32::MIN, -1, 0, 1, 3, 4, 17, 64, i32::MAX];
        /// Nothing, too short, exactly enough for a small frame, and roomy.
        const PAYLOADS: &[usize] = &[0, 15, 32, 128, 1024];

        let mut cases = 0usize;
        for codec_sel in 0u8..11 {
            for &width in DIMENSIONS {
                for &height in DIMENSIONS {
                    for &stride in DIMENSIONS {
                        for &payload_len in PAYLOADS {
                            fuzz_media_frame(&case(
                                codec_sel,
                                0,
                                width,
                                height,
                                stride,
                                payload_len,
                            ));
                            cases += 1;
                        }
                    }
                }
            }
        }

        // Guard against the loops being narrowed into uselessness by a later edit.
        assert!(cases > 30_000, "sweep collapsed to {cases} cases");
    }

    /// The frame-type field is orthogonal to the conversion paths, so it gets a
    /// small dedicated pass rather than multiplying the sweep above.
    #[test]
    fn every_frame_type_selector_is_handled() {
        for type_sel in 0u8..=255 {
            for &dim in &[-1i32, 0, 4, 17] {
                fuzz_media_frame(&case(2, type_sel, dim, dim, dim * 2, 128));
            }
        }
    }

    /// Assembles one fuzz input from explicit header fields.
    fn case(
        codec_sel: u8,
        type_sel: u8,
        width: i32,
        height: i32,
        stride: i32,
        payload_len: usize,
    ) -> Vec<u8> {
        let mut input = Vec::with_capacity(HEADER_LEN + payload_len);
        input.push(codec_sel);
        input.push(type_sel);
        input.push(1); // color space selector
        input.push(0b0011_0011); // flags
        input.extend_from_slice(&width.to_le_bytes());
        input.extend_from_slice(&height.to_le_bytes());
        input.extend_from_slice(&stride.to_le_bytes());
        // Channels / samples-per-channel, sized so the audio path is sometimes
        // satisfiable (`payload = channels * samples * 4`).
        input.extend_from_slice(&2i32.to_le_bytes());
        input.extend_from_slice(&((payload_len / 8) as i32).to_le_bytes());
        debug_assert_eq!(input.len(), HEADER_LEN);
        // Non-zero, non-ASCII bytes: exercises the NUL trimming and the
        // invalid-UTF-8 metadata path.
        input.extend((0..payload_len).map(|i| (i % 251).wrapping_add(1) as u8));
        input
    }

    /// Inputs too short to carry a header are ignored rather than panicking.
    #[test]
    fn short_inputs_are_ignored() {
        for len in 0..HEADER_LEN {
            fuzz_media_frame(&vec![0xAB; len]);
        }
    }

    /// A payload of pure NUL bytes: metadata trims to empty rather than absent,
    /// and nothing downstream trips over the zero-length result.
    #[test]
    fn all_nul_payload_is_handled() {
        let mut input = vec![0u8; HEADER_LEN];
        input[0] = 2; // UYVY
        input[4..8].copy_from_slice(&4i32.to_le_bytes()); // width
        input[8..12].copy_from_slice(&4i32.to_le_bytes()); // height
        input[12..16].copy_from_slice(&8i32.to_le_bytes()); // stride
        input.extend(std::iter::repeat_n(0u8, 64));

        fuzz_media_frame(&input);
    }
}
