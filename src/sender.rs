//! High-level sender API for Open Media Transport (OMT).
//!
//! A sender publishes a source and streams video plus bi-directional metadata
//! over the OMT TCP transport. Metadata sent by receivers is retrieved via
//! `omt_send_receive` (as described in `libomt.h`). See
//! <https://github.com/openmediatransport>

use crate::ffi;
use crate::media_frame::MediaFrame;
use crate::types::{Address, Name, Quality, SenderInfo, Statistics, Tally, Timeout};
use crate::Error;
use std::ffi::CString;
use std::ptr::NonNull;

/// High-level sender handle. Drops cleanly by releasing the native instance.
pub struct Sender {
    handle: NonNull<ffi::omt_send_t>,
}

impl Sender {
    /// Creates a new instance of the OMT Sender and publishes it on the network.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the source (not including hostname)
    /// * `quality` - The quality to use for video encoding. If `Quality::Default`, this can be
    ///   automatically adjusted based on receiver requirements.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidCString` if the name contains null bytes.
    /// Returns `Error::NullHandle` if the underlying C library fails to create the sender.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// // Create sender with default quality (can be adjusted by receivers)
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    /// ```
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// // Create sender with fixed high quality
    /// let name = Name::new("Studio Camera");
    /// let sender = Sender::create(&name, Quality::High).unwrap();
    /// ```
    pub fn create(name: &Name, quality: Quality) -> Result<Self, Error> {
        let c_name = CString::new(name.as_str()).map_err(|_| Error::InvalidCString)?;
        // SAFETY: FFI call to C library. The c_name pointer is valid for the duration
        // of the call, and all parameters are properly constructed C-compatible types.
        let handle = unsafe { ffi::omt_send_create(c_name.as_ptr(), quality.into()) };
        let handle = NonNull::new(handle).ok_or(Error::NullHandle)?;
        Ok(Self { handle })
    }

    /// Optionally sets information describing the sender.
    ///
    /// This allows you to provide metadata about the sender such as product name,
    /// manufacturer, and version information that receivers can query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality, SenderInfo};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// let info = SenderInfo::new()
    ///     .with_product_name("Professional Camera")
    ///     .with_manufacturer("ACME Corp")
    ///     .with_version("1.0.0");
    ///
    /// sender.set_sender_info(&info);
    /// ```
    pub fn set_sender_info(&self, info: &SenderInfo) {
        let mut raw = ffi::OMTSenderInfo::from(info);
        // SAFETY: FFI call with valid handle and mutable sender info struct pointer.
        unsafe { ffi::omt_send_setsenderinformation(self.handle.as_ptr(), &mut raw) };
    }

    /// Adds to the list of metadata that is sent immediately upon a new connection by a receiver.
    ///
    /// This metadata is sent immediately upon a new connection by a receiver, and is also
    /// sent to any currently connected receivers. Metadata should be UTF-8 encoded XML.
    ///
    /// # Null Terminator Handling
    ///
    /// Although `libomt.h` explicitly documents that metadata strings must *include*
    /// the null terminator, this high-level Rust wrapper handles that automatically.
    /// You should pass metadata strings that do *not* include a null character - the
    /// wrapper will add the null terminator behind the scenes when creating the C string.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata contains null bytes (null terminators are added automatically).
    ///
    /// # Examples
    ///
    /// ## Web management interface
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("My Camera");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// sender.add_connection_metadata(r#"<OMTWeb URL="http://192.168.1.100/" />"#).unwrap();
    /// ```
    ///
    /// ## PTZ control metadata
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("PTZ Camera");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// // VISCA over IP
    /// sender.add_connection_metadata(
    ///     r#"<OMTPTZ Protocol="VISCAoverIP" URL="visca://192.168.1.100:52381" />"#
    /// ).unwrap();
    /// ```
    ///
    /// ## Multiple metadata entries
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("Multi Camera");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// // Add web interface
    /// sender.add_connection_metadata(r#"<OMTWeb URL="http://192.168.1.100/" />"#).unwrap();
    ///
    /// // Add PTZ control
    /// sender.add_connection_metadata(
    ///     r#"<OMTPTZ Protocol="VISCAoverIP" URL="visca://192.168.1.100:52381" />"#
    /// ).unwrap();
    /// ```
    pub fn add_connection_metadata<S: AsRef<str>>(&self, metadata: S) -> Result<(), Error> {
        let c_meta = CString::new(metadata.as_ref()).map_err(|_| Error::InvalidCString)?;
        // SAFETY: FFI call with valid handle and C string pointer that remains valid for call duration.
        unsafe { ffi::omt_send_addconnectionmetadata(self.handle.as_ptr(), c_meta.as_ptr()) };
        Ok(())
    }

