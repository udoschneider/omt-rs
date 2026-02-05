//! High-level receiver API for Open Media Transport (OMT).
//!
//! A receiver connects to a published sender and pulls video/audio/metadata
//! over the OMT TCP transport. The address is typically discovered via DNS-SD
//! (Bonjour/Avahi) or a discovery server (as described by `libomt.h`).
//! See <https://github.com/openmediatransport> for protocol background.

use crate::ffi;
use crate::ffi_utils::c_char_array_to_string;
use crate::types::{
    Address, FrameRef, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, Timeout,
};
use crate::OmtError;
use std::ffi::CString;
use std::ptr::NonNull;

/// Audio-specific accessors for a received media frame.
pub struct AudioFrame<'a> {
    raw: &'a ffi::OMTMediaFrame,
}

impl<'a> AudioFrame<'a> {
    pub(crate) fn new(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self { raw }
    }

    pub fn sample_rate(&self) -> i32 {
        self.raw.SampleRate
    }

    pub fn channels(&self) -> i32 {
        self.raw.Channels
    }

    pub fn samples_per_channel(&self) -> i32 {
        self.raw.SamplesPerChannel
    }

    pub fn raw_data(&self) -> Option<&'a [u8]> {
        if self.raw.Data.is_null() || self.raw.DataLength <= 0 {
            return None;
        }
        let len = self.raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.Data as *const u8, len) })
    }

    pub fn data(&self) -> Option<Vec<Vec<f32>>> {
        let data = self.raw_data()?;
        let channels = self.channels();
        let samples_per_channel = self.samples_per_channel();

        if channels <= 0 || samples_per_channel <= 0 {
            return None;
        }

        let channels = channels as usize;
        let samples_per_channel = samples_per_channel as usize;
        let total_samples = channels.checked_mul(samples_per_channel)?;
        let expected_len = total_samples.checked_mul(4)?;
        if data.len() != expected_len {
            return None;
        }

        let mut out = vec![vec![0f32; samples_per_channel]; channels];

        for (ch, channel_data) in out.iter_mut().enumerate() {
            let plane_base = ch * samples_per_channel * 4;
            for (sample_idx, sample) in channel_data.iter_mut().enumerate() {
                let i = plane_base + sample_idx * 4;
                let bytes = [data[i], data[i + 1], data[i + 2], data[i + 3]];
                *sample = f32::from_le_bytes(bytes);
            }
        }

        Some(out)
    }

    pub fn compressed_data(&self) -> Option<&'a [u8]> {
        if self.raw.CompressedData.is_null() || self.raw.CompressedLength <= 0 {
            return None;
        }
        let len = self.raw.CompressedLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.CompressedData as *const u8, len) })
    }

    pub fn metadata(&self) -> Option<&'a [u8]> {
        if self.raw.FrameMetadata.is_null() || self.raw.FrameMetadataLength <= 0 {
            return None;
        }
        let len = self.raw.FrameMetadataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(self.raw.FrameMetadata as *const u8, len) })
    }
}

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
    pub fn receive(
        &mut self,
        frame_types: FrameType,
        timeout: Timeout,
    ) -> Result<Option<FrameRef<'_>>, OmtError> {
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
            Ok(Some(FrameRef::new(unsafe { &*frame_ptr })))
        }
    }

    /// Sends XML metadata back to the sender (bi-directional metadata channel).
    ///
    /// Metadata must be UTF-8 XML with a terminating null byte (length includes
    /// the null). Timestamps use the OMT timebase (10,000,000 ticks per second).
    pub fn send_metadata_xml(&self, xml: &str, timestamp: i64) -> Result<i32, OmtError> {
        let c_xml = CString::new(xml).map_err(|_| OmtError::InvalidCString)?;
        let mut frame: ffi::OMTMediaFrame = unsafe { std::mem::zeroed() };
        frame.Type = ffi::OMTFrameType::Metadata;
        frame.Timestamp = timestamp;
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
