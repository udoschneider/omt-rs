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
/// The frame type can be queried using [`frame_type()`](MediaFrame::frame_type).
/// Type-specific methods are available in dedicated impl blocks for video, audio, and metadata frames.
#[derive(Debug)]
pub struct MediaFrame<'a> {
    ffi: omt_sys::OMTMediaFrame,
    _marker: PhantomData<&'a ()>,
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
            Some(Self {
                ffi: unsafe { *ptr },
                _marker: PhantomData,
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
        }
    }

    /// Returns a reference to the underlying FFI structure.
    pub(crate) fn as_ffi(&self) -> &omt_sys::OMTMediaFrame {
        &self.ffi
    }

    /// Returns a mutable reference to the underlying FFI structure.
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
    /// The returned slice is valid for the lifetime of this MediaFrame.
    pub fn data(&self) -> &'a [u8] {
        if self.ffi.Data.is_null() || self.ffi.DataLength <= 0 {
            &[]
        } else {
            // SAFETY: The lifetime 'a ensures this slice cannot outlive the source data.
            // The C API guarantees Data is valid for the frame's lifetime.
            unsafe {
                slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
            }
        }
    }

    /// Returns the compressed data (VMX1) if available.
    ///
    /// The returned slice is valid for the lifetime of this MediaFrame.
    pub fn compressed_data(&self) -> &'a [u8] {
        if self.ffi.CompressedData.is_null() || self.ffi.CompressedLength <= 0 {
            &[]
        } else {
            // SAFETY: The lifetime 'a ensures this slice cannot outlive the source data.
            // The C API guarantees CompressedData is valid for the frame's lifetime.
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
    /// The returned string slice is valid for the lifetime of this MediaFrame.
    pub fn frame_metadata(&self) -> &'a str {
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

// SAFETY: MediaFrame contains borrowed data with lifetime 'a, which prevents
// use-after-free. The underlying C library is thread-safe for read operations.
unsafe impl<'a> Send for MediaFrame<'a> {}