    /// Clears the list of metadata that is sent immediately upon a new connection by a receiver.
    ///
    /// This removes all metadata previously added with `add_connection_metadata`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("My Sender");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// sender.add_connection_metadata(r#"<OMTWeb URL="http://192.168.1.100/" />"#).unwrap();
    /// sender.clear_connection_metadata();
    /// ```
    pub fn clear_connection_metadata(&self) {
        // SAFETY: FFI call with valid handle.
        unsafe { ffi::omt_send_clearconnectionmetadata(self.handle.as_ptr()) };
    }

    /// Redirects receivers to connect to a different address.
    ///
    /// This is used to create a "virtual source" that can be dynamically switched as needed.
    /// This is useful for scenarios where a receiver needs to be changed remotely.
    ///
    /// # Parameters
    ///
    /// * `new_address` - The new address to redirect to, or `None` to disable redirect
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidCString` if the address contains null bytes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality, Address};
    ///
    /// let name = Name::new("Virtual Camera");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// // Redirect to a different camera
    /// let new_addr = Address::from("camera2.local");
    /// sender.set_redirect(Some(&new_addr)).unwrap();
    ///
    /// // Clear redirect
    /// sender.set_redirect(None).unwrap();
    /// ```
    pub fn set_redirect(&self, new_address: Option<&Address>) -> Result<(), Error> {
        match new_address {
            Some(addr) => {
                let c_addr = CString::new(addr.as_str()).map_err(|_| Error::InvalidCString)?;
                // SAFETY: FFI call with valid handle and C string pointer.
                unsafe { ffi::omt_send_setredirect(self.handle.as_ptr(), c_addr.as_ptr()) };
            }
            None => {
                // SAFETY: FFI call with valid handle and null pointer to clear redirect.
                unsafe { ffi::omt_send_setredirect(self.handle.as_ptr(), std::ptr::null()) }
            }
        }
        Ok(())
    }

    /// Retrieves the discovery address in the format "HOSTNAME (NAME)".
    ///
    /// Returns the sender's published address that receivers use for discovery.
    ///
    /// # Returns
    ///
    /// Returns `Some(Address)` containing the UTF-8 encoded discovery address,
    /// or `None` if the address could not be retrieved.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// if let Some(address) = sender.get_address() {
    ///     println!("Sender published at: {}", address.as_str());
    /// }
    /// ```
    pub fn get_address(&self) -> Option<Address> {
        let mut buf = vec![0u8; ffi::OMT_MAX_STRING_LENGTH];
        // SAFETY: FFI call with valid handle and mutable buffer of sufficient size.
        let len = unsafe {
            ffi::omt_send_getaddress(
                self.handle.as_ptr(),
                buf.as_mut_ptr() as *mut std::ffi::c_char,
                buf.len() as i32,
            )
        };
        if len <= 0 {
            return None;
        }
        // SAFETY: The C library has written a null-terminated string to our buffer.
        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Some(Address::from(cstr.to_string_lossy().to_string()))
    }

