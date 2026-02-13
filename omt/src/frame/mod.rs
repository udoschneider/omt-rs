//! Media frame types for video, audio, and metadata.

mod audio;
mod metadata;
mod video;

use crate::types::{Codec, FrameType};
use std::slice;

/// A media frame containing video, audio, or metadata.
///
/// This is a safe wrapper around the FFI `OMTMediaFrame` structure.
/// The frame data is valid until the next receive call or until the frame is dropped.
///
/// The frame type can be queried using [`frame_type()`](MediaFrame::frame_type).
/// Type-specific methods are available in dedicated impl blocks for video, audio, and metadata frames.
#[derive(Debug)]
pub struct MediaFrame {
    ffi: omt_sys::OMTMediaFrame,
}

// Common methods available for all frame types
impl MediaFrame {
    /// Creates a new zeroed media frame.
    pub(crate) fn new() -> Self {
        Self {
            ffi: unsafe { std::mem::zeroed() },
        }
    }

    /// Creates a frame from an FFI pointer (receive only).
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a properly initialized OMTMediaFrame.
    pub(crate) unsafe fn from_ffi_ptr(ptr: *const omt_sys::OMTMediaFrame) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ffi: unsafe { *ptr },
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
    /// must remain valid for the lifetime of the returned MediaFrame.
    pub(crate) unsafe fn from_owned_ffi(ffi: omt_sys::OMTMediaFrame) -> Self {
        Self { ffi }
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
    pub fn data(&self) -> &[u8] {
        if self.ffi.Data.is_null() || self.ffi.DataLength <= 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
            }
        }
    }

    /// Returns the compressed data (VMX1) if available.
    pub fn compressed_data(&self) -> &[u8] {
        if self.ffi.CompressedData.is_null() || self.ffi.CompressedLength <= 0 {
            &[]
        } else {
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

unsafe impl Send for MediaFrame {}
