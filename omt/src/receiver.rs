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

    /// Receives a frame of the specified type(s).
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
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
    /// # let receiver = Receiver::new("omt://localhost:6400", FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
    /// // Receive any video frame, wait up to 1 second
    /// if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
    ///     println!("Received frame");
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn receive(&self, frame_types: FrameType, timeout_ms: i32) -> Result<Option<MediaFrame>> {
        let ptr = unsafe {
            omt_sys::omt_receive(
                self.handle.as_ptr() as *mut _,
                frame_types.to_ffi(),
                timeout_ms,
            )
        };

        Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
    }

    /// Sends a metadata frame to the sender.
    ///
    /// Only metadata frames are supported for sending from a receiver.
    pub fn send_metadata(&self, _frame: &MediaFrame) -> Result<bool> {
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
