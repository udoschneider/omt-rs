//! High-level receiver API for Open Media Transport (OMT).
//!
//! A receiver connects to a published sender and pulls video/audio/metadata
//! over the OMT TCP transport. The address is typically discovered via DNS-SD
//! (Bonjour/Avahi) or a discovery server (as described by `libomt.h`).
//! See <https://github.com/openmediatransport> for protocol background.

use crate::ffi;
use crate::ffi_utils::c_char_array_to_string;
use crate::media_frame::MediaFrame;
use crate::types::{Address, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, Timeout};
use crate::OmtError;
use std::ffi::CString;
use std::ptr::NonNull;

#[derive(Clone, Debug, Default)]
/// On-air tally state reported by the sender (preview/program).
pub struct Tally {
    pub preview: bool,
    pub program: bool,
}

#[derive(Clone, Debug, Default)]
/// Optional metadata describing the sender device/software.
pub struct SenderInfo {
    pub product_name: String,
    pub manufacturer: String,
    pub version: String,
    pub reserved1: String,
    pub reserved2: String,
    pub reserved3: String,
}

impl SenderInfo {
    /// Converts this `SenderInfo` to the FFI representation.
    pub(crate) fn to_ffi(&self) -> ffi::OMTSenderInfo {
        use crate::ffi_utils::write_c_char_array;

        let mut raw = ffi::OMTSenderInfo {
            ProductName: [0; ffi::OMT_MAX_STRING_LENGTH],
            Manufacturer: [0; ffi::OMT_MAX_STRING_LENGTH],
            Version: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved1: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved2: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved3: [0; ffi::OMT_MAX_STRING_LENGTH],
        };

        write_c_char_array(&mut raw.ProductName, &self.product_name);
        write_c_char_array(&mut raw.Manufacturer, &self.manufacturer);
        write_c_char_array(&mut raw.Version, &self.version);
        write_c_char_array(&mut raw.Reserved1, &self.reserved1);
        write_c_char_array(&mut raw.Reserved2, &self.reserved2);
        write_c_char_array(&mut raw.Reserved3, &self.reserved3);

        raw
    }
}

#[derive(Clone, Debug, Default)]
/// Transport and codec statistics for audio or video streams.
pub struct Statistics {
    pub bytes_sent: i64,
    pub bytes_received: i64,
    pub bytes_sent_since_last: i64,
    pub bytes_received_since_last: i64,
    pub frames: i64,
    pub frames_since_last: i64,
    pub frames_dropped: i64,
    pub codec_time: i64,
    pub codec_time_since_last: i64,
    pub reserved1: i64,
    pub reserved2: i64,
    pub reserved3: i64,
    pub reserved4: i64,
    pub reserved5: i64,
    pub reserved6: i64,
    pub reserved7: i64,
}

fn sender_info_from_ffi(info: &ffi::OMTSenderInfo) -> Option<SenderInfo> {
    let product_name = c_char_array_to_string(&info.ProductName[..]);
    let manufacturer = c_char_array_to_string(&info.Manufacturer[..]);
    let version = c_char_array_to_string(&info.Version[..]);
    let reserved1 = c_char_array_to_string(&info.Reserved1[..]);
    let reserved2 = c_char_array_to_string(&info.Reserved2[..]);
    let reserved3 = c_char_array_to_string(&info.Reserved3[..]);

    let has_any = !product_name.is_empty()
        || !manufacturer.is_empty()
        || !version.is_empty()
        || !reserved1.is_empty()
        || !reserved2.is_empty()
        || !reserved3.is_empty();

    if has_any {
        Some(SenderInfo {
            product_name,
            manufacturer,
            version,
            reserved1,
            reserved2,
            reserved3,
        })
    } else {
        None
    }
}

fn tally_to_ffi(tally: &Tally) -> ffi::OMTTally {
    ffi::OMTTally {
        preview: if tally.preview { 1 } else { 0 },
        program: if tally.program { 1 } else { 0 },
    }
}

/// High-level receiver handle. Drops cleanly by releasing the native instance.
pub struct Receiver {
    handle: NonNull<ffi::omt_receive_t>,
}

unsafe impl Send for Receiver {}
unsafe impl Sync for Receiver {}

