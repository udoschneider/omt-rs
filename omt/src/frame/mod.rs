//! Media frame types for video, audio, and metadata.

mod audio;
mod metadata;
mod video;

use crate::types::{Codec, FrameType};
use std::marker::PhantomData;
use std::slice;

/// A media frame containing video, audio, or metadata.
///
/// This is a safe wrapper around the FFI `OMTMediaFrame` structure.
///
/// # Lifetime
///
/// The lifetime parameter `'a` ensures that the frame data cannot outlive its source.
/// For frames received from the C API, this is tied to the receiver/sender instance.
/// For frames created from `OwnedMediaFrame`, this is tied to the owned frame's lifetime.
///
/// **IMPORTANT:** Frames received from `Receiver::receive()` or `Sender::receive_metadata()`
/// are only valid until the next call to those methods. The lifetime parameter enforces this.
///
/// # Cloning
///
/// `MediaFrame` implements `Clone` to perform a **deep copy** of all frame data.
/// This is useful when using the unsafe API (`receive_unchecked`) and you need to
/// keep a frame beyond the next receive call.
///
/// **Warning:** Cloning copies all frame data (potentially ~64MB for 4K 16-bit RGBA).
/// Use sparingly and only when necessary. Consider processing frames immediately
/// instead of cloning them.
///
/// The frame type can be queried using [`frame_type()`](MediaFrame::frame_type).
/// Type-specific methods are available in dedicated impl blocks for video, audio, and metadata frames.
#[derive(Debug)]
pub struct MediaFrame<'a> {
    ffi: omt_sys::OMTMediaFrame,
    _marker: PhantomData<&'a ()>,
    // `Some` for cloned (deep-copied) frames that own their buffers, `None` for
    // frames borrowing C-owned or `OwnedMediaFrame`-owned memory. When `Some`,
    // the pointers in `ffi` alias into these boxes, which free themselves on
    // drop — so no manual `Drop` impl is required. Held solely for that drop
    // side-effect (the pointers are read through `ffi`, not this field), hence
    // `dead_code` is allowed.
    #[allow(dead_code)]
    owned: Option<OwnedBuffers>,
}

/// Heap buffers owned by a cloned [`MediaFrame`].
///
/// A cloned frame's `OMTMediaFrame` pointers alias into these boxes; dropping
/// this struct releases them through ordinary Rust ownership, which is why
/// `MediaFrame` needs no hand-written `Drop`.
#[derive(Debug, Default)]
struct OwnedBuffers {
    data: Option<Box<[u8]>>,
    compressed: Option<Box<[u8]>>,
    metadata: Option<Box<[u8]>>,
}

// Common methods available for all frame types
impl<'a> MediaFrame<'a> {
    /// Creates a frame from an FFI pointer (receive only).
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a properly initialized OMTMediaFrame.
    /// The caller must ensure that the data pointed to by the OMTMediaFrame remains
    /// valid for the lifetime 'a.
    pub(crate) unsafe fn from_ffi_ptr(ptr: *const omt_sys::OMTMediaFrame) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            let ffi = unsafe { *ptr };

            // Canary for assumption #1 in the crate-level "Safety assumptions
            // about the `libomt` C library" docs: a positive length must be
            // paired with a non-null pointer. `data()`, `compressed_data()` and
            // `frame_metadata()` already guard against `null`/`len <= 0` (so
            // release builds stay sound regardless), but a frame that reports
            // `len > 0` with a null base would mean libomt broke its buffer
            // contract. These are `debug_assert!`s: zero cost in release, but
            // they trip the test suite (in CI) if a future libomt version
            // regresses, surfacing it as a failed test rather than latent UB.
            debug_assert!(
                !(ffi.DataLength > 0 && ffi.Data.is_null()),
                "libomt returned Data=null with DataLength={} (broken frame-buffer contract)",
                ffi.DataLength,
            );
            // NOTE: There is intentionally *no* analogous canary for the
            // `CompressedData`/`CompressedLength` pair. `libomt.h` documents that
            // `CompressedData` is only populated when the `IncludeCompressed` or
            // `CompressedOnly` receive flags are set, but libomt still fills in
            // `CompressedLength` with the compressed VMX1 frame size regardless.
            // So `CompressedLength > 0` with a null `CompressedData` is a normal,
            // expected state (observed with plain `ReceiveFlags::NONE`), not a
            // broken buffer contract. `compressed_data()` guards the null case
            // and returns `&[]` there, so release builds remain sound.
            debug_assert!(
                !(ffi.FrameMetadataLength > 0 && ffi.FrameMetadata.is_null()),
                "libomt returned FrameMetadata=null with FrameMetadataLength={}",
                ffi.FrameMetadataLength,
            );

