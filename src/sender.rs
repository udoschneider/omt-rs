//! High-level sender API for Open Media Transport (OMT).
//!
//! A sender publishes a source and streams video plus bi-directional metadata
//! over the OMT TCP transport. Metadata sent by receivers is retrieved via
//! `omt_send_receive` (as described in `libomt.h`). See
//! <https://github.com/openmediatransport>

use crate::ffi;
use crate::ffi_utils::write_c_char_array;
use crate::receiver::{SenderInfo, Statistics, Tally};
use crate::types::{Address, Codec, ColorSpace, FrameRef, Quality, Source, Timeout, VideoFlags};
use crate::OmtError;
use std::ffi::CString;
use std::ptr::NonNull;

/// Owned frame payload for sending video or metadata.
///
/// The internal buffer is kept alive for the duration of the send call.
pub struct OutgoingFrame {
    frame: ffi::OMTMediaFrame,
    _data: Vec<u8>,
}

impl OutgoingFrame {
    /// Creates a video frame from raw, uncompressed pixel data.
    ///
    /// The `data` buffer must match the chosen `codec`, `width`, `height`, and `stride`.
    /// Timestamps use the OMT timebase (10,000,000 ticks per second). Use `timestamp = -1`
    /// to let OMT generate timestamps and pace frames based on the frame rate.
    #[allow(clippy::too_many_arguments)]
    pub fn video(
        codec: Codec,
        width: i32,
        height: i32,
        stride: i32,
        flags: VideoFlags,
        frame_rate_n: i32,
        frame_rate_d: i32,
        aspect_ratio: f32,
        color_space: ColorSpace,
        timestamp: i64,
        data: Vec<u8>,
    ) -> Self {
        let data_len = data.len() as i32;
        let data_ptr = data.as_ptr() as *mut std::ffi::c_void;

        let frame = ffi::OMTMediaFrame {
            Type: ffi::OMTFrameType::Video,
            Timestamp: timestamp,
            Codec: codec_to_ffi(codec),
            Width: width,
            Height: height,
            Stride: stride,
            Flags: i32::from(flags),
            FrameRateN: frame_rate_n,
            FrameRateD: frame_rate_d,
            AspectRatio: aspect_ratio,
            ColorSpace: color_space.into(),
            SampleRate: 0,
            Channels: 0,
            SamplesPerChannel: 0,
            Data: data_ptr,
            DataLength: data_len,
            CompressedData: std::ptr::null_mut(),
            CompressedLength: 0,
            FrameMetadata: std::ptr::null_mut(),
            FrameMetadataLength: 0,
        };

        Self { frame, _data: data }
    }

    /// Creates an audio frame from raw, uncompressed sample data.
    ///
    /// The `data` buffer should match the chosen `codec`, `sample_rate`,
    /// `channels`, and `samples_per_channel`. For planar 32-bit float audio
    /// (FPA1), this is `channels * samples_per_channel * 4` bytes.
    /// Timestamps use the OMT timebase (10,000,000 ticks per second). Use
    /// `timestamp = -1` to let OMT generate timestamps and pace frames based
    /// on the sample rate.
    pub fn audio(
        codec: Codec,
        sample_rate: i32,
        channels: i32,
        samples_per_channel: i32,
        timestamp: i64,
        data: Vec<u8>,
    ) -> Self {
        let data_len = data.len() as i32;
        let data_ptr = data.as_ptr() as *mut std::ffi::c_void;

        let frame = ffi::OMTMediaFrame {
            Type: ffi::OMTFrameType::Audio,
            Timestamp: timestamp,
            Codec: codec_to_ffi(codec),
            Width: 0,
            Height: 0,
            Stride: 0,
            Flags: ffi::OMT_VIDEO_FLAGS_NONE,
            FrameRateN: 0,
            FrameRateD: 0,
            AspectRatio: 0.0,
            ColorSpace: ffi::OMTColorSpace::Undefined,
            SampleRate: sample_rate,
            Channels: channels,
            SamplesPerChannel: samples_per_channel,
            Data: data_ptr,
            DataLength: data_len,
            CompressedData: std::ptr::null_mut(),
            CompressedLength: 0,
            FrameMetadata: std::ptr::null_mut(),
            FrameMetadataLength: 0,
        };

        Self { frame, _data: data }
    }