    /// Sends a frame to all currently connected receivers.
    ///
    /// # Supported Formats
    ///
    /// * **Video**: `UYVY`, `YUY2`, `NV12`, `YV12`, `BGRA`, `UYVA`, `VMX1`
    ///   - Note: `BGRA` will be treated as `BGRX` and `UYVA` as `UYVY` where alpha flags are not set
    /// * **Audio**: Planar 32-bit floating point audio
    /// * **Metadata**: UTF-8 encoded XML
    ///
    /// # Returns
    ///
    /// Returns the number of bytes sent, or a negative value on error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality, MediaFrame};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// // Send a video frame (assuming frame is prepared)
    /// // let mut frame = MediaFrame::new_video(...);
    /// // let bytes_sent = sender.send(&mut frame);
    /// ```
    pub fn send(&self, frame: &mut MediaFrame) -> i32 {
        // SAFETY: FFI call with valid handle and mutable frame pointer.
        unsafe { ffi::omt_send(self.handle.as_ptr(), frame.as_mut()) as i32 }
    }

    /// Returns the total number of connections to this sender.
    ///
    /// Note: Receivers establish one connection for video/metadata and a second for audio,
    /// so the connection count may be twice the number of actual receivers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// let conn_count = sender.connections();
    /// println!("Connected: {} connections", conn_count);
    /// ```
    pub fn connections(&self) -> i32 {
        // SAFETY: FFI call with valid handle.
        unsafe { ffi::omt_send_connections(self.handle.as_ptr()) as i32 }
    }

    /// Receives any available metadata from the buffer, or waits if empty.
    ///
    /// This function receives metadata sent from connected receivers. If metadata
    /// is available in the buffer, it returns immediately. Otherwise, it waits up
    /// to the specified timeout for metadata to arrive.
    ///
    /// # Lifetime
    ///
    /// The returned `MediaFrame` is valid until the next call to `receive_metadata`
    /// on this sender (matching the `libomt.h` lifetime rules for `omt_send_receive`).
    /// The metadata payload is UTF-8 encoded XML with a terminating null byte.
    ///
    /// # Parameters
    ///
    /// * `timeout` - Maximum time to wait for metadata
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MediaFrame))` - Metadata frame received
    /// * `Ok(None)` - Timed out waiting for metadata
    /// * `Err(Error)` - Error occurred
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality, Timeout};
    /// use std::time::Duration;
    ///
    /// let name = Name::new("Camera 1");
    /// let mut sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// // Poll for metadata with 100ms timeout
    /// let timeout = Timeout::from(Duration::from_millis(100));
    /// if let Ok(Some(frame)) = sender.receive_metadata(timeout) {
    ///     println!("Received metadata");
    /// }
    /// ```
    pub fn receive_metadata(&self, timeout: Timeout) -> Result<Option<MediaFrame<'_>>, Error> {
        // SAFETY: FFI call with valid handle and timeout value.
        let frame_ptr =
            unsafe { ffi::omt_send_receive(self.handle.as_ptr(), timeout.as_millis_i32()) };
        if frame_ptr.is_null() {
            Ok(None)
        } else {
            // SAFETY: The C library returns a valid pointer to a frame that remains
            // valid until the next receive call or until the sender is destroyed.
            Ok(Some(unsafe { &*frame_ptr }.into()))
        }
    }

    /// Receives the current tally state across all connections to a sender.
    ///
    /// If this function times out, the last known tally state will be received.
    ///
    /// # Parameters
    ///
    /// * `timeout` - Maximum time to wait for tally updates
    /// * `tally` - Mutable reference to `Tally` struct that will be updated
    ///
    /// # Returns
    ///
    /// * `1` - Tally state changed
    /// * `0` - Timed out or tally didn't change
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality, Timeout, Tally};
    /// use std::time::Duration;
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// let mut tally = Tally::default();
    /// let timeout = Timeout::from(Duration::from_millis(100));
    /// let changed = sender.get_tally(timeout, &mut tally);
    ///
    /// if changed == 1 {
    ///     println!("Tally state changed - Preview: {}, Program: {}",
    ///              tally.preview, tally.program);
    /// }
    /// ```
    pub fn get_tally(&self, timeout: Timeout, tally: &mut Tally) -> i32 {
        let mut raw = ffi::OMTTally {
            preview: 0,
            program: 0,
        };
        // SAFETY: FFI call with valid handle and mutable tally struct pointer.
        let result = unsafe {
            ffi::omt_send_gettally(self.handle.as_ptr(), timeout.as_millis_i32(), &mut raw) as i32
        };
        *tally = Tally::from(&raw);
        result
    }

    /// Retrieves statistics for the video stream.
    ///
    /// Returns detailed statistics about the video transmission including bytes sent,
    /// frames transmitted, frames dropped, and codec performance metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// let stats = sender.get_video_statistics();
    /// println!("Video frames sent: {}", stats.frames);
    /// println!("Bytes sent: {}", stats.bytes_sent);
    /// ```
    pub fn get_video_statistics(&self) -> Statistics {
        let mut stats = ffi::OMTStatistics::default();
        // SAFETY: FFI call with valid handle and mutable statistics struct pointer.
        unsafe { ffi::omt_send_getvideostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }

    /// Retrieves statistics for the audio stream.
    ///
    /// Returns detailed statistics about the audio transmission including bytes sent,
    /// frames transmitted, frames dropped, and codec performance metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Sender, Name, Quality};
    ///
    /// let name = Name::new("Camera 1");
    /// let sender = Sender::create(&name, Quality::Default).unwrap();
    ///
    /// let stats = sender.get_audio_statistics();
    /// println!("Audio frames sent: {}", stats.frames);
    /// println!("Bytes sent: {}", stats.bytes_sent);
    /// ```
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut stats = ffi::OMTStatistics::default();
        // SAFETY: FFI call with valid handle and mutable statistics struct pointer.
        unsafe { ffi::omt_send_getaudiostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }
}