            Some(Self {
                ffi,
                _marker: PhantomData,
                owned: None, // Borrowed from C library
            })
        }
    }

    /// Creates a frame from an owned FFI structure.
    ///
    /// This is used by frame builders to create frames that borrow from owned data.
    ///
    /// # Safety
    ///
    /// The FFI structure must be properly initialized and all pointers within it
    /// must remain valid for the lifetime 'a of the returned MediaFrame.
    pub(crate) unsafe fn from_owned_ffi(ffi: omt_sys::OMTMediaFrame) -> Self {
        Self {
            ffi,
            _marker: PhantomData,
            owned: None, // Borrowed from OwnedMediaFrame
        }
    }

    /// Returns a reference to the underlying FFI structure.
    pub(crate) fn as_ffi(&self) -> &omt_sys::OMTMediaFrame {
        &self.ffi
    }

    /// Returns a mutable reference to the underlying FFI structure.
    #[allow(dead_code)] // Companion to `as_ffi`; kept for API symmetry.
    pub(crate) fn as_ffi_mut(&mut self) -> &mut omt_sys::OMTMediaFrame {
        &mut self.ffi
    }

    /// Returns the frame type.
    pub fn frame_type(&self) -> FrameType {
        FrameType::from_ffi(self.ffi.Type).unwrap_or(FrameType::NONE)
    }

    /// Returns the timestamp (where 1 second = 10,000,000 units).
    ///
    /// A value of -1 indicates auto-generated timestamps.
    pub fn timestamp(&self) -> i64 {
        self.ffi.Timestamp
    }

    /// Returns the codec.
    pub fn codec(&self) -> Option<Codec> {
        Codec::from_ffi(self.ffi.Codec)
    }

    /// Returns the frame data as a byte slice.
    ///
    /// The returned slice borrows from `self` and cannot outlive this frame. For
    /// borrowed frames the buffer is owned by the C library and stays valid until
    /// the next receive call (enforced on the safe path by the receiver's
    /// `&mut self` borrow); for cloned frames the buffer is owned by this frame.
    ///
    /// Because the slice borrows `self`, it cannot dangle past the frame's `Drop`.
    /// The following use-after-free is rejected at compile time:
    ///
    /// ```compile_fail
    /// # use omt::MediaFrame;
    /// # fn demo(frame: &MediaFrame<'_>) {
    /// let slice: &[u8];
    /// {
    ///     let owned = frame.clone(); // deep copy; owns its buffer
    ///     slice = owned.data();
    /// } // `owned` dropped here, its buffer freed
    /// let _ = slice[0]; // ERROR: `slice` borrows `owned`, which does not live long enough
    /// # }
    /// ```
    pub fn data(&self) -> &[u8] {
        if self.ffi.Data.is_null() || self.ffi.DataLength <= 0 {
            &[]
        } else {
            // SAFETY: The returned slice is tied to the borrow of `self`, so it
            // cannot outlive the frame — and therefore cannot outlive a cloned
            // frame's owned buffer (freed in Drop) nor a borrowed frame's C buffer.
            // `DataLength` is > 0 here and gives the length of `Data` in bytes.
            unsafe {
                slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
            }
        }
    }

    /// Returns the compressed data (VMX1) if available.
    ///
    /// The returned slice borrows from `self` and cannot outlive this frame.
    pub fn compressed_data(&self) -> &[u8] {
        if self.ffi.CompressedData.is_null() || self.ffi.CompressedLength <= 0 {
            &[]
        } else {
            // SAFETY: The returned slice is tied to the borrow of `self`, so it
            // cannot outlive the frame (and thus not the underlying buffer).
            // `CompressedLength` is > 0 here and gives the length in bytes.
            unsafe {
                slice::from_raw_parts(
                    self.ffi.CompressedData as *const u8,
                    self.ffi.CompressedLength as usize,
                )
            }
        }
    }

    /// Returns the raw per-frame metadata bytes if present.
    ///
    /// This is the primitive accessor for per-frame metadata; the string-typed
    /// [`try_frame_metadata`](MediaFrame::try_frame_metadata) and
    /// [`frame_metadata`](MediaFrame::frame_metadata) are layered on top of it.
    ///
    /// At the ABI level the metadata is an untyped `void* FrameMetadata` with an
    /// `int FrameMetadataLength`, so this accessor deliberately exposes bytes
    /// rather than a `&str`: by the OMT protocol *convention* the buffer holds
    /// UTF-8-encoded, null-terminated XML, but it is supplied by an untrusted
    /// network sender and may not honour that convention.
    ///
    /// The three cases are distinguished as follows:
    /// - **absent** — `FrameMetadata` is null or `FrameMetadataLength <= 0`:
    ///   returns `None`.
    /// - **present but empty** — returns `Some(&[])`.
    /// - **present** — returns `Some(&[u8])` borrowing `self`, trimmed at the
    ///   first NUL byte so the trailing null terminator is *not* included.
    ///
    /// The returned slice borrows from `self` and cannot outlive this frame.
    pub fn frame_metadata_bytes(&self) -> Option<&[u8]> {
        if self.ffi.FrameMetadata.is_null() || self.ffi.FrameMetadataLength <= 0 {
            None
        } else {
            // SAFETY: The returned slice is tied to the borrow of `self`, so it
            // cannot outlive the frame — and therefore cannot outlive a cloned
            // frame's owned buffer (freed in Drop) nor a borrowed frame's C
            // buffer (valid until the next receive). `FrameMetadataLength` is
            // > 0 here and gives the length of `FrameMetadata` in bytes.
            let bytes = unsafe {
                slice::from_raw_parts(
                    self.ffi.FrameMetadata as *const u8,
                    self.ffi.FrameMetadataLength as usize,
                )
            };
            // Trim at the first NUL: the buffer is null-terminated by protocol
            // convention, and the terminator is not part of the payload.
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            Some(&bytes[..end])
        }
    }

    /// Returns the per-frame metadata as a UTF-8 string, preserving decode errors.
    ///
    /// Unlike [`frame_metadata`](MediaFrame::frame_metadata), this distinguishes
    /// the three situations the ABI allows instead of collapsing them:
    /// - **absent** (null pointer or non-positive length) — returns `None`.
    /// - **present and valid UTF-8** (possibly empty) — returns `Some(Ok(s))`.
    /// - **present but not valid UTF-8** — returns `Some(Err(e))`.
    ///
    /// The metadata is UTF-8-encoded XML by OMT protocol *convention* over an
    /// untyped `void*` buffer from an untrusted network sender, so the invalid
    /// case is real rather than theoretical — prefer this accessor when a
    /// corrupt-but-present signal must not be silently discarded.
    ///
    /// The returned string slice borrows from `self` and cannot outlive this frame.
    pub fn try_frame_metadata(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.frame_metadata_bytes().map(std::str::from_utf8)
    }

    /// Returns the per-frame metadata as a UTF-8 string if available.
    ///
    /// This is the **lossy** convenience accessor: it returns an empty string in
    /// all three of the cases that
    /// [`try_frame_metadata`](MediaFrame::try_frame_metadata) distinguishes —
    /// metadata absent, present but empty, and present but not valid UTF-8. Use
    /// [`try_frame_metadata`](MediaFrame::try_frame_metadata) or
    /// [`frame_metadata_bytes`](MediaFrame::frame_metadata_bytes) when those
    /// cases must be told apart (e.g. so a corrupt-but-present payload is not
    /// silently swallowed).
    ///
    /// The returned string slice borrows from `self` and cannot outlive this frame.
    pub fn frame_metadata(&self) -> &str {
        self.try_frame_metadata().and_then(Result::ok).unwrap_or("")
    }
}

