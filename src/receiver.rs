//! High-level receiver API for Open Media Transport (OMT).
//!
//! A receiver connects to a published sender and pulls video/audio/metadata
//! over the OMT TCP transport. The address is typically discovered via DNS-SD
//! (Bonjour/Avahi) or using [`crate::discovery`] to get a list of available
//! sources on the network.
//!
//! # Address Format
//!
//! The address parameter can be:
//! - The full name provided by OMT Discovery (e.g., "HOST (Sender Name)")
//! - A URL in the format `omt://hostname:port`
//!
//! # Frame Lifetime
//!
//! Frames returned by [`Receiver::receive`] are valid only until the next
//! call to `receive` for that instance and frame type. This matches the
//! underlying `omt_receive` behavior in `libomt.h`. Pointers within frames
//! do not need to be freed by the caller.
//!
//! # Threading
//!
//! Make sure any threads currently accessing receiver functions are properly
//! closed before dropping the receiver instance.
//!
//! See <https://github.com/openmediatransport> for protocol background.

pub use crate::types::Tally;

use crate::ffi;
use crate::media_frame::MediaFrame;
use crate::types::{
    Address, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, SenderInfo, Statistics,
    Timeout,
};
use crate::Error;
use std::ffi::CString;
use std::ptr::NonNull;

/// High-level receiver handle for connecting to OMT senders.
///
/// A `Receiver` connects to a sender at a specified address and receives
/// video, audio, and/or metadata frames. The receiver maintains a connection
/// to the sender and buffers incoming frames.
///
/// # Thread Safety
///
/// `Receiver` is `Send` and `Sync`, allowing it to be used across threads.
/// However, ensure proper synchronization when calling methods from multiple threads.
///
/// # Cleanup
///
/// The receiver automatically disconnects and cleans up resources when dropped.
/// Make sure any threads accessing receiver functions are closed before the
/// receiver is dropped.
///
/// # Examples
///
/// ## Basic video reception
///
/// ```no_run
/// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};
///
/// let address = Address::new("HOST (My Sender)");
/// let mut receiver = Receiver::create(
///     &address,
///     FrameType::Video,
///     PreferredVideoFormat::UYVYorBGRA,
///     ReceiveFlags::NONE,
/// ).unwrap();
///
/// loop {
///     if let Ok(Some(frame)) = receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
///         println!("Received {}x{} video frame", frame.width(), frame.height());
///     }
/// }
/// ```
///
/// ## Receiving multiple frame types
///
/// ```no_run
/// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};
///
/// # let address = Address::new("HOST (My Sender)");
/// let mut receiver = Receiver::create(
///     &address,
///     FrameType::Video,
///     PreferredVideoFormat::UYVYorBGRA,
///     ReceiveFlags::NONE,
/// ).unwrap();
///
/// // Receive all frame types in a single thread
/// if let Ok(Some(frame)) = receiver.receive(
///     FrameType::Video,
///     Timeout::from_millis(100)
/// ) {
///     match frame.frame_type() {
///         FrameType::Video => println!("Got video"),
///         FrameType::Audio => println!("Got audio"),
///         FrameType::Metadata => println!("Got metadata"),
///         _ => {}
///     }
/// }
/// ```
pub struct Receiver {
    handle: NonNull<ffi::omt_receive_t>,
}

