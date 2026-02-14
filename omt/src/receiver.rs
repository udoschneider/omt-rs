//! OMT receiver for receiving media streams.

use crate::error::{Error, Result};
use crate::frame::MediaFrame;
use crate::statistics::Statistics;
use crate::tally::Tally;
use crate::types::{FrameType, PreferredVideoFormat, Quality, ReceiveFlags, SenderInfo};
use std::ffi::CString;
use std::ptr::NonNull;

/// Receiver for connecting to and receiving media from an OMT sender.
///
/// The receiver automatically manages the connection and provides methods
/// for receiving video, audio, and metadata frames.
///
/// # Frame Lifetime and Safety
///
/// Frames returned by receive methods are only valid until the next receive call.
/// Two APIs are provided:
///
/// - [`receive`](Self::receive): Safe API requiring `&mut self`. This is the
///   recommended method that prevents holding multiple frames through Rust's borrow checker.
///
/// - [`receive_unchecked`](Self::receive_unchecked): Unsafe API using `&self` for
///   performance-critical scenarios where you need concurrent access to other receiver
///   methods. Caller must ensure no previous frame is still held when calling this.
///
/// For most use cases, prefer `receive` for compile-time safety.
pub struct Receiver {
    handle: NonNull<omt_sys::omt_receive_t>,
}

