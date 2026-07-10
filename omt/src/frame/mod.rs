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
            debug_assert!(
                !(ffi.CompressedLength > 0 && ffi.CompressedData.is_null()),
                "libomt returned CompressedData=null with CompressedLength={}",
                ffi.CompressedLength,
            );
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

    /// Returns the per-frame metadata as a UTF-8 string if available.
    ///
    /// Returns an empty string if no metadata is present.
    /// If the metadata is not valid UTF-8, this will return an empty string.
    ///
    /// The returned string slice borrows from `self` and cannot outlive this frame.
    pub fn frame_metadata(&self) -> &str {
        if self.ffi.FrameMetadata.is_null() || self.ffi.FrameMetadataLength <= 0 {
            ""
        } else {
            let bytes = unsafe {
                slice::from_raw_parts(
                    self.ffi.FrameMetadata as *const u8,
                    self.ffi.FrameMetadataLength as usize,
                )
            };
            // Remove null terminator if present
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            std::str::from_utf8(&bytes[..end]).unwrap_or("")
        }
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