impl<'a> Clone for MediaFrame<'a> {
    /// Creates a deep copy of this frame.
    ///
    /// **Performance Warning:** This method copies all frame data, including:
    /// - Main data buffer (potentially ~64MB for 4K 16-bit RGBA)
    /// - Compressed data buffer (if present)
    /// - Frame metadata string (if present)
    ///
    /// # Use Cases
    ///
    /// This is primarily useful when using `receive_unchecked()` and you need to
    /// store frames beyond the next receive call:
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # use std::sync::Arc;
    /// let receiver = Arc::new(Receiver::new("omt://localhost:6400",
    ///     FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?);
    ///
    /// let mut frames = Vec::new();
    /// for _ in 0..10 {
    ///     unsafe {
    ///         if let Some(frame) = receiver.receive_unchecked(FrameType::VIDEO, 1000)? {
    ///             // Clone to keep the frame data beyond next receive
    ///             frames.push(frame.clone());
    ///         }
    ///     }
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    ///
    /// # Alternatives
    ///
    /// Consider these alternatives before cloning:
    /// - Process frames immediately without storing them
    /// - Use `OwnedMediaFrame` from frame builders for created frames
    /// - Use the safe API (`receive()`) which prevents this issue at compile time
    fn clone(&self) -> Self {
        // Copy the FFI structure, then repoint its buffer pointers at freshly
        // allocated boxes that this clone owns. No `unsafe` and no manual
        // deallocation: `OwnedBuffers` frees everything on drop.
        let mut ffi = self.ffi;
        let mut owned = OwnedBuffers::default();

        // Deep copy the main data buffer.
        if !self.ffi.Data.is_null() && self.ffi.DataLength > 0 {
            let mut buf = self.data().to_vec().into_boxed_slice();
            ffi.Data = buf.as_mut_ptr() as *mut std::os::raw::c_void;
            owned.data = Some(buf);
        }

        // Deep copy the compressed data buffer.
        if !self.ffi.CompressedData.is_null() && self.ffi.CompressedLength > 0 {
            let mut buf = self.compressed_data().to_vec().into_boxed_slice();
            ffi.CompressedData = buf.as_mut_ptr() as *mut std::os::raw::c_void;
            owned.compressed = Some(buf);
        }

        // Deep copy the frame metadata string (re-appending the null terminator).
        if !self.ffi.FrameMetadata.is_null() && self.ffi.FrameMetadataLength > 0 {
            let mut bytes = self.frame_metadata().as_bytes().to_vec();
            bytes.push(0); // Null terminator
            let len = bytes.len() as i32;
            let mut buf = bytes.into_boxed_slice();
            ffi.FrameMetadata = buf.as_mut_ptr() as *mut std::os::raw::c_void;
            ffi.FrameMetadataLength = len;
            owned.metadata = Some(buf);
        }

        Self {
            ffi,
            _marker: PhantomData,
            owned: Some(owned), // Cloned frame owns its data
        }
    }
}