impl Receiver {
    /// Creates a new receiver and begins connecting to the sender at the specified address.
    ///
    /// # Parameters
    ///
    /// - `address`: Address to connect to, either:
    ///   - The full name provided by OMT Discovery (e.g., "HOST (Sender Name)")
    ///   - A URL in the format `omt://hostname:port`
    /// - `frame_types`: Specifies which types of frames to receive (e.g., video only, audio only, or combinations)
    /// - `format`: Preferred uncompressed video format. `UYVYorBGRA` will only receive BGRA frames when an alpha channel is present
    /// - `flags`: Optional flags such as:
    ///   - `Preview`: Request preview feed only
    ///   - `IncludeCompressed`: Include compressed (VMX) data with each frame for further processing or recording
    ///   - `CompressedOnly`: Receive only compressed data without decoding
    ///
    /// # Returns
    ///
    /// Returns `Ok(Receiver)` on successful connection, or `Err(Error)` if the connection fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// // Connect to a sender discovered via discovery
    /// let address = Address::new("HOST (My Camera)");
    /// let receiver = Receiver::create(
    ///     &address,
    ///     FrameType::Video,
    ///     PreferredVideoFormat::UYVYorBGRA,
    ///     ReceiveFlags::NONE,
    /// ).unwrap();
    /// ```
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// // Connect via direct URL
    /// let address = Address::new("omt://192.168.1.100:8001");
    /// let receiver = Receiver::create(
    ///     &address,
    ///     FrameType::Video,
    ///     PreferredVideoFormat::UYVY,
    ///     ReceiveFlags::PREVIEW,
    /// ).unwrap();
    /// ```
    pub fn create(
        address: &Address,
        frame_types: FrameType,
        format: PreferredVideoFormat,
        flags: ReceiveFlags,
    ) -> Result<Self, Error> {
        let c_address = CString::new(address.as_str()).map_err(|_| Error::InvalidCString)?;
        // SAFETY: FFI call to C library. The c_address pointer is valid for the duration
        // of the call, and all parameters are properly constructed C-compatible types.
        let handle = unsafe {
            ffi::omt_receive_create(
                c_address.as_ptr(),
                frame_types.into(),
                format.into(),
                i32::from(flags),
            )
        };
        let handle = NonNull::new(handle).ok_or(Error::NullHandle)?;
        Ok(Self { handle })
    }