impl Receiver {
    /// Creates a new receiver and begins connecting to the specified sender.
    ///
    /// # Arguments
    ///
    /// * `address` - Address to connect to. Either the full name from discovery
    ///               or a URL in the format `omt://hostname:port`
    /// * `frame_types` - Types of frames to receive (e.g., Video, Audio, Metadata)
    /// * `format` - Preferred uncompressed video format
    /// * `flags` - Optional flags such as preview mode or compressed data
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// let receiver = Receiver::new(
    ///     "omt://localhost:6400",
    ///     FrameType::VIDEO | FrameType::AUDIO,
    ///     PreferredVideoFormat::Uyvy,
    ///     ReceiveFlags::NONE,
    /// )?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn new(
        address: &str,
        frame_types: FrameType,
        format: PreferredVideoFormat,
        flags: ReceiveFlags,
    ) -> Result<Self> {
        let c_address = CString::new(address)?;

        let handle = unsafe {
            omt_sys::omt_receive_create(
                c_address.as_ptr(),
                frame_types.to_ffi(),
                format.to_ffi(),
                flags.to_ffi(),
            )
        };

        NonNull::new(handle as *mut _)
            .map(|handle| Self { handle })
            .ok_or(Error::ReceiverCreateFailed)
    }

    /// Receives a frame of the specified type(s) - safe version.
    ///
    /// This is the recommended API that requires mutable access to the receiver.
    /// The borrow checker ensures you cannot hold multiple frames simultaneously,
    /// preventing use-after-invalidation bugs at compile time.
    ///
    /// Blocks until a frame is available or the timeout expires.
    ///
    /// # Arguments
    ///
    /// * `frame_types` - The frame types to receive. Can combine multiple types.
    /// * `timeout_ms` - Maximum time to wait in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(frame))` if a frame was received, `Ok(None)` if timed out.
    ///
    /// # Frame Lifetime
    ///
    /// The returned frame is valid until the next call to any receive method on this receiver.
    /// The frame's lifetime is tied to `&mut self`, ensuring exclusive access.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # let mut receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// // Receive and process frames in a loop
    /// loop {
    ///     if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
    ///         println!("Received frame with {} bytes", frame.data().len());
    ///         // Process frame here
    ///     } // frame dropped before next receive
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn receive(
        &mut self,
        frame_types: FrameType,
        timeout_ms: i32,
    ) -> Result<Option<MediaFrame<'_>>> {
        let ptr = unsafe {
            omt_sys::omt_receive(
                self.handle.as_ptr() as *mut _,
                frame_types.to_ffi(),
                timeout_ms,
            )
        };

        // SAFETY: The C API guarantees the frame data is valid until the next call to omt_receive.
        // The lifetime bound to &mut self ensures the frame cannot outlive this receiver instance
        // and prevents calling receive again while a frame exists (enforced by borrow checker).
        Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
    }

    /// Receives a frame of the specified type(s) - unsafe version.
    ///
    /// This is a performance-oriented API for advanced users who need concurrent access
    /// to other receiver methods (like statistics) while holding frames. It uses `&self`
    /// instead of `&mut self`, allowing more flexible usage patterns.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no `MediaFrame` returned from a previous call to
    /// `receive_unchecked` or `receive` on this receiver is still alive when calling
    /// this method. The underlying C library reuses the frame buffer, so holding multiple
    /// frames leads to undefined behavior (data corruption, crashes, or worse).
    ///
    /// This is a fundamental limitation of the C library that cannot be expressed in
    /// Rust's type system without using `&mut self`.
    ///
    /// # Arguments
    ///
    /// * `frame_types` - The frame types to receive. Can combine multiple types.
    /// * `timeout_ms` - Maximum time to wait in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(frame))` if a frame was received, `Ok(None)` if timed out.
    ///
    /// # Correct Usage Pattern
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// // CORRECT: Process and drop frame before next receive
    /// loop {
    ///     unsafe {
    ///         if let Some(frame) = receiver.receive_unchecked(FrameType::VIDEO, 1000)? {
    ///             process_frame(&frame);
    ///         } // frame dropped here
    ///     }
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    ///
    /// # Incorrect Usage (Undefined Behavior!)
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// // WRONG: Holding multiple frames
    /// unsafe {
    ///     let frame1 = receiver.receive_unchecked(FrameType::VIDEO, 1000)?;
    ///     let frame2 = receiver.receive_unchecked(FrameType::VIDEO, 1000)?;
    ///     // frame1's data is now INVALID! Accessing it is undefined behavior!
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    ///
    /// # Storing Frames
    ///
    /// If you need to store frames beyond the next receive call, clone them:
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # use std::sync::Arc;
    /// # let receiver = Arc::new(Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?);
    /// let mut frames = Vec::new();
    /// for _ in 0..10 {
    ///     unsafe {
    ///         if let Some(frame) = receiver.receive_unchecked(FrameType::VIDEO, 1000)? {
    ///             // Clone creates a deep copy - safe to store
    ///             frames.push(frame.clone());
    ///         }
    ///     }
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    ///
    /// **Warning:** Cloning copies all frame data (potentially ~64MB for 4K 16-bit RGBA).
    /// Use sparingly.
    ///
    /// # When to Use This
    ///
    /// Only use this method if you need to:
    /// - Call other receiver methods (like `get_video_statistics()`) concurrently while processing frames
    /// - Share the receiver across threads with `Arc` without `Mutex` overhead
    ///
    /// For typical single-threaded receive loops, prefer [`receive`](Self::receive).
    pub unsafe fn receive_unchecked(
        &self,
        frame_types: FrameType,
        timeout_ms: i32,
    ) -> Result<Option<MediaFrame<'_>>> {
        let ptr = unsafe {
            omt_sys::omt_receive(
                self.handle.as_ptr() as *mut _,
                frame_types.to_ffi(),
                timeout_ms,
            )
        };

        // SAFETY: Caller must ensure no previous frame from this receiver is still alive.
        // The C API reuses the frame buffer on each call to omt_receive.
        Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
    }

    /// Sends a metadata frame to the sender.
    ///
    /// Only metadata frames are supported for sending from a receiver.
    pub fn send_metadata(&self, _frame: &MediaFrame<'_>) -> Result<bool> {
        // TODO: Implement when we have a MediaFrame builder
        Ok(false)
    }

    /// Sets the tally state for this receiver.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags, Tally};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// let tally = Tally::new(true, false);
    /// receiver.set_tally(Tally::program_only());
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn set_tally(&self, tally: Tally) {
        let mut ffi_tally = tally.to_ffi();
        unsafe {
            omt_sys::omt_receive_settally(self.handle.as_ptr() as *mut _, &mut ffi_tally as *mut _);
        }
    }

    /// Gets the current tally state across all connections.
    ///
    /// Returns the last known state if timed out.
    pub fn get_tally(&self, timeout_ms: i32) -> Result<(Tally, bool)> {
        let mut ffi_tally = unsafe { std::mem::zeroed() };
        let changed = unsafe {
            omt_sys::omt_receive_gettally(
                self.handle.as_ptr() as *mut _,
                timeout_ms,
                &mut ffi_tally as *mut _,
            )
        };

        Ok((Tally::from_ffi(&ffi_tally), changed != 0))
    }

    /// Changes the receive flags dynamically.
    ///
    /// Changes apply from the next frame received.
    pub fn set_flags(&self, flags: ReceiveFlags) {
        unsafe {
            omt_sys::omt_receive_setflags(self.handle.as_ptr() as *mut _, flags.to_ffi());
        }
    }

    /// Sets the suggested quality level for this receiver.
    ///
    /// The sender will use the highest quality requested by any receiver.
    pub fn set_suggested_quality(&self, quality: Quality) {
        unsafe {
            omt_sys::omt_receive_setsuggestedquality(
                self.handle.as_ptr() as *mut _,
                quality.to_ffi(),
            );
        }
    }

    /// Retrieves information about the sender.
    ///
    /// Returns `None` if disconnected or no sender information is available.
    pub fn get_sender_information(&self) -> Result<Option<SenderInfo>> {
        let mut ffi_info: omt_sys::OMTSenderInfo = unsafe { std::mem::zeroed() };
        unsafe {
            omt_sys::omt_receive_getsenderinformation(
                self.handle.as_ptr() as *mut _,
                &mut ffi_info as *mut _,
            );
        }

        // Check if info is empty
        if ffi_info.ProductName[0] == 0 {
            Ok(None)
        } else {
            Ok(Some(SenderInfo::from_ffi(&ffi_info)?))
        }
    }

    /// Retrieves video statistics.
    pub fn get_video_statistics(&self) -> Statistics {
        let mut ffi_stats = unsafe { std::mem::zeroed() };
        unsafe {
            omt_sys::omt_receive_getvideostatistics(
                self.handle.as_ptr() as *mut _,
                &mut ffi_stats as *mut _,
            );
        }
        Statistics::from_ffi(&ffi_stats)
    }

    /// Retrieves audio statistics.
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut ffi_stats = unsafe { std::mem::zeroed() };
        unsafe {
            omt_sys::omt_receive_getaudiostatistics(
                self.handle.as_ptr() as *mut _,
                &mut ffi_stats as *mut _,
            );
        }
        Statistics::from_ffi(&ffi_stats)
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        unsafe {
            omt_sys::omt_receive_destroy(self.handle.as_ptr() as *mut _);
        }
    }
}

// SAFETY: The underlying C library is thread-safe
unsafe impl Send for Receiver {}
unsafe impl Sync for Receiver {}