// SAFETY: Moving a `MediaFrame` to another thread is sound in both of its forms:
//
// * Cloned frames (`owned.is_some()`) own heap `Box` buffers outright, so
//   ownership transfer is unconditionally safe.
//
// * Borrowed frames (`owned.is_none()`) hold a by-value copy of the
//   `OMTMediaFrame` struct whose `Data`/`CompressedData`/`FrameMetadata`
//   pointers aim into a receiver-owned C buffer. That buffer is freed *only* by
//   the next `omt_receive` of the same frame type on the same instance, or by
//   destroying the instance — never asynchronously by an internal thread (the
//   library keeps one `lastVideo`/`lastAudio`/`lastMetadata` slot per instance
//   and recycles it synchronously inside the receive call). On the safe
//   `receive(&mut self)` path the borrow checker therefore serializes any
//   invalidation behind the frame's lifetime regardless of which thread holds
//   the frame; on the `receive_unchecked(&self)` path the same guarantee is the
//   caller's documented safety contract.
//
// This impl deliberately grants `Send` but NOT `Sync`: the raw pointers in
// `OMTMediaFrame` keep `MediaFrame` `!Sync`, so a `&MediaFrame` can never be
// shared across threads — only ownership can be transferred. Do not add an
// `unsafe impl Sync`; nothing here requires it and it would allow two threads
// to alias the same borrowed C buffer.
unsafe impl<'a> Send for MediaFrame<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_builder::VideoFrameBuilder;
    use crate::types::Codec;

    /// Builds a small valid video frame, optionally carrying per-frame metadata.
    fn video_builder(metadata: Option<&str>) -> VideoFrameBuilder {
        let mut builder = VideoFrameBuilder::new()
            .codec(Codec::Bgra)
            .dimensions(2, 2)
            .data(vec![0u8; 2 * 2 * 4]);
        if let Some(m) = metadata {
            builder = builder.frame_metadata(m.to_string());
        }
        builder
    }

    // Case A: metadata absent.
    #[test]
    fn frame_metadata_absent() {
        let owned = video_builder(None).build().unwrap();
        let frame = owned.as_media_frame();

        assert_eq!(frame.frame_metadata_bytes(), None);
        assert!(frame.try_frame_metadata().is_none());
        assert_eq!(frame.frame_metadata(), "");
    }

    // Case B: metadata present and valid UTF-8 (non-empty).
    #[test]
    fn frame_metadata_present_valid() {
        let owned = video_builder(Some("<tally on=\"1\"/>")).build().unwrap();
        let frame = owned.as_media_frame();

        assert_eq!(
            frame.frame_metadata_bytes(),
            Some(b"<tally on=\"1\"/>".as_slice())
        );
        assert_eq!(frame.try_frame_metadata(), Some(Ok("<tally on=\"1\"/>")));
        assert_eq!(frame.frame_metadata(), "<tally on=\"1\"/>");
    }

    // Case B: metadata present but empty. `CString::new("")` stores just the NUL
    // terminator, so the payload trims to an empty slice — still "present".
    #[test]
    fn frame_metadata_present_empty() {
        let owned = video_builder(Some("")).build().unwrap();
        let frame = owned.as_media_frame();

        assert_eq!(frame.frame_metadata_bytes(), Some(b"".as_slice()));
        assert_eq!(frame.try_frame_metadata(), Some(Ok("")));
        assert_eq!(frame.frame_metadata(), "");
    }

    // Case C: metadata present but not valid UTF-8. Fabricate an FFI frame whose
    // metadata buffer holds raw non-UTF-8 bytes plus the trailing NUL, mirroring
    // how `OwnedMediaFrame::as_media_frame` wires up the pointers.
    #[test]
    fn frame_metadata_present_invalid_utf8() {
        // 0xFF, 0xFE is not valid UTF-8; the 0x00 is the null terminator.
        let bytes: [u8; 3] = [0xFF, 0xFE, 0x00];

        let ffi = omt_sys::OMTMediaFrame {
            Type: FrameType::VIDEO.to_ffi(),
            Timestamp: -1,
            Codec: Codec::Bgra.to_ffi(),
            Width: 2,
            Height: 2,
            Stride: 8,
            Flags: 0,
            FrameRateN: 30,
            FrameRateD: 1,
            AspectRatio: 16.0 / 9.0,
            ColorSpace: 0,
            SampleRate: 0,
            Channels: 0,
            SamplesPerChannel: 0,
            Data: std::ptr::null_mut(),
            DataLength: 0,
            CompressedData: std::ptr::null_mut(),
            CompressedLength: 0,
            FrameMetadata: bytes.as_ptr() as *mut _,
            FrameMetadataLength: bytes.len() as i32,
        };

        // SAFETY: `ffi` is fully initialized and its `FrameMetadata` pointer aims
        // at `bytes`, which outlives `frame` (same stack scope). The frame's
        // lifetime is tied to that borrow, so it cannot dangle.
        let frame = unsafe { MediaFrame::from_owned_ffi(ffi) };

        assert_eq!(frame.frame_metadata_bytes(), Some([0xFF, 0xFE].as_slice()));
        assert!(matches!(frame.try_frame_metadata(), Some(Err(_))));
        assert_eq!(frame.frame_metadata(), "");
    }
}
