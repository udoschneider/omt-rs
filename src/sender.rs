//! High-level sender API for Open Media Transport (OMT).
//!
//! A sender publishes a source and streams video plus bi-directional metadata
//! over the OMT TCP transport. Metadata sent by receivers is retrieved via
//! `omt_send_receive` (as described in `libomt.h`). See
//! <https://github.com/openmediatransport>

use crate::ffi;
use crate::media_frame::MediaFrame;
use crate::receiver::{SenderInfo, Statistics, Tally};
use crate::types::{Address, Name, Quality, Timeout};
use crate::OmtError;
use std::ffi::CString;
use std::ptr::NonNull;

/// High-level sender handle. Drops cleanly by releasing the native instance.
pub struct Sender {
    handle: NonNull<ffi::omt_send_t>,
}

unsafe impl Send for Sender {}
unsafe impl Sync for Sender {}

impl Sender {
    /// Creates a new sender and publishes it on the network.
    ///
    /// When `quality` is `Default`, receivers can suggest a preferred quality.
    pub fn create(name: &Name, quality: Quality) -> Result<Self, OmtError> {
        let c_name = CString::new(name.as_str()).map_err(|_| OmtError::InvalidCString)?;
        let handle = unsafe { ffi::omt_send_create(c_name.as_ptr(), quality.into()) };
        let handle = NonNull::new(handle).ok_or(OmtError::NullHandle)?;
        Ok(Self { handle })
    }

    pub fn set_sender_info(&self, info: &SenderInfo) {
        let mut raw = info.to_ffi();
        unsafe { ffi::omt_send_setsenderinformation(self.handle.as_ptr(), &mut raw) };
    }

    /// Adds connection-level metadata (applies to new connections).
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
    pub fn add_connection_metadata<S: AsRef<str>>(&self, metadata: S) -> Result<(), OmtError> {
        let c_meta = CString::new(metadata.as_ref()).map_err(|_| OmtError::InvalidCString)?;
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
        unsafe { ffi::omt_send_clearconnectionmetadata(self.handle.as_ptr()) };
    }

    /// Redirects receivers to a new address, or clears the redirect when `None`.
    pub fn set_redirect(&self, new_address: Option<&Address>) -> Result<(), OmtError> {
        match new_address {
            Some(addr) => {
                let c_addr = CString::new(addr.as_str()).map_err(|_| OmtError::InvalidCString)?;
                unsafe { ffi::omt_send_setredirect(self.handle.as_ptr(), c_addr.as_ptr()) };
            }
            None => unsafe { ffi::omt_send_setredirect(self.handle.as_ptr(), std::ptr::null()) },
        }
        Ok(())
    }

    /// Returns the published sender address as `Address`, if available.
    pub fn get_address(&self) -> Option<Address> {
        let mut buf = vec![0u8; ffi::OMT_MAX_STRING_LENGTH];
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
        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
        Some(Address::from(cstr.to_string_lossy().to_string()))
    }

    /// Sends a prepared frame (video or metadata) to all connected receivers.
    pub fn send(&self, frame: &mut MediaFrame) -> i32 {
        unsafe { ffi::omt_send(self.handle.as_ptr(), frame.as_mut()) as i32 }
    }

    /// Returns the current number of connected receivers.
    pub fn connections(&self) -> i32 {
        unsafe { ffi::omt_send_connections(self.handle.as_ptr()) as i32 }
    }

    /// Receives metadata sent from receivers within the timeout.
    ///
    /// Call this in a loop to drive continuous receive since the iterator API
    /// was removed.
    ///
    /// Returned frames are valid until the next receive call on this sender
    /// (matching the `libomt.h` lifetime rules for `omt_send_receive`). The
    /// metadata payload is UTF-8 XML with a terminating null byte.
    pub fn receive_metadata(
        &mut self,
        timeout: Timeout,
    ) -> Result<Option<MediaFrame<'_>>, OmtError> {
        let frame_ptr =
            unsafe { ffi::omt_send_receive(self.handle.as_ptr(), timeout.as_millis_i32()) };
        if frame_ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(MediaFrame::new(unsafe { &*frame_ptr })))
        }
    }

    /// Retrieves tally state updates from connected receivers.
    pub fn get_tally(&self, timeout: Timeout, tally: &mut Tally) -> i32 {
        let mut raw = ffi::OMTTally {
            preview: 0,
            program: 0,
        };
        let result = unsafe {
            ffi::omt_send_gettally(self.handle.as_ptr(), timeout.as_millis_i32(), &mut raw) as i32
        };
        *tally = Tally::from(&raw);
        result
    }

    /// Returns video stream statistics for this sender.
    pub fn get_video_statistics(&self) -> Statistics {
        let mut stats = unsafe { std::mem::zeroed::<ffi::OMTStatistics>() };
        unsafe { ffi::omt_send_getvideostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }

    /// Returns audio stream statistics for this sender.
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut stats = unsafe { std::mem::zeroed::<ffi::OMTStatistics>() };
        unsafe { ffi::omt_send_getaudiostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        unsafe { ffi::omt_send_destroy(self.handle.as_ptr()) };
    }
}

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
