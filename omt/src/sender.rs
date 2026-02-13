//! OMT sender for broadcasting media streams.

use crate::MAX_STRING_LENGTH;
use crate::error::{Error, Result};
use crate::frame::MediaFrame;
use crate::statistics::Statistics;
use crate::tally::Tally;
use crate::types::{Quality, SenderInfo};
use std::ffi::CString;
use std::ptr::NonNull;

/// Sender for broadcasting media streams to receivers.
///
/// The sender manages network connections, encoding, and transmission
/// of video, audio, and metadata to all connected receivers.
pub struct Sender {
    handle: NonNull<omt_sys::omt_send_t>,
}

impl Sender {
    /// Creates a new sender instance.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the source (not including hostname)
    /// * `quality` - Initial encoding quality. Use `Quality::Default` for auto-adjustment
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Quality};
    ///
    /// let sender = Sender::new("My Camera", Quality::High)?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn new(name: &str, quality: Quality) -> Result<Self> {
        let c_name = CString::new(name)?;

        let handle = unsafe { omt_sys::omt_send_create(c_name.as_ptr(), quality.to_ffi()) };

        NonNull::new(handle as *mut _)
            .map(|handle| Self { handle })
            .ok_or(Error::SenderCreateFailed)
    }

    /// Sets information describing this sender.
    ///
    /// This information is sent to receivers upon connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Sender, Quality, SenderInfo};
    /// # let sender = Sender::new("My Camera", Quality::High)?;
    /// let info = SenderInfo::new(
    ///     "My Product".to_string(),
    ///     "ACME Corp".to_string(),
    ///     "1.0.0".to_string(),
    /// );
    /// sender.set_sender_information(&info)?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn set_sender_information(&self, info: &SenderInfo) -> Result<()> {
        let mut ffi_info = info.to_ffi()?;
        unsafe {
            omt_sys::omt_send_setsenderinformation(
                self.handle.as_ptr() as *mut _,
                &mut ffi_info as *mut _,
            );
        }
        Ok(())
    }

    /// Adds metadata that is sent immediately upon new receiver connections.
    ///
    /// This metadata is also immediately sent to currently connected receivers.
    ///
    /// # Arguments
    ///
    /// * `metadata` - UTF-8 encoded XML metadata string
    pub fn add_connection_metadata(&self, metadata: &str) -> Result<()> {
        let c_metadata = CString::new(metadata)?;
        unsafe {
            omt_sys::omt_send_addconnectionmetadata(
                self.handle.as_ptr() as *mut _,
                c_metadata.as_ptr(),
            );
        }
        Ok(())
    }

    /// Clears all connection metadata.
    pub fn clear_connection_metadata(&self) {
        unsafe {
            omt_sys::omt_send_clearconnectionmetadata(self.handle.as_ptr() as *mut _);
        }
    }

    /// Sets a redirect address to inform receivers to connect elsewhere.
    ///
    /// This creates a "virtual source" that can be dynamically switched.
    ///
    /// # Arguments
    ///
    /// * `address` - New address, or `None` to disable redirect
    pub fn set_redirect(&self, address: Option<&str>) -> Result<()> {
        use std::ptr;

        if let Some(addr) = address {
            let c_addr = CString::new(addr)?;
            unsafe {
                omt_sys::omt_send_setredirect(self.handle.as_ptr() as *mut _, c_addr.as_ptr());
            }
        } else {
            unsafe {
                omt_sys::omt_send_setredirect(self.handle.as_ptr() as *mut _, ptr::null());
            }
        }
        Ok(())
    }

    /// Retrieves the discovery address in the format "HOSTNAME (NAME)".
    pub fn get_address(&self) -> Result<String> {
        let mut buffer = vec![0i8; MAX_STRING_LENGTH];
        let len = unsafe {
            omt_sys::omt_send_getaddress(
                self.handle.as_ptr() as *mut _,
                buffer.as_mut_ptr(),
                MAX_STRING_LENGTH as i32,
            )
        };

        if len <= 0 {
            return Ok(String::new());
        }

        let bytes: Vec<u8> = buffer[..len as usize].iter().map(|&b| b as u8).collect();
        String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
    }

    /// Sends a frame to all connected receivers.
    ///
    /// Supports video, audio, and metadata frames.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use omt::{Sender, Quality};
    /// # let sender = Sender::new("My Camera", Quality::High)?;
    /// // Send a frame
    /// // let frame = ...; // Create frame
    /// // sender.send(&frame)?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn send(&self, frame: &MediaFrame) -> Result<bool> {
        let result = unsafe {
            omt_sys::omt_send(
                self.handle.as_ptr() as *mut _,
                frame.as_ffi() as *const _ as *mut _,
            )
        };
        Ok(result != 0)
    }

    /// Returns the total number of connections to this sender.
    ///
    /// Note: Receivers establish one connection for video/metadata and
    /// a second for audio.
    pub fn connections(&self) -> i32 {
        unsafe { omt_sys::omt_send_connections(self.handle.as_ptr() as *mut _) }
    }

    /// Receives metadata from receivers.
    ///
    /// Blocks until metadata is available or timeout expires.
    pub fn receive_metadata(&self, timeout_ms: i32) -> Result<Option<MediaFrame>> {
        let ptr = unsafe { omt_sys::omt_send_receive(self.handle.as_ptr() as *mut _, timeout_ms) };

        Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
    }

    /// Gets the current tally state across all connections.
    ///
    /// Returns the last known state if timed out.
    pub fn get_tally(&self, timeout_ms: i32) -> Result<(Tally, bool)> {
        let mut ffi_tally = unsafe { std::mem::zeroed() };
        let changed = unsafe {
            omt_sys::omt_send_gettally(
                self.handle.as_ptr() as *mut _,
                timeout_ms,
                &mut ffi_tally as *mut _,
            )
        };

        Ok((Tally::from_ffi(&ffi_tally), changed != 0))
    }

    /// Retrieves video statistics.
    pub fn get_video_statistics(&self) -> Statistics {
        let mut ffi_stats = unsafe { std::mem::zeroed() };
        unsafe {
            omt_sys::omt_send_getvideostatistics(
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
            omt_sys::omt_send_getaudiostatistics(
                self.handle.as_ptr() as *mut _,
                &mut ffi_stats as *mut _,
            );
        }
        Statistics::from_ffi(&ffi_stats)
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        unsafe {
            omt_sys::omt_send_destroy(self.handle.as_ptr() as *mut _);
        }
    }
}

// SAFETY: The underlying C library is thread-safe
unsafe impl Send for Sender {}
unsafe impl Sync for Sender {}