impl Receiver {
    /// Connects to a sender address and creates a receiver instance.
    ///
    /// `address` uses the `Address` newtype to distinguish sender addresses from other strings.
    /// `frame_types` selects which streams to receive, `format` controls the
    /// preferred pixel formats, and `flags` toggles optional behaviors such as
    /// preview or compressed delivery.
    pub fn create(
        address: &Address,
        frame_types: FrameType,
        format: PreferredVideoFormat,
        flags: ReceiveFlags,
    ) -> Result<Self, OmtError> {
        let c_address = CString::new(address.as_str()).map_err(|_| OmtError::InvalidCString)?;
        let handle = unsafe {
            ffi::omt_receive_create(
                c_address.as_ptr(),
                frame_types.into(),
                format.into(),
                i32::from(flags),
            )
        };
        let handle = NonNull::new(handle).ok_or(OmtError::NullHandle)?;
        Ok(Self { handle })
    }

    /// Receives the next frame of the requested type within the timeout.
    ///
    /// Call this in a loop to drive continuous receive since the iterator API
    /// was removed.
    ///
    /// Returned frames are valid until the next `receive` call on this receiver
    /// (matching the `libomt.h` lifetime rules for `omt_receive`). Timestamps are
    /// in OMT ticks (10,000,000 per second). Metadata frames carry UTF-8 XML with
    /// a terminating null byte.
    ///
    /// # Examples
    ///
    /// ## Receive video frames
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};
    ///
    /// let address = Address::new("HOST (Sender Name)");
    /// let mut receiver = Receiver::create(
    ///     &address,
    ///     FrameType::Video,
    ///     PreferredVideoFormat::UYVYorBGRA,
    ///     ReceiveFlags::NONE,
    /// ).unwrap();
    ///
    /// if let Ok(Some(frame)) = receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
    ///     println!("Received {}x{} frame", frame.width(), frame.height());
    /// }
    /// ```
    ///
    /// ## Receive metadata frames
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};
    ///
    /// # let address = Address::new("HOST (Sender Name)");
    /// # let mut receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// if let Ok(Some(frame)) = receiver.receive(FrameType::Metadata, Timeout::from_millis(100)) {
    ///     if let Some(metadata) = frame.xml_data() {
    ///         println!("Received metadata: {}", metadata);
    ///     }
    /// }
    /// ```
    pub fn receive(
        &mut self,
        frame_types: FrameType,
        timeout: Timeout,
    ) -> Result<Option<MediaFrame<'_>>, OmtError> {
        let frame_ptr = unsafe {
            ffi::omt_receive(
                self.handle.as_ptr(),
                frame_types.into(),
                timeout.as_millis_i32(),
            )
        };
        if frame_ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(MediaFrame::new(unsafe { &*frame_ptr })))
        }
    }

    /// Sends a metadata frame back to the sender (bi-directional metadata channel).
    ///
    /// The frame should be a metadata frame created with `MediaFrame::metadata()`.
    /// If a non-metadata frame is passed, a warning will be logged and the frame
    /// type will be used as-is.
    ///
    /// # Examples
    ///
    /// ## Send simple metadata
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, MediaFrame};
    ///
    /// let address = Address::new("HOST (Sender Name)");
    /// let receiver = Receiver::create(
    ///     &address,
    ///     FrameType::Video,
    ///     PreferredVideoFormat::UYVYorBGRA,
    ///     ReceiveFlags::NONE,
    /// ).unwrap();
    ///
    /// let mut metadata = MediaFrame::metadata("<status>ready</status>", -1).unwrap();
    /// receiver.send_metadata(&mut metadata).unwrap();
    /// ```
    ///
    /// ## Send PTZ command
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, MediaFrame};
    ///
    /// # let address = Address::new("HOST (Sender Name)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// // Send PTZ VISCA command to sender
    /// let mut ptz_cmd = MediaFrame::metadata(
    ///     r#"<OMTPTZ Protocol="VISCA" Sequence="22" Command="8101040700FF" />"#,
    ///     -1
    /// ).unwrap();
    /// receiver.send_metadata(&mut ptz_cmd).unwrap();
    /// ```
    pub fn send_metadata(&self, frame: &mut MediaFrame) -> Result<i32, OmtError> {
        // Check if frame is a metadata frame and log warning if not
        let frame_ref = frame.as_mut();
        if frame_ref.Type != ffi::OMTFrameType::Metadata {
            log::warn!(
                "Receiver::send_metadata called with non-metadata frame type: {:?}. Expected OMTFrameType::Metadata.",
                frame_ref.Type
            );
        }

        let result = unsafe { ffi::omt_receive_send(self.handle.as_ptr(), frame_ref) };
        Ok(result as i32)
    }

    /// Sets preview/program tally state for this receiver.
    pub fn set_tally(&self, tally: &Tally) {
        let mut raw = tally_to_ffi(tally);
        unsafe { ffi::omt_receive_settally(self.handle.as_ptr(), &mut raw) };
    }

    /// Retrieves tally state updates from the sender.
    pub fn get_tally(&self, timeout: Timeout, tally: &mut Tally) -> i32 {
        let mut raw = ffi::OMTTally {
            preview: 0,
            program: 0,
        };
        let result = unsafe {
            ffi::omt_receive_gettally(
                self.handle.as_ptr() as *mut ffi::omt_send_t,
                timeout.as_millis_i32(),
                &mut raw,
            ) as i32
        };
        *tally = Tally::from(&raw);
        result
    }

    /// Updates receiver flags (e.g., preview or compressed stream delivery).
    pub fn set_flags(&self, flags: ReceiveFlags) {
        unsafe { ffi::omt_receive_setflags(self.handle.as_ptr(), i32::from(flags)) };
    }

    /// Suggests a preferred quality to the sender when it is in Default mode.
    pub fn set_suggested_quality(&self, quality: Quality) {
        unsafe { ffi::omt_receive_setsuggestedquality(self.handle.as_ptr(), quality.into()) };
    }

    /// Fetches optional metadata about the connected sender.
    ///
    /// Returns sender device/software information if available, including product name,
    /// manufacturer, and version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// let address = Address::new("HOST (Sender Name)");
    /// let receiver = Receiver::create(
    ///     &address,
    ///     FrameType::Video,
    ///     PreferredVideoFormat::UYVYorBGRA,
    ///     ReceiveFlags::NONE,
    /// ).unwrap();
    ///
    /// if let Some(info) = receiver.get_sender_info() {
    ///     println!("Connected to: {} {} v{}",
    ///         info.manufacturer,
    ///         info.product_name,
    ///         info.version
    ///     );
    /// }
    /// ```
    pub fn get_sender_info(&self) -> Option<SenderInfo> {
        let mut info = ffi::OMTSenderInfo {
            ProductName: [0; ffi::OMT_MAX_STRING_LENGTH],
            Manufacturer: [0; ffi::OMT_MAX_STRING_LENGTH],
            Version: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved1: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved2: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved3: [0; ffi::OMT_MAX_STRING_LENGTH],
        };
        unsafe { ffi::omt_receive_getsenderinformation(self.handle.as_ptr(), &mut info) };
        sender_info_from_ffi(&info)
    }

    /// Returns video stream statistics for this receiver.
    pub fn get_video_statistics(&self) -> Statistics {
        let mut stats = unsafe { std::mem::zeroed::<ffi::OMTStatistics>() };
        unsafe { ffi::omt_receive_getvideostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }

    /// Returns audio stream statistics for this receiver.
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut stats = unsafe { std::mem::zeroed::<ffi::OMTStatistics>() };
        unsafe { ffi::omt_receive_getaudiostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        unsafe { ffi::omt_receive_destroy(self.handle.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Codec, ColorSpace, VideoFlags};

    #[test]
    fn test_send_metadata_with_metadata_frame() {
        // Create a receiver (this will fail if no sender is available, but we're just testing the API)
        let address = Address::new("HOST (Test Sender)");
        let receiver = Receiver::create(
            &address,
            FrameType::Video,
            PreferredVideoFormat::UYVYorBGRA,
            ReceiveFlags::NONE,
        );

        // Skip if receiver creation fails (no sender available)
        if receiver.is_err() {
            return;
        }

        let receiver = receiver.unwrap();

        // Create a metadata frame
        let mut metadata = MediaFrame::metadata("<test>data</test>", -1).unwrap();

        // This should work without warnings (though it may fail if no sender is connected)
        let _ = receiver.send_metadata(&mut metadata);
    }

    #[test]
    fn test_send_metadata_with_video_frame_logs_warning() {
        // Create a receiver
        let address = Address::new("HOST (Test Sender)");
        let receiver = Receiver::create(
            &address,
            FrameType::Video,
            PreferredVideoFormat::UYVYorBGRA,
            ReceiveFlags::NONE,
        );

        // Skip if receiver creation fails (no sender available)
        if receiver.is_err() {
            return;
        }

        let receiver = receiver.unwrap();

        // Create a video frame (not a metadata frame)
        let mut video_frame = MediaFrame::video(
            Codec::BGRA,
            640,
            480,
            640 * 4,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            -1,
            vec![0u8; 640 * 480 * 4],
        );

        // This should log a warning but not fail
        // (The actual send may fail if no sender is connected, but that's okay for this test)
        let _ = receiver.send_metadata(&mut video_frame);
        // The warning will be logged but we can't easily assert on log output in this test
    }
}
