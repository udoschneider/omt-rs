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
    ///   or a URL in the format `omt://hostname:port`
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
    /// Returns `Ok(Some(frame))` if a frame was received, `Ok(None)` if none
    /// arrived before the timeout.
    ///
    /// Note on errors: the underlying C API reports "no frame" with a null
    /// pointer and exposes no separate error channel — it catches and logs any
    /// internal error itself, then also returns null. `Ok(None)` therefore means
    /// "no frame this call" and cannot be distinguished from an internal
    /// failure. A persistent `None` across many calls usually indicates the
    /// sender is unavailable rather than a momentary timeout.
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
    /// The caller must uphold **both** of the following invariants:
    ///
    /// 1. No `MediaFrame` returned from a previous call to `receive_unchecked` or
    ///    `receive` on this receiver may be alive when calling this method. The
    ///    underlying C library reuses the frame buffer, so holding multiple frames
    ///    leads to undefined behavior (data corruption, crashes, or worse).
    ///
    /// 2. Calls to `receive_unchecked`/`receive` on the same receiver must never
    ///    overlap in time across threads. The C library's marshalling layer frees
    ///    the previous frame buffer and allocates a new one without any locking, so
    ///    two concurrent receive calls on one instance race and can double-free.
    ///    Other `&self` methods (statistics, tally, flags) may be called
    ///    concurrently; only the receive calls themselves must be serialized.
    ///
    /// These are fundamental limitations of the C library that cannot be expressed
    /// in Rust's type system without using `&mut self`.
    ///
    /// # Arguments
    ///
    /// * `frame_types` - The frame types to receive. Can combine multiple types.
    /// * `timeout_ms` - Maximum time to wait in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(frame))` if a frame was received, `Ok(None)` if none
    /// arrived before the timeout.
    ///
    /// Note on errors: the underlying C API reports "no frame" with a null
    /// pointer and exposes no separate error channel — it catches and logs any
    /// internal error itself, then also returns null. `Ok(None)` therefore means
    /// "no frame this call" and cannot be distinguished from an internal
    /// failure. A persistent `None` across many calls usually indicates the
    /// sender is unavailable rather than a momentary timeout.
    ///
    /// # Correct Usage Pattern
    ///
    /// ```no_run
    /// # use omt::{MediaFrame, Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # fn process_frame(frame: &MediaFrame<'_>) {}
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
    /// Storing frames beyond the next receive call does **not** require this
    /// method — [`MediaFrame::to_static`] deep-copies a frame out of the
    /// receiver's borrow and works on the safe [`receive`](Self::receive) path
    /// too. See its documentation for the example.
    ///
    /// **Warning:** Copying duplicates all frame data (potentially ~64MB for 4K
    /// 16-bit RGBA). Use sparingly.
    ///
    /// # When to Use This
    ///
    /// Only use this method if you need to:
    /// - Call other receiver methods (like `get_video_statistics()`) from another
    ///   thread while a single receive thread processes frames
    ///
    /// When sharing the receiver across threads via `Arc`, the receive calls
    /// themselves must still be serialized (see the second safety invariant above) —
    /// e.g. keep all receiving on one thread, or guard it with a `Mutex`. Sharing
    /// via `Arc` without a `Mutex` is only sound for the non-receiving methods.
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

        // SAFETY: Per this function's safety contract, the caller guarantees no
        // previously returned frame is still alive and that receive calls on this
        // instance never overlap. The C API reuses (and reallocates) the frame
        // buffer on each call to omt_receive.
        Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
    }

    /// Sends a metadata frame back to the connected sender.
    ///
    /// Only metadata frames are supported on this channel; passing a video or
    /// audio frame returns [`Error::InvalidParameter`] without touching the C
    /// library. Build a frame with [`MetadataFrameBuilder`](crate::MetadataFrameBuilder)
    /// and convert it via [`OwnedMediaFrame::as_media_frame`](crate::OwnedMediaFrame::as_media_frame).
    ///
    /// Returns `Ok(true)` if the frame was accepted for sending, `Ok(false)`
    /// otherwise (e.g. no sender is currently connected).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags, MetadataFrameBuilder};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::METADATA, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// let frame = MetadataFrameBuilder::new()
    ///     .metadata("<metadata>hello</metadata>")
    ///     .build()?;
    /// receiver.send_metadata(&frame.as_media_frame())?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn send_metadata(&self, frame: &MediaFrame<'_>) -> Result<bool> {
        if frame.frame_type() != FrameType::METADATA {
            return Err(Error::InvalidParameter {
                parameter: "frame".to_string(),
                reason: "only metadata frames can be sent from a receiver".to_string(),
            });
        }

        // SAFETY: `omt_receive_send` takes a non-const `OMTMediaFrame*` but only
        // reads it — it marshals the frame in via `OMTMediaFrame.FromIntPtr` (a
        // `Marshal.PtrToStructure`, i.e. a pure copy out of our memory) and
        // sends from that managed copy, so the `&` -> `*mut` cast never results
        // in a write through the shared reference. The handle is a valid live
        // instance.
        let result = unsafe {
            omt_sys::omt_receive_send(
                self.handle.as_ptr() as *mut _,
                frame.as_ffi() as *const _ as *mut _,
            )
        };
        Ok(result != 0)
    }

    /// Sets the tally state for this receiver.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags, Tally};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
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

        // "No information" is reported as an all-empty struct; any populated
        // field means real information even if the product name is blank.
        let is_empty = ffi_info.ProductName[0] == 0
            && ffi_info.Manufacturer[0] == 0
            && ffi_info.Version[0] == 0;

        if is_empty {
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

// SAFETY: A Receiver owns an opaque C handle that may be moved between threads (Send).
// Sync is sound for the safe API: every `&self` method other than the `unsafe`
// receive_unchecked delegates to internally-synchronized operations in the C library.
// The one unsynchronized path — the reused receive frame buffer — is reachable only
// through `receive` (which takes `&mut self`) or `receive_unchecked` (whose safety
// contract forbids concurrent receive calls on the same instance).
unsafe impl Send for Receiver {}
unsafe impl Sync for Receiver {}
