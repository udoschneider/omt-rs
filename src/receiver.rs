//! High-level receiver API for Open Media Transport (OMT).
//!
//! A receiver connects to a published sender and pulls video/audio/metadata
//! over the OMT TCP transport. The address is typically discovered via DNS-SD
//! (Bonjour/Avahi) or a discovery server (as described by `libomt.h`).
//! See https://github.com/openmediatransport for protocol background.

use crate::ffi;
use crate::types::{FrameRef, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, Timeout};
use crate::OmtError;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr::NonNull;

#[derive(Clone, Debug, Default)]
/// On-air tally state reported by the sender (preview/program).
pub struct Tally {
    pub preview: bool,
    pub program: bool,
}

#[derive(Clone, Debug)]
/// Optional metadata describing the sender device/software.
pub struct SenderInfo {
    pub product_name: String,
    pub manufacturer: String,
    pub version: String,
    pub reserved1: String,
    pub reserved2: String,
    pub reserved3: String,
}

#[derive(Clone, Debug)]
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

fn c_char_array_to_string(arr: &[c_char]) -> String {
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    let bytes: Vec<u8> = arr[..len].iter().map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).to_string()
}

fn sender_info_from_ffi(info: &ffi::OMTSenderInfo) -> Option<SenderInfo> {
    let product_name = c_char_array_to_string(&info.ProductName);
    let manufacturer = c_char_array_to_string(&info.Manufacturer);
    let version = c_char_array_to_string(&info.Version);
    let reserved1 = c_char_array_to_string(&info.Reserved1);
    let reserved2 = c_char_array_to_string(&info.Reserved2);
    let reserved3 = c_char_array_to_string(&info.Reserved3);

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

fn stats_from_ffi(stats: &ffi::OMTStatistics) -> Statistics {
    Statistics {
        bytes_sent: stats.BytesSent as i64,
        bytes_received: stats.BytesReceived as i64,
        bytes_sent_since_last: stats.BytesSentSinceLast as i64,
        bytes_received_since_last: stats.BytesReceivedSinceLast as i64,
        frames: stats.Frames as i64,
        frames_since_last: stats.FramesSinceLast as i64,
        frames_dropped: stats.FramesDropped as i64,
        codec_time: stats.CodecTime as i64,
        codec_time_since_last: stats.CodecTimeSinceLast as i64,
        reserved1: stats.Reserved1 as i64,
        reserved2: stats.Reserved2 as i64,
        reserved3: stats.Reserved3 as i64,
        reserved4: stats.Reserved4 as i64,
        reserved5: stats.Reserved5 as i64,
        reserved6: stats.Reserved6 as i64,
        reserved7: stats.Reserved7 as i64,
    }
}

fn tally_to_ffi(tally: &Tally) -> ffi::OMTTally {
    ffi::OMTTally {
        preview: if tally.preview { 1 } else { 0 },
        program: if tally.program { 1 } else { 0 },
    }
}

fn tally_from_ffi(tally: &ffi::OMTTally) -> Tally {
    Tally {
        preview: tally.preview != 0,
        program: tally.program != 0,
    }
}

fn frame_type_to_ffi(frame_type: FrameType) -> ffi::OMTFrameType {
    match frame_type {
        FrameType::Metadata => ffi::OMTFrameType::Metadata,
        FrameType::Video => ffi::OMTFrameType::Video,
        FrameType::Audio => ffi::OMTFrameType::Audio,
        FrameType::None => ffi::OMTFrameType::None,
    }
}

fn preferred_format_to_ffi(format: PreferredVideoFormat) -> ffi::OMTPreferredVideoFormat {
    match format {
        PreferredVideoFormat::UYVYorBGRA => ffi::OMTPreferredVideoFormat::UYVYorBGRA,
        PreferredVideoFormat::BGRA => ffi::OMTPreferredVideoFormat::BGRA,
        PreferredVideoFormat::UYVYorUYVA => ffi::OMTPreferredVideoFormat::UYVYorUYVA,
        PreferredVideoFormat::UYVYorUYVAorP216orPA16 => {
            ffi::OMTPreferredVideoFormat::UYVYorUYVAorP216orPA16
        }
        PreferredVideoFormat::P216 => ffi::OMTPreferredVideoFormat::P216,
        PreferredVideoFormat::UYVY => ffi::OMTPreferredVideoFormat::UYVY,
    }
}

fn receive_flags_to_ffi(flags: ReceiveFlags) -> ffi::OMTReceiveFlags {
    i32::from(flags)
}

fn quality_to_ffi(quality: Quality) -> ffi::OMTQuality {
    match quality {
        Quality::Low => ffi::OMTQuality::Low,
        Quality::Medium => ffi::OMTQuality::Medium,
        Quality::High => ffi::OMTQuality::High,
        Quality::Default => ffi::OMTQuality::Default,
    }
}

fn timeout_ms(timeout: Timeout) -> i32 {
    timeout
        .as_duration()
        .as_millis()
        .min(u128::from(i32::MAX as u32)) as i32
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
    /// `frame_types` selects which streams to receive, `format` controls the
    /// preferred pixel formats, and `flags` toggles optional behaviors such as
    /// preview or compressed delivery.
    pub fn create(
        address: &str,
        frame_types: FrameType,
        format: PreferredVideoFormat,
        flags: ReceiveFlags,
    ) -> Result<Self, OmtError> {
        let c_address = CString::new(address).map_err(|_| OmtError::InvalidCString)?;
        let handle = unsafe {
            ffi::omt_receive_create(
                c_address.as_ptr(),
                frame_type_to_ffi(frame_types),
                preferred_format_to_ffi(format),
                receive_flags_to_ffi(flags),
            )
        };
        let handle = NonNull::new(handle).ok_or(OmtError::NullHandle)?;
        Ok(Self { handle })
    }

    /// Receives the next frame of the requested type within the timeout.
    ///
    /// Returned frames are valid until the next `receive` call on this receiver
    /// (matching the `libomt.h` lifetime rules for `omt_receive`).
    pub fn receive(
        &mut self,
        frame_types: FrameType,
        timeout: Timeout,
    ) -> Result<Option<FrameRef<'_>>, OmtError> {
        let frame_ptr = unsafe {
            ffi::omt_receive(self.handle.as_ptr(), frame_type_to_ffi(frame_types), timeout_ms(timeout))
        };
        if frame_ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(FrameRef::new(unsafe { &*frame_ptr })))
        }
    }

    /// Returns an iterator that yields frames until a timeout occurs.
    ///
    /// Each iteration borrows the receiver, preventing re-entry while a frame
    /// reference is alive.
    pub fn frames<'a>(
        &'a mut self,
        frame_types: FrameType,
        timeout: Timeout,
    ) -> ReceiverFrames<'a> {
        ReceiverFrames {
            receiver: self,
            frame_types,
            timeout,
        }
    }

    /// Sends XML metadata back to the sender (bi-directional metadata channel).
    ///
    /// Timestamps use the OMT timebase (10,000,000 ticks per second).
    pub fn send_metadata_xml(&self, xml: &str, timestamp: i64) -> Result<i32, OmtError> {
        let c_xml = CString::new(xml).map_err(|_| OmtError::InvalidCString)?;
        let mut frame: ffi::OMTMediaFrame = unsafe { std::mem::zeroed() };
        frame.Type = ffi::OMTFrameType::Metadata;
        frame.Timestamp = timestamp as i64;
        frame.Data = c_xml.as_ptr() as *mut _;
        frame.DataLength = c_xml.as_bytes_with_nul().len() as i32;

        let result = unsafe { ffi::omt_receive_send(self.handle.as_ptr(), &mut frame) };
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
                timeout_ms(timeout),
                &mut raw,
            ) as i32
        };
        *tally = tally_from_ffi(&raw);
        result
    }

    /// Updates receiver flags (e.g., preview or compressed stream delivery).
    pub fn set_flags(&self, flags: ReceiveFlags) {
        unsafe { ffi::omt_receive_setflags(self.handle.as_ptr(), receive_flags_to_ffi(flags)) };
    }

    /// Suggests a preferred quality to the sender when it is in Default mode.
    pub fn set_suggested_quality(&self, quality: Quality) {
        unsafe { ffi::omt_receive_setsuggestedquality(self.handle.as_ptr(), quality_to_ffi(quality)) };
    }

    /// Fetches optional metadata about the connected sender.
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
        stats_from_ffi(&stats)
    }

    /// Returns audio stream statistics for this receiver.
    pub fn get_audio_statistics(&self) -> Statistics {
        let mut stats = unsafe { std::mem::zeroed::<ffi::OMTStatistics>() };
        unsafe { ffi::omt_receive_getaudiostatistics(self.handle.as_ptr(), &mut stats) };
        stats_from_ffi(&stats)
    }
}

/// Iterator over received frames for a receiver.
pub struct ReceiverFrames<'a> {
    receiver: &'a mut Receiver,
    frame_types: FrameType,
    timeout: Timeout,
}

impl<'a> Iterator for ReceiverFrames<'a> {
    type Item = Result<FrameRef<'a>, OmtError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.receive(self.frame_types, self.timeout) {
            Ok(Some(frame)) => {
                let frame = unsafe { std::mem::transmute::<FrameRef<'_>, FrameRef<'a>>(frame) };
                Some(Ok(frame))
            }
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        unsafe { ffi::omt_receive_destroy(self.handle.as_ptr()) };
    }
}