    /// Creates an XML metadata frame.
    ///
    /// Metadata is sent over the bi-directional metadata channel and must be UTF-8
    /// XML with a terminating null byte (length includes the null). Timestamps use
    /// the OMT timebase (10,000,000 ticks per second).
    pub fn metadata_xml(xml: &str, timestamp: i64) -> Result<Self, OmtError> {
        let c_xml = CString::new(xml).map_err(|_| OmtError::InvalidCString)?;
        let data = c_xml.into_bytes_with_nul();
        let data_len = data.len() as i32;
        let data_ptr = data.as_ptr() as *mut std::ffi::c_void;

        let frame = ffi::OMTMediaFrame {
            Type: ffi::OMTFrameType::Metadata,
            Timestamp: timestamp,
            Codec: ffi::OMTCodec::VMX1,
            Width: 0,
            Height: 0,
            Stride: 0,
            Flags: ffi::OMT_VIDEO_FLAGS_NONE,
            FrameRateN: 0,
            FrameRateD: 0,
            AspectRatio: 0.0,
            ColorSpace: ffi::OMTColorSpace::Undefined,
            SampleRate: 0,
            Channels: 0,
            SamplesPerChannel: 0,
            Data: data_ptr,
            DataLength: data_len,
            CompressedData: std::ptr::null_mut(),
            CompressedLength: 0,
            FrameMetadata: std::ptr::null_mut(),
            FrameMetadataLength: 0,
        };

        Ok(Self { frame, _data: data })
    }

    pub(crate) fn as_mut(&mut self) -> &mut ffi::OMTMediaFrame {
        &mut self.frame
    }
}

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
    pub fn create(source: &Source, quality: Quality) -> Result<Self, OmtError> {
        let c_name = CString::new(source.as_str()).map_err(|_| OmtError::InvalidCString)?;
        let handle = unsafe { ffi::omt_send_create(c_name.as_ptr(), quality.into()) };
        let handle = NonNull::new(handle).ok_or(OmtError::NullHandle)?;
        Ok(Self { handle })
    }

    pub fn set_sender_info(&self, info: &SenderInfo) {
        let mut raw = sender_info_to_ffi(info);
        unsafe { ffi::omt_send_setsenderinformation(self.handle.as_ptr(), &mut raw) };
    }

    /// Adds connection-level metadata (applies to new connections).
    pub fn add_connection_metadata(&self, metadata: &str) -> Result<(), OmtError> {
        let c_meta = CString::new(metadata).map_err(|_| OmtError::InvalidCString)?;
        unsafe { ffi::omt_send_addconnectionmetadata(self.handle.as_ptr(), c_meta.as_ptr()) };
        Ok(())
    }

    /// Clears any connection-level metadata set with `add_connection_metadata`.
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
    pub fn send(&self, frame: &mut OutgoingFrame) -> i32 {
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
    pub fn receive_metadata(&mut self, timeout: Timeout) -> Result<Option<FrameRef<'_>>, OmtError> {
        let frame_ptr =
            unsafe { ffi::omt_send_receive(self.handle.as_ptr(), timeout.as_millis_i32()) };
        if frame_ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(FrameRef::new(unsafe { &*frame_ptr })))
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

fn sender_info_to_ffi(info: &SenderInfo) -> ffi::OMTSenderInfo {
    let mut raw = ffi::OMTSenderInfo {
        ProductName: [0; ffi::OMT_MAX_STRING_LENGTH],
        Manufacturer: [0; ffi::OMT_MAX_STRING_LENGTH],
        Version: [0; ffi::OMT_MAX_STRING_LENGTH],
        Reserved1: [0; ffi::OMT_MAX_STRING_LENGTH],
        Reserved2: [0; ffi::OMT_MAX_STRING_LENGTH],
        Reserved3: [0; ffi::OMT_MAX_STRING_LENGTH],
    };

    write_c_char_array(&mut raw.ProductName, &info.product_name);
    write_c_char_array(&mut raw.Manufacturer, &info.manufacturer);
    write_c_char_array(&mut raw.Version, &info.version);
    write_c_char_array(&mut raw.Reserved1, &info.reserved1);
    write_c_char_array(&mut raw.Reserved2, &info.reserved2);
    write_c_char_array(&mut raw.Reserved3, &info.reserved3);

    raw
}

fn codec_to_ffi(codec: Codec) -> ffi::OMTCodec {
    match codec {
        Codec::VMX1 => ffi::OMTCodec::VMX1,
        Codec::FPA1 => ffi::OMTCodec::FPA1,
        Codec::UYVY => ffi::OMTCodec::UYVY,
        Codec::YUY2 => ffi::OMTCodec::YUY2,
        Codec::BGRA => ffi::OMTCodec::BGRA,
        Codec::NV12 => ffi::OMTCodec::NV12,
        Codec::YV12 => ffi::OMTCodec::YV12,
        Codec::UYVA => ffi::OMTCodec::UYVA,
        Codec::P216 => ffi::OMTCodec::P216,
        Codec::PA16 => ffi::OMTCodec::PA16,
        Codec::Unknown(_) => ffi::OMTCodec::VMX1,
    }
}