impl Drop for Sender {
    /// Destroys the sender instance created with `create`.
    ///
    /// Make sure any threads currently accessing sender functions with this instance
    /// are closed before dropping.
    fn drop(&mut self) {
        // SAFETY: FFI call to destroy the sender handle. This is called once when
        // the sender is dropped, and the handle is not used after this call.
        unsafe { ffi::omt_send_destroy(self.handle.as_ptr()) };
    }
}

// SAFETY: The OMT C library's sender handle is an opaque pointer that can be
// safely sent between threads and accessed from multiple threads. The underlying
// C library uses internal synchronization for thread safety.
unsafe impl Send for Sender {}

// SAFETY: The OMT C library's sender handle can be safely shared between threads.
// All sender operations use the C library's internal synchronization mechanisms.
unsafe impl Sync for Sender {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_connection_metadata_rejects_null_chars() {
        // Create a sender
        let name = Name::new("Test Sender");
        let sender = Sender::create(&name, Quality::Default).unwrap();

        // Try to add metadata with null byte - should fail
        let result = sender.add_connection_metadata("metadata\0with_null");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_connection_metadata_accepts_valid_string() {
        // Create a sender
        let name = Name::new("Test Sender");
        let sender = Sender::create(&name, Quality::Default).unwrap();

        // Add valid metadata - should succeed
        let result = sender.add_connection_metadata("<test>valid metadata</test>");
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_connection_metadata_accepts_asref_str() {
        // Create a sender
        let name = Name::new("Test Sender");
        let sender = Sender::create(&name, Quality::Default).unwrap();

        // Test with &str
        assert!(sender.add_connection_metadata("<test>str</test>").is_ok());

        // Test with String
        let metadata = String::from("<test>string</test>");
        assert!(sender.add_connection_metadata(&metadata).is_ok());

        // Test with owned String
        assert!(sender.add_connection_metadata(metadata).is_ok());
    }
}