    /// Receives any available frames in the buffer, or waits for frames if empty.
    ///
    /// # Parameters
    ///
    /// - `frame_types`: The frame types to receive. Set multiple types to receive them all in a single thread.
    ///   Set individually if using separate threads for audio/video/metadata.
    /// - `timeout`: The maximum time to wait for a new frame if the buffer is empty
    ///
    /// # Returns
    ///
    /// - `Ok(Some(MediaFrame))` if a frame was received
    /// - `Ok(None)` if the operation timed out
    /// - `Err(Error)` on error
    ///
    /// # Frame Lifetime
    ///
    /// **IMPORTANT**: The data in the returned frame is valid only until the next call to `receive`
    /// for this instance and frame type. Pointers within the frame do not need to be freed by the caller.
    ///
    /// # Timestamps
    ///
    /// Frame timestamps are in OMT ticks where 1 second = 10,000,000 ticks. Timestamps represent
    /// the accurate time the frame or audio sample was generated at the original source and should
    /// be used on the receiving end to synchronize and record to file as a presentation timestamp (PTS).
    ///
    /// # Metadata Format
    ///
    /// Metadata frames carry UTF-8 XML data with a terminating null byte.
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
    ) -> Result<Option<MediaFrame<'_>>, Error> {
        // SAFETY: FFI call to C library with valid handle and parameters.
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
            // SAFETY: The C library returns a valid pointer to a frame that remains
            // valid until the next receive call or until the receiver is destroyed.
            // We convert it to a reference with appropriate lifetime.
            Ok(Some(unsafe { &*frame_ptr }.into()))
        }
    }

    /// Sends a metadata frame to the sender (bi-directional metadata channel).
    ///
    /// This function only supports metadata frames. Use this for sending commands, status updates,
    /// or other control information to the sender.
    ///
    /// # Parameters
    ///
    /// - `frame`: A metadata frame created with [`crate::media_frame::MediaFrame::metadata`]
    ///
    /// # Returns
    ///
    /// Returns the result code from the underlying send operation.
    ///
    /// # Notes
    ///
    /// If a non-metadata frame is passed, a warning will be logged and the frame
    /// type will be used as-is, but this is not recommended.
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
    /// receiver.send(&mut metadata).unwrap();
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
    /// receiver.send(&mut ptz_cmd).unwrap();
    /// ```
    pub fn send(&self, frame: &mut MediaFrame) -> Result<i32, Error> {
        // Check if frame is a metadata frame and log warning if not
        let frame_ref = frame.as_mut();
        if frame_ref.Type != ffi::OMTFrameType::Metadata {
            log::warn!(
                "Receiver::send called with non-metadata frame type: {:?}. Expected OMTFrameType::Metadata.",
                frame_ref.Type
            );
        }

        // SAFETY: FFI call with valid handle and frame pointer.
        let result = unsafe { ffi::omt_receive_send(self.handle.as_ptr(), frame_ref) };
        Ok(result as i32)
    }

    /// Sets the preview/program tally state for this receiver.
    ///
    /// Informs the sender about this receiver's tally status. The sender aggregates
    /// tally information from all connected receivers.
    ///
    /// # Parameters
    ///
    /// - `tally`: Tally state with `preview` and `program` boolean flags
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Tally, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// let tally = Tally {
    ///     preview: true,
    ///     program: false,
    /// };
    /// receiver.set_tally(&tally);
    /// ```
    pub fn set_tally(&self, tally: &Tally) {
        let mut raw: ffi::OMTTally = tally.into();
        // SAFETY: FFI call with valid handle and tally struct pointer.
        unsafe { ffi::omt_receive_settally(self.handle.as_ptr(), &mut raw) };
    }

    /// Receives the current aggregated tally state across all connections to the sender.
    ///
    /// This returns the tally state across **all** receivers connected to the sender,
    /// not just this receiver's tally state.
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for tally updates
    /// - `tally`: Mutable reference to store the received tally state
    ///
    /// # Returns
    ///
    /// - `0` if timed out or tally didn't change
    /// - `1` if new tally state was received
    ///
    /// # Notes
    ///
    /// If this function times out, the last known tally state will be stored in `tally`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Tally, Timeout, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// let mut tally = Tally::default();
    /// let result = receiver.get_tally(Timeout::from_millis(100), &mut tally);
    /// if result == 1 {
    ///     println!("Tally updated: preview={}, program={}", tally.preview, tally.program);
    /// }
    /// ```
    pub fn get_tally(&self, timeout: Timeout, tally: &mut Tally) -> i32 {
        let mut raw = ffi::OMTTally {
            preview: 0,
            program: 0,
        };
        // SAFETY: FFI call with valid handle and mutable tally struct pointer.
        let result = unsafe {
            ffi::omt_receive_gettally(self.handle.as_ptr(), timeout.as_millis_i32(), &mut raw)
                as i32
        };
        *tally = Tally::from(&raw);
        result
    }

    /// Changes the flags on the current receive instance.
    ///
    /// This allows dynamic switching between preview mode and other receive behaviors.
    /// Changes will apply from the next frame received.
    ///
    /// # Parameters
    ///
    /// - `flags`: New receiver flags (e.g., `Preview`, `IncludeCompressed`, `CompressedOnly`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, ReceiveFlags, Address, FrameType, PreferredVideoFormat};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// // Switch to preview mode dynamically
    /// receiver.set_flags(ReceiveFlags::PREVIEW);
    /// ```
    pub fn set_flags(&self, flags: ReceiveFlags) {
        // SAFETY: FFI call with valid handle and flags value.
        unsafe { ffi::omt_receive_setflags(self.handle.as_ptr(), i32::from(flags)) };
    }

    /// Informs the sender of the quality preference for this receiver.
    ///
    /// # Quality Modes
    ///
    /// - `Quality::Default`: This receiver defers quality to whatever is set amongst other receivers
    /// - `Quality::Low`, `Quality::Medium`, `Quality::High`: Specific quality preferences
    ///
    /// If the sender is set to `Default` mode, it will allow quality suggestions from all receivers
    /// and select the highest suggested quality amongst them.
    ///
    /// # Parameters
    ///
    /// - `quality`: The preferred quality level for this receiver
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Quality, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// // Request high quality encoding
    /// receiver.set_suggested_quality(Quality::High);
    /// ```
    pub fn set_suggested_quality(&self, quality: Quality) {
        // SAFETY: FFI call with valid handle and quality value.
        unsafe { ffi::omt_receive_setsuggestedquality(self.handle.as_ptr(), quality.into()) };
    }

    /// Retrieves optional information describing the sender.
    ///
    /// This information is valid only when connected. Returns `None` if disconnected
    /// or if no sender information was provided by the sender.
    ///
    /// # Returns
    ///
    /// - `Some(SenderInfo)` containing product name, manufacturer, and version if available
    /// - `None` if not connected or sender didn't provide information
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
        use std::os::raw::c_char;
        let mut info = ffi::OMTSenderInfo {
            ProductName: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Manufacturer: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Version: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved1: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved2: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved3: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
        };
        // SAFETY: FFI call with valid handle and mutable sender info struct pointer.
        unsafe { ffi::omt_receive_getsenderinformation(self.handle.as_ptr(), &mut info) };
        Option::<SenderInfo>::from(&info)
    }

    /// Retrieves video stream statistics for this receiver.
    ///
    /// Statistics include byte counts, frame counts, codec timing, and deltas since
    /// the last statistics query.
    ///
    /// # Returns
    ///
    /// A [`Statistics`] struct containing video stream metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Video, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// let stats = receiver.get_video_statistics();
    /// println!("Received {} frames, {} bytes", stats.frames, stats.bytes_received);
    /// if stats.frames > 0 {
    ///     let avg_codec_time = stats.codec_time as f64 / stats.frames as f64;
    ///     println!("Average codec time: {:.2}ms per frame", avg_codec_time);
    /// }
    /// ```
    pub fn get_video_statistics(&self) -> Statistics {
        let mut stats = ffi::OMTStatistics::default();
        // SAFETY: FFI call with valid handle and mutable statistics struct pointer.
        unsafe { ffi::omt_receive_getvideostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }

    /// Retrieves audio stream statistics for this receiver.
    ///
    /// Statistics include byte counts, frame counts, codec timing, and deltas since
    /// the last statistics query.
    ///
    /// # Returns
    ///
    /// A [`Statistics`] struct containing audio stream metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::{Receiver, Address, FrameType, PreferredVideoFormat, ReceiveFlags};
    ///
    /// # let address = Address::new("HOST (Sender)");
    /// # let receiver = Receiver::create(&address, FrameType::Audio, PreferredVideoFormat::UYVYorBGRA, ReceiveFlags::NONE).unwrap();
    /// let stats = receiver.get_audio_statistics();
    /// println!("Received {} audio frames, {} bytes", stats.frames, stats.bytes_received);
    /// ```
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut stats = ffi::OMTStatistics::default();
        // SAFETY: FFI call with valid handle and mutable statistics struct pointer.
        unsafe { ffi::omt_receive_getaudiostatistics(self.handle.as_ptr(), &mut stats) };
        Statistics::from(&stats)
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        // SAFETY: FFI call to destroy the receiver handle. This is called once when
        // the receiver is dropped, and the handle is not used after this call.
        unsafe { ffi::omt_receive_destroy(self.handle.as_ptr()) };
    }
}

// SAFETY: The OMT C library's receiver handle is an opaque pointer that can be
// safely sent between threads and accessed from multiple threads. The underlying
// C library uses internal synchronization for thread safety.
unsafe impl Send for Receiver {}

// SAFETY: The OMT C library's receiver handle can be safely shared between threads.
// All receiver operations use the C library's internal synchronization mechanisms.
unsafe impl Sync for Receiver {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Codec, ColorSpace, FrameRate, VideoFlags};

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
        let _ = receiver.send(&mut metadata);
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
            FrameRate::fps_30(),
            1.33333,
            ColorSpace::BT601,
            -1,
            vec![0u8; 640 * 480 * 4],
        );

        // This should log a warning but not fail
        // (The actual send may fail if no sender is connected, but that's okay for this test)
        let _ = receiver.send(&mut video_frame);
        // The warning will be logged but we can't easily assert on log output in this test
    }
}
