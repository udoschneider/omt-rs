//! Media frame types for both sending and receiving via OMT.
//!
//! This module provides the [`MediaFrame`] type for constructing video, audio,
//! and metadata frames to be sent over the OMT transport, as well as accessing
//! received frames.
//!
//! # Frame Types
//!
//! OMT supports three types of media frames:
//!
//! - **Video frames**: Contain uncompressed or compressed video data with associated
//!   metadata like resolution, frame rate, color space, etc.
//! - **Audio frames**: Contain planar 32-bit floating point audio data with sample rate
//!   and channel information.
//! - **Metadata frames**: Contain UTF-8 encoded XML strings for control data, PTZ commands,
//!   web management interfaces, ancillary data, etc.
//!
//! # Timestamps
//!
//! All frames use the OMT timebase where 1 second = 10,000,000 ticks. This allows for
//! precise synchronization between audio and video streams.
//!
//! A special timestamp value of `-1` tells the sender to automatically generate timestamps
//! and throttle frame delivery based on the specified frame rate or sample rate.
//!
//! # Video Codecs
//!
//! When sending video frames, the following codecs are supported:
//! - `UYVY`, `YUY2`: 8-bit YUV 4:2:2 formats
//! - `NV12`, `YV12`: 8-bit YUV 4:2:0 planar formats
//! - `BGRA`: 8-bit RGB with alpha
//! - `UYVA`: 8-bit YUV 4:2:2 with alpha
//! - `P216`, `PA16`: 16-bit high bit depth formats
//! - `VMX1`: Compressed video format
//!
//! When receiving video frames, only `UYVY`, `UYVA`, `BGRA`, and `BGRX` are supported.
//!
//! # Audio Codec
//!
//! Audio frames use the `FPA1` codec (32-bit floating point planar audio).
//! Up to 32 audio channels are supported.
//!
//! # Frame Metadata
//!
//! Both video and audio frames can have per-frame metadata attached using
//! [`MediaFrame::set_frame_metadata`]. This metadata is carried alongside the payload
//! and is limited to 65,536 bytes.
//!
//! # Examples
//!
//! ## Creating a video frame
//!
//! ```
//! use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
//!
//! let frame = MediaFrame::video(
//!     Codec::BGRA,
//!     1920,
//!     1080,
//!     1920 * 4,
//!     VideoFlags::NONE,
//!     30,
//!     1,
//!     1.77778,
//!     ColorSpace::BT709,
//!     -1,  // Auto-generate timestamps
//!     vec![0u8; 1920 * 1080 * 4],
//! );
//! ```
//!
//! ## Creating an audio frame
//!
//! ```
//! use omt::{MediaFrame, Codec};
//!
//! let frame = MediaFrame::audio(
//!     Codec::FPA1,
//!     48000,  // Sample rate
//!     2,      // Stereo
//!     1920,   // Samples per channel
//!     -1,     // Auto-generate timestamps
//!     vec![0u8; 1920 * 2 * 4],
//! );
//! ```
//!
//! ## Creating a metadata frame
//!
//! ```no_run
//! use omt::MediaFrame;
//!
//! let frame = MediaFrame::metadata(
//!     r#"<OMTPTZ Protocol="VISCA" Sequence="22" Command="8101040700FF" />"#,
//!     -1,
//! ).expect("metadata frame");
//! ```

use crate::ffi;
use crate::helpers::{null_terminated_bytes, without_null_terminator};
use crate::types::{Codec, ColorSpace, VideoFlags};
use crate::video_conversion;
use crate::Error;
use rgb::bytemuck;

/// Media frame that can be either owned (for sending) or borrowed (for receiving).
///
/// This type wraps the underlying C FFI `OMTMediaFrame` structure and provides
/// safe Rust accessors for frame properties and data. Frames can be constructed for
/// sending video, audio, or metadata, or received from an OMT sender.
///
/// The internal buffers (data, compressed data, frame metadata) are kept alive for
/// the duration of the send call for owned frames.
///
/// # Lifetime
///
/// The `'a` lifetime parameter represents the lifetime of borrowed frames received
/// from an OMT sender. Owned frames (created for sending) use `'static` lifetime.
pub struct MediaFrame<'a> {
    frame: MediaFrameInner<'a>,
    _data: Option<Vec<u8>>,
    _frame_metadata: Option<Vec<u8>>,
}

enum MediaFrameInner<'a> {
    Owned(ffi::OMTMediaFrame),
    Borrowed(&'a ffi::OMTMediaFrame),
}

impl<'a> MediaFrame<'a> {
    // Construction methods for sending (owned frames)

    /// Creates a video frame from raw, uncompressed pixel data.
    ///
    /// This constructs an owned video frame for sending over OMT. The `data` buffer must
    /// contain pixel data in the format specified by `codec`, with dimensions matching
    /// `width`, `height`, and `stride`.
    ///
    /// # Parameters
    ///
    /// - `codec`: The pixel format of the data. Supported sending codecs include:
    ///   `UYVY`, `YUY2`, `NV12`, `YV12`, `BGRA`, `UYVA`, `P216`, `PA16`, and `VMX1`.
    /// - `width`: Frame width in pixels.
    /// - `height`: Frame height in pixels.
    /// - `stride`: Number of bytes per row of pixels. Typically:
    ///   - `width * 2` for UYVY/YUY2
    ///   - `width * 4` for BGRA
    ///   - `width` for planar formats
    /// - `flags`: Video flags indicating properties like interlacing, alpha channel, etc.
    /// - `frame_rate_n`: Frame rate numerator (frames per second).
    /// - `frame_rate_d`: Frame rate denominator. For example, `60/1` = 60 fps, `30000/1001` ≈ 29.97 fps.
    /// - `aspect_ratio`: Display aspect ratio as width/height. For example, `1.77778` for 16:9.
    /// - `color_space`: Color space of the video (BT601, BT709, or Undefined).
    /// - `timestamp`: Frame timestamp in OMT timebase (10,000,000 ticks per second).
    ///   Use `-1` to auto-generate timestamps based on frame rate.
    /// - `data`: Raw pixel data buffer. The size must match the stride and height.
    ///
    /// # Timestamps
    ///
    /// Timestamps represent the accurate time the frame was generated at the original source
    /// and should be used on the receiving end to synchronize and record with proper presentation
    /// timestamps (PTS). A value of `-1` tells the sender to generate timestamps automatically
    /// and throttle delivery to maintain the specified frame rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
    ///
    /// // Create a 1920x1080 BGRA frame at 30 fps
    /// let frame = MediaFrame::video(
    ///     Codec::BGRA,
    ///     1920,
    ///     1080,
    ///     1920 * 4,
    ///     VideoFlags::NONE,
    ///     30,
    ///     1,
    ///     1.77778,
    ///     ColorSpace::BT709,
    ///     -1,
    ///     vec![0u8; 1920 * 1080 * 4],
    /// );
    /// ```
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
            Codec: codec.to_ffi(),
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

        Self {
            frame: MediaFrameInner::Owned(frame),
            _data: Some(data),
            _frame_metadata: None,
        }
    }

    /// Creates an audio frame from raw, uncompressed sample data.
    ///
    /// This constructs an owned audio frame for sending over OMT. The audio must be
    /// in planar 32-bit floating point format (FPA1 codec), which is the only supported
    /// audio format.
    ///
    /// # Parameters
    ///
    /// - `codec`: Must be [`Codec::FPA1`] (32-bit floating point planar audio).
    /// - `sample_rate`: Sample rate in Hz (e.g., 48000, 44100).
    /// - `channels`: Number of audio channels. Maximum of 32 channels supported.
    /// - `samples_per_channel`: Number of samples in each channel/plane.
    /// - `timestamp`: Audio timestamp in OMT timebase (10,000,000 ticks per second).
    ///   Use `-1` to auto-generate timestamps based on sample rate.
    /// - `data`: Planar 32-bit floating point audio data. Each plane should contain
    ///   `samples_per_channel * 4` bytes. Total size: `channels * samples_per_channel * 4`.
    ///
    /// # Data Format
    ///
    /// The audio data is planar, meaning each channel's samples are stored contiguously:
    /// `[channel0_samples...][channel1_samples...][channel2_samples...]`
    ///
    /// Each sample is a 32-bit little-endian float.
    ///
    /// # Timestamps
    ///
    /// Timestamps represent the accurate time the audio sample was generated at the original
    /// source and should be used on the receiving end to synchronize with video. A value of
    /// `-1` tells the sender to generate timestamps automatically and throttle delivery to
    /// maintain the specified sample rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::{MediaFrame, Codec};
    ///
    /// // Create a stereo audio frame at 48kHz with 1920 samples per channel
    /// let frame = MediaFrame::audio(
    ///     Codec::FPA1,
    ///     48000,
    ///     2,
    ///     1920,
    ///     -1,
    ///     vec![0u8; 1920 * 2 * 4],
    /// );
    /// ```
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
            Codec: codec.to_ffi(),
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

        Self {
            frame: MediaFrameInner::Owned(frame),
            _data: Some(data),
            _frame_metadata: None,
        }
    }

    /// Creates an XML metadata frame.
    ///
    /// Metadata is sent over the bi-directional metadata channel and must be UTF-8 XML.
    /// Timestamps use the OMT timebase (10,000,000 ticks per second). A timestamp of `-1`
    /// can be used for immediate delivery.
    ///
    /// # Null Terminator Handling
    ///
    /// Although `libomt.h` explicitly documents that metadata strings must *include*
    /// the null terminator, this high-level Rust wrapper handles that automatically.
    /// You should pass metadata strings that do *not* include a null character - the
    /// wrapper will add the null terminator behind the scenes before passing to the
    /// C library.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata contains null bytes (null terminators are added automatically).
    ///
    /// # Examples
    ///
    /// ## Simple metadata
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let metadata = MediaFrame::metadata(
    ///     "<example><value>42</value></example>",
    ///     123456789,
    /// ).expect("metadata frame");
    /// ```
    ///
    /// ## Web management interface
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let web_metadata = MediaFrame::metadata(
    ///     r#"<OMTWeb URL="http://192.168.1.100/" />"#,
    ///     -1,
    /// ).expect("web management metadata");
    /// ```
    ///
    /// ## PTZ control - VISCA over IP
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let ptz_metadata = MediaFrame::metadata(
    ///     r#"<OMTPTZ Protocol="VISCAoverIP" URL="visca://192.168.1.100:52381" />"#,
    ///     -1,
    /// ).expect("PTZ metadata");
    /// ```
    ///
    /// ## PTZ control - VISCA inband command
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let ptz_command = MediaFrame::metadata(
    ///     r#"<OMTPTZ Protocol="VISCA" Sequence="22" Command="8101040700FF" />"#,
    ///     -1,
    /// ).expect("PTZ command");
    /// ```
    ///
    /// ## PTZ control - VISCA inband reply
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let ptz_reply = MediaFrame::metadata(
    ///     r#"<OMTPTZ Protocol="VISCA" Sequence="22" Reply="0011AABBCC" />"#,
    ///     -1,
    /// ).expect("PTZ reply");
    /// ```
    ///
    /// ## Ancillary data
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let anc_data = MediaFrame::metadata(
    ///     r#"<AncillaryData xmlns="urn:anc:1.0">
    /// <Packet did="45" sdid="01" field="1" line="21" horizOffset="0" st2110Channel="0" pts90k="32109876" link="A" stream="VANC">
    /// <Payload>81010A011E0000</Payload>
    /// </Packet>
    /// </AncillaryData>"#,
    ///     -1,
    /// ).expect("ancillary data");
    /// ```
    ///
    /// ## Grouped metadata
    ///
    /// Multiple metadata elements can be grouped in a single frame:
    ///
    /// ```no_run
    /// use omt::MediaFrame;
    ///
    /// let grouped = r#"<OMTGroup>
    /// <OMTPTZ Protocol="VISCA" Sequence="22" Reply="0011AABBCC" />
    /// <AncillaryData xmlns="urn:anc:1.0">
    /// <Packet did="45" sdid="01" field="1" line="21" horizOffset="0" st2110Channel="0" pts90k="32109876" link="A" stream="VANC">
    /// <Payload>81010A011E0000</Payload>
    /// </Packet>
    /// </AncillaryData>
    /// </OMTGroup>"#;
    ///
    /// let metadata = MediaFrame::metadata(grouped, -1).expect("grouped metadata");
    /// ```
    pub fn metadata<S: AsRef<str>>(data: S, timestamp: i64) -> Result<Self, Error> {
        let bytes = null_terminated_bytes(data.as_ref())?;
        let data_len = bytes.len() as i32;
        let data_ptr = bytes.as_ptr() as *mut std::ffi::c_void;

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

        Ok(Self {
            frame: MediaFrameInner::Owned(frame),
            _data: Some(bytes),
            _frame_metadata: None,
        })
    }
}

impl<'a> From<&'a ffi::OMTMediaFrame> for MediaFrame<'a> {
    fn from(raw: &'a ffi::OMTMediaFrame) -> Self {
        Self {
            frame: MediaFrameInner::Borrowed(raw),
            _data: None,
            _frame_metadata: None,
        }
    }
}

impl<'a> MediaFrame<'a> {
    // Common Methods for all frame types

    pub(crate) fn as_mut(&mut self) -> &mut ffi::OMTMediaFrame {
        match &mut self.frame {
            MediaFrameInner::Owned(ref mut frame) => frame,
            MediaFrameInner::Borrowed(_) => {
                panic!("Cannot get mutable reference to borrowed frame")
            }
        }
    }

    /// Returns a reference to the underlying FFI frame structure.
    fn raw(&self) -> &ffi::OMTMediaFrame {
        match &self.frame {
            MediaFrameInner::Owned(ref frame) => frame,
            MediaFrameInner::Borrowed(frame) => frame,
        }
    }

    /// Returns the type of the frame.
    ///
    /// This determines which fields of the frame structure are valid and should be used.
    ///
    /// # Returns
    ///
    /// One of:
    /// - `FrameType::Video`: Video frame with pixel data
    /// - `FrameType::Audio`: Audio frame with sample data
    /// - `FrameType::Metadata`: Metadata frame with XML string
    /// - `FrameType::None`: No frame data
    pub fn frame_type(&self) -> crate::types::FrameType {
        self.raw().Type.into()
    }

    /// Returns the timestamp of the frame.
    ///
    /// Timestamps use the OMT timebase (10,000,000 ticks per second).
    /// A timestamp of `-1` indicates that OMT should generate timestamps automatically.
    pub fn timestamp(&self) -> i64 {
        self.raw().Timestamp
    }

    /// Returns the codec used for this frame.
    ///
    /// For video frames, this indicates the pixel format (UYVY, BGRA, VMX1, etc.).
    /// For audio frames, this is always FPA1 (32-bit floating point planar audio).
    /// For metadata frames, this field is not meaningful.
    pub fn codec(&self) -> Codec {
        Codec::from(self.raw().Codec)
    }

    /// Returns the raw uncompressed data for this frame.
    ///
    /// # Data Contents
    ///
    /// - **Video frames**: Uncompressed pixel data (or compressed VMX1 data when sending with Codec set to VMX1)
    /// - **Audio frames**: Planar 32-bit floating point audio data
    /// - **Metadata frames**: UTF-8 encoded XML string with null terminator
    ///
    /// # Returns
    ///
    /// - `Some(&[u8])` containing the frame data
    /// - `None` if no data is present or the data pointer is null
    pub fn raw_data(&self) -> Option<&[u8]> {
        let raw = self.raw();
        if raw.Data.is_null() || raw.DataLength <= 0 {
            return None;
        }
        let len = raw.DataLength as usize;
        Some(unsafe { std::slice::from_raw_parts(raw.Data as *const u8, len) })
    }

    /// Returns the compressed video data (receive only).
    ///
    /// When receiving with `ReceiveFlags::IncludeCompressed` or `ReceiveFlags::CompressedOnly`
    /// set, this field will contain the original compressed video frame in VMX1 format.
    /// This can be muxed into an AVI or MOV file using FFmpeg or similar APIs.
    ///
    /// **Note**: This is only valid for received frames. When sending, use the standard
    /// `Data` field for VMX1 frames.
    ///
    /// # Returns
    ///
    /// - `Some(&[u8])` containing the compressed VMX1 data
    /// - `None` if no compressed data is present, the pointer is null, or this is a sent frame
    pub fn compressed_data(&self) -> Option<&[u8]> {
        let raw = self.raw();
        if raw.CompressedData.is_null() || raw.CompressedLength <= 0 {
            return None;
        }
        let len = raw.CompressedLength as usize;
        Some(unsafe { std::slice::from_raw_parts(raw.CompressedData as *const u8, len) })
    }

    /// Returns the per-frame metadata as a string slice.
    ///
    /// Per-frame metadata is carried alongside the video or audio payload and can contain
    /// frame-specific information like timecodes, ancillary data, or custom metadata.
    /// The metadata is UTF-8 encoded XML, limited to 65,536 bytes.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` containing the UTF-8 metadata (without null terminator)
    /// - `None` if no frame metadata is present or the pointer is null
    pub fn frame_metadata(&self) -> Option<&str> {
        let raw = self.raw();
        if raw.FrameMetadata.is_null() || raw.FrameMetadataLength <= 0 {
            return None;
        }
        let len = raw.FrameMetadataLength as usize;
        let slice = unsafe { std::slice::from_raw_parts(raw.FrameMetadata as *const u8, len) };
        Some(without_null_terminator(slice))
    }

    /// Sets the per-frame metadata.
    ///
    /// The metadata should be a UTF-8 XML string that will be attached to this frame.
    /// Per-frame metadata is carried alongside the video or audio payload and can contain
    /// frame-specific information. The metadata length is limited to 65536 bytes.
    /// Returns `Ok(&mut self)` to allow method chaining.
    ///
    /// # Null Terminator Handling
    ///
    /// Although `libomt.h` explicitly documents that metadata strings must *include*
    /// the null terminator, this high-level Rust wrapper handles that automatically.
    /// You should pass metadata strings that do *not* include a null character - the
    /// wrapper will add the null terminator behind the scenes before passing to the
    /// C library.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Called on a borrowed frame (only owned frames can have metadata set)
    /// - The metadata contains null bytes (null terminators are added automatically)
    ///
    /// # Examples
    ///
    /// ## Basic per-frame metadata
    ///
    /// ```
    /// use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
    ///
    /// let mut frame = MediaFrame::video(
    ///     Codec::BGRA,
    ///     1920,
    ///     1080,
    ///     1920 * 4,
    ///     VideoFlags::NONE,
    ///     30,
    ///     1,
    ///     1.77778,
    ///     ColorSpace::BT709,
    ///     -1,
    ///     vec![0u8; 1920 * 1080 * 4],
    /// );
    ///
    /// frame.set_frame_metadata("<custom>frame-specific metadata</custom>").unwrap();
    /// ```
    ///
    /// ## Method chaining
    ///
    /// ```
    /// use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
    ///
    /// let mut frame = MediaFrame::video(
    ///     Codec::BGRA,
    ///     640,
    ///     480,
    ///     640 * 4,
    ///     VideoFlags::NONE,
    ///     30,
    ///     1,
    ///     1.33333,
    ///     ColorSpace::BT601,
    ///     -1,
    ///     vec![0u8; 640 * 480 * 4],
    /// )
    /// .set_frame_metadata("<timecode>01:23:45:12</timecode>").unwrap();
    /// ```
    ///
    /// ## Frame-specific ancillary data
    ///
    /// ```
    /// use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
    ///
    /// let mut frame = MediaFrame::video(
    ///     Codec::UYVY,
    ///     1920,
    ///     1080,
    ///     1920 * 2,
    ///     VideoFlags::NONE,
    ///     25,
    ///     1,
    ///     1.77778,
    ///     ColorSpace::BT709,
    ///     -1,
    ///     vec![0u8; 1920 * 1080 * 2],
    /// );
    ///
    /// frame.set_frame_metadata(
    ///     r#"<AncillaryData xmlns="urn:anc:1.0">
    /// <Packet did="45" sdid="01" field="1" line="21" horizOffset="0">
    /// <Payload>81010A011E0000</Payload>
    /// </Packet>
    /// </AncillaryData>"#
    /// ).unwrap();
    /// ```
    pub fn set_frame_metadata<S: AsRef<str>>(&mut self, metadata: S) -> Result<&mut Self, Error> {
        match &mut self.frame {
            MediaFrameInner::Owned(ref mut frame) => {
                // Create null-terminated metadata bytes
                let metadata_bytes = null_terminated_bytes(metadata.as_ref())?;

                // Store the metadata bytes
                self._frame_metadata = Some(metadata_bytes);

                // Update FFI frame fields
                let metadata_ref = self._frame_metadata.as_ref().unwrap();
                frame.FrameMetadata = metadata_ref.as_ptr() as *mut std::ffi::c_void;
                frame.FrameMetadataLength = metadata_ref.len() as i32; // Includes null terminator

                Ok(self)
            }
            MediaFrameInner::Borrowed(_) => {
                Err(Error::InvalidCString) // TODO: Add a better error variant
            }
        }
    }

    /// Clears the per-frame metadata.
    ///
    /// Removes any metadata previously set with `set_frame_metadata`.
    /// Returns `Ok(&mut self)` to allow method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if called on a borrowed frame (only owned frames can have metadata cleared).
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};
    ///
    /// let mut frame = MediaFrame::video(
    ///     Codec::BGRA,
    ///     1920,
    ///     1080,
    ///     1920 * 4,
    ///     VideoFlags::NONE,
    ///     30,
    ///     1,
    ///     1.77778,
    ///     ColorSpace::BT709,
    ///     -1,
    ///     vec![0u8; 1920 * 1080 * 4],
    /// );
    ///
    /// frame.set_frame_metadata("<custom>metadata</custom>").unwrap();
    /// frame.clear_frame_metadata().unwrap();
    ///
    /// // Method chaining is also supported
    /// let mut frame2 = MediaFrame::video(
    ///     Codec::BGRA,
    ///     640,
    ///     480,
    ///     640 * 4,
    ///     VideoFlags::NONE,
    ///     30,
    ///     1,
    ///     1.33333,
    ///     ColorSpace::BT601,
    ///     -1,
    ///     vec![0u8; 640 * 480 * 4],
    /// )
    /// .set_frame_metadata("<chained>metadata</chained>").unwrap()
    /// .clear_frame_metadata().unwrap();
    /// ```
    pub fn clear_frame_metadata(&mut self) -> Result<&mut Self, Error> {
        match &mut self.frame {
            MediaFrameInner::Owned(ref mut frame) => {
                // Clear the metadata string
                self._frame_metadata = None;

                // Update FFI frame fields
                frame.FrameMetadata = std::ptr::null_mut();
                frame.FrameMetadataLength = 0;

                Ok(self)
            }
            MediaFrameInner::Borrowed(_) => {
                Err(Error::InvalidCString) // TODO: Add a better error variant
            }
        }
    }
}

impl<'a> MediaFrame<'a> {
    // Methods for video frames

    /// Returns the video frame width in pixels.
    ///
    /// This is only meaningful for video frames.
    pub fn width(&self) -> i32 {
        self.raw().Width
    }

    /// Returns the video frame height in pixels.
    ///
    /// This is only meaningful for video frames.
    pub fn height(&self) -> i32 {
        self.raw().Height
    }

    /// Returns the stride (number of bytes per row of pixels).
    ///
    /// The stride represents the actual bytes per row, which may include padding.
    /// Typical values:
    /// - `width * 2` for UYVY/YUY2
    /// - `width * 4` for BGRA
    /// - `width` for planar formats
    ///
    /// This is only meaningful for video frames.
    pub fn stride(&self) -> i32 {
        self.raw().Stride
    }

    /// Returns the video flags indicating frame properties.
    ///
    /// Flags can indicate whether the frame is interlaced, has an alpha channel,
    /// uses premultiplied alpha, is a preview frame, or uses high bit depth.
    ///
    /// This is only meaningful for video frames.
    pub fn flags(&self) -> VideoFlags {
        self.raw().Flags.into()
    }

    /// Returns the frame rate as a numerator/denominator pair.
    ///
    /// Frame rate is expressed in frames per second. For example:
    /// - `(60, 1)` = 60 fps
    /// - `(30000, 1001)` ≈ 29.97 fps
    /// - `(25, 1)` = 25 fps
    ///
    /// # Returns
    ///
    /// A tuple of `(numerator, denominator)`.
    ///
    /// This is only meaningful for video frames.
    pub fn frame_rate(&self) -> (i32, i32) {
        (self.raw().FrameRateN, self.raw().FrameRateD)
    }

    /// Returns the display aspect ratio.
    ///
    /// The aspect ratio is expressed as width/height. Common values:
    /// - `1.77778` (16/9)
    /// - `1.33333` (4/3)
    /// - `2.35` (cinemascope)
    ///
    /// This is only meaningful for video frames.
    pub fn aspect_ratio(&self) -> f32 {
        self.raw().AspectRatio
    }

    /// Returns the color space of the video frame.
    ///
    /// The color space determines how YUV values are converted to RGB. Common values:
    /// - `ColorSpace::BT709`: HDTV standard (1920x1080 and above)
    /// - `ColorSpace::BT601`: SDTV standard (SD resolution)
    /// - `ColorSpace::Undefined`: Color space not specified
    ///
    /// This is only meaningful for video frames.
    pub fn color_space(&self) -> ColorSpace {
        self.raw().ColorSpace.into()
    }
}

impl<'a> MediaFrame<'a> {
    // Conversion Methods for video frames

    /// Converts the video frame to 8-bit RGB format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGB pixel data (3 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// Not all combinations of input codecs and flags have been implemented or tested yet.
    pub fn rgb8_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgb8(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 8-bit RGBA format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGBA pixel data (4 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// Not all combinations of input codecs and flags have been implemented or tested yet.
    pub fn rgba8_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgba8(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 16-bit RGB format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGB16 pixel data (6 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// 16-bit output formats are not currently implemented and will always return `None`.
    pub fn rgb16_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgb16(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    /// Converts the video frame to 16-bit RGBA format.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<u8>)` containing the converted RGBA16 pixel data (8 bytes per pixel),
    /// or `None` if the conversion is not supported.
    ///
    /// # Note
    ///
    /// 16-bit output formats are not currently implemented and will always return `None`.
    pub fn rgba16_data(&self) -> Option<Vec<u8>> {
        video_conversion::to_rgba16(self).map(|data| bytemuck::cast_slice(&data).to_vec())
    }

    // Audio-specific methods
}

impl<'a> MediaFrame<'a> {
    // Methods for audio frames

    /// Returns the audio sample rate in Hz.
    ///
    /// This is typically only meaningful for audio frames.
    pub fn sample_rate(&self) -> i32 {
        self.raw().SampleRate
    }

    /// Returns the number of audio channels.
    ///
    /// This is typically only meaningful for audio frames.
    pub fn channels(&self) -> i32 {
        self.raw().Channels
    }

    /// Returns the number of samples per channel.
    ///
    /// This is typically only meaningful for audio frames.
    pub fn samples_per_channel(&self) -> i32 {
        self.raw().SamplesPerChannel
    }
}
impl<'a> MediaFrame<'a> {
    // Conversion Methods for audio frames
    /// Returns the audio data as a 2D vector of f32 samples.
    ///
    /// The outer vector represents channels, and the inner vector represents samples.
    /// Each sample is a 32-bit floating-point value in little-endian format.
    ///
    /// Returns `None` if:
    /// - No data is present
    /// - Channel or sample count is invalid
    /// - Data length doesn't match expected size
    ///
    /// This is typically only meaningful for audio frames.
    pub fn audio_data(&self) -> Option<Vec<Vec<f32>>> {
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
}
impl<'a> MediaFrame<'a> {
    // Conversion Methods for metadata frames

    /// Returns the metadata frame content as a string slice.
    ///
    /// For metadata frames, this returns the UTF-8 encoded XML string (without null terminator).
    /// This is equivalent to calling [`raw_data()`](MediaFrame::raw_data) and converting to a string,
    /// but more convenient for metadata frames.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` containing the UTF-8 XML metadata
    /// - `None` if no data is present or the pointer is null
    ///
    /// This is only meaningful for metadata frames.
    pub fn xml_data(&self) -> Option<&str> {
        let raw = self.raw();
        if raw.Data.is_null() || raw.DataLength <= 0 {
            return None;
        }
        let len = raw.DataLength as usize;
        let slice = unsafe { std::slice::from_raw_parts(raw.Data as *const u8, len) };
        Some(without_null_terminator(slice))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_frame_metadata_field() {
        let width = 1920;
        let height = 1080;
        let stride = 1920 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.77778,
            ColorSpace::BT709,
            -1,
            data,
        );

        // Initially no metadata
        assert!(frame._frame_metadata.is_none());

        // Set metadata using the setter method
        frame.set_frame_metadata("<test>metadata</test>").unwrap();
        assert_eq!(
            frame._frame_metadata.as_deref(),
            Some(b"<test>metadata</test>\0".as_ref())
        );
    }

    #[test]
    fn test_audio_frame_metadata_field() {
        let sample_rate = 48000;
        let channels = 2;
        let samples_per_channel = 1024;
        let data = vec![0u8; (channels * samples_per_channel * 4) as usize];

        let mut frame = MediaFrame::audio(
            Codec::FPA1,
            sample_rate,
            channels,
            samples_per_channel,
            -1,
            data,
        );

        // Set metadata using the setter method
        frame.set_frame_metadata("<audio>track</audio>").unwrap();
        assert_eq!(
            frame._frame_metadata.as_deref(),
            Some(b"<audio>track</audio>\0".as_ref())
        );
    }

    #[test]
    fn test_audio_methods_on_media_frame() {
        let sample_rate = 48000;
        let channels = 2;
        let samples_per_channel = 1024;

        // Create test audio data with known values
        let mut data = vec![0u8; (channels * samples_per_channel * 4) as usize];
        // Set first sample of first channel to 0.5
        data[0..4].copy_from_slice(&0.5f32.to_le_bytes());
        // Set first sample of second channel to -0.5
        let second_channel_offset = samples_per_channel as usize * 4;
        data[second_channel_offset..second_channel_offset + 4]
            .copy_from_slice(&(-0.5f32).to_le_bytes());

        let frame = MediaFrame::audio(
            Codec::FPA1,
            sample_rate,
            channels,
            samples_per_channel,
            -1,
            data.clone(),
        );

        // Test audio-specific accessors
        assert_eq!(frame.sample_rate(), sample_rate);
        assert_eq!(frame.channels(), channels);
        assert_eq!(frame.samples_per_channel(), samples_per_channel);

        // Test raw audio data
        let raw_audio = frame.raw_data().unwrap();
        assert_eq!(raw_audio.len(), data.len());
        assert_eq!(raw_audio, data.as_slice());

        // Test parsed audio data
        let audio_data = frame.audio_data().unwrap();
        assert_eq!(audio_data.len(), channels as usize);
        assert_eq!(audio_data[0].len(), samples_per_channel as usize);
        assert_eq!(audio_data[1].len(), samples_per_channel as usize);
        assert_eq!(audio_data[0][0], 0.5f32);
        assert_eq!(audio_data[1][0], -0.5f32);
    }

    #[test]
    fn test_metadata_frame() {
        let xml = "<test>metadata frame</test>";
        let frame = MediaFrame::metadata(xml, 1000).unwrap();

        // metadata creates a metadata-only frame with no additional metadata field
        assert!(frame._frame_metadata.is_none());
    }

    #[test]
    fn test_set_frame_metadata_with_string() {
        let width = 640;
        let height = 480;
        let stride = 640 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            5000,
            data,
        );

        // Test with String
        let metadata = String::from("<frame>test</frame>");
        frame.set_frame_metadata(&metadata).unwrap();
        assert_eq!(
            frame._frame_metadata.as_deref(),
            Some(b"<frame>test</frame>\0".as_ref())
        );

        // Test with &str and method chaining
        frame
            .set_frame_metadata("<frame>updated</frame>")
            .unwrap()
            .set_frame_metadata("<frame>chained</frame>")
            .unwrap();
        assert_eq!(
            frame._frame_metadata.as_deref(),
            Some(b"<frame>chained</frame>\0".as_ref())
        );
    }

    #[test]
    fn test_clear_frame_metadata() {
        let width = 640;
        let height = 480;
        let stride = 640 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            5000,
            data,
        );

        // Set metadata
        frame.set_frame_metadata("<frame>test</frame>").unwrap();
        assert!(frame._frame_metadata.is_some());

        // Clear metadata
        frame.clear_frame_metadata().unwrap();
        assert!(frame._frame_metadata.is_none());

        // Test chaining: set and clear
        frame
            .set_frame_metadata("<frame>new</frame>")
            .unwrap()
            .clear_frame_metadata()
            .unwrap();
        assert!(frame._frame_metadata.is_none());
    }

    #[test]
    fn test_metadata_updates_ffi_fields() {
        let width = 640;
        let height = 480;
        let stride = 640 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            5000,
            data,
        );

        // Initially metadata fields should be null/zero
        let ffi_frame = frame.as_mut();
        assert!(ffi_frame.FrameMetadata.is_null());
        assert_eq!(ffi_frame.FrameMetadataLength, 0);

        // Set metadata and verify FFI fields are updated
        frame.set_frame_metadata("<test>metadata</test>").unwrap();
        let ffi_frame = frame.as_mut();
        assert!(!ffi_frame.FrameMetadata.is_null());
        assert_eq!(ffi_frame.FrameMetadataLength, 22); // "<test>metadata</test>".len() + 1 for null

        // Clear metadata and verify FFI fields are reset
        frame.clear_frame_metadata().unwrap();
        let ffi_frame = frame.as_mut();
        assert!(ffi_frame.FrameMetadata.is_null());
        assert_eq!(ffi_frame.FrameMetadataLength, 0);
    }

    #[test]
    fn test_metadata_with_null_byte_fails() {
        let width = 640;
        let height = 480;
        let stride = 640 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            5000,
            data,
        );

        // Try to set metadata with null byte - should fail
        let result = frame.set_frame_metadata("metadata\0with_null");
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_has_null_terminator() {
        let width = 640;
        let height = 480;
        let stride = 640 * 4;
        let data = vec![0u8; (width * height * 4) as usize];

        let mut frame = MediaFrame::video(
            Codec::BGRA,
            width,
            height,
            stride,
            VideoFlags::NONE,
            30,
            1,
            1.33333,
            ColorSpace::BT601,
            5000,
            data,
        );

        // Set metadata
        frame.set_frame_metadata("<test>data</test>").unwrap();

        // Verify the stored bytes end with null terminator
        let metadata_bytes = frame._frame_metadata.as_ref().unwrap();
        assert_eq!(
            metadata_bytes.last(),
            Some(&0u8),
            "Metadata should end with null terminator"
        );

        // Verify length includes the null terminator
        let expected_len = "<test>data</test>".len() + 1;
        assert_eq!(metadata_bytes.len(), expected_len);

        // Verify FFI field has correct length (including null)
        let ffi_frame = frame.as_mut();
        assert_eq!(ffi_frame.FrameMetadataLength, expected_len as i32);
    }

    #[test]
    fn test_metadata_constructor_rejects_null_chars() {
        // Test that metadata constructor rejects strings with null characters
        let result = MediaFrame::metadata("metadata\0with_null", 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_constructor_adds_null_terminator() {
        // Test that metadata constructor adds null terminator
        let xml = "<test>metadata</test>";
        let frame = MediaFrame::metadata(xml, 1000).unwrap();

        // The data should include the null terminator
        let data_bytes = frame._data.as_ref().unwrap();
        assert_eq!(
            data_bytes.last(),
            Some(&0u8),
            "Metadata data should end with null terminator"
        );

        // Verify length includes the null terminator
        let expected_len = xml.len() + 1;
        assert_eq!(data_bytes.len(), expected_len);

        // Verify FFI field has correct length (including null)
        let ffi_frame = frame.raw();
        assert_eq!(ffi_frame.DataLength, expected_len as i32);
    }

    #[test]
    fn test_timestamp_and_frame_type_accessors() {
        use crate::types::FrameType;

        // Test video frame
        let timestamp_video = 12345678;
        let video_frame = MediaFrame::video(
            Codec::VMX1,
            1920,
            1080,
            1920 * 4,
            VideoFlags::empty(),
            30,
            1,
            16.0 / 9.0,
            ColorSpace::BT709,
            timestamp_video,
            vec![0u8; 1920 * 1080 * 4],
        );
        assert_eq!(video_frame.timestamp(), timestamp_video);
        assert_eq!(video_frame.frame_type(), FrameType::Video);

        // Test audio frame
        let timestamp_audio = 87654321;
        let audio_frame = MediaFrame::audio(
            Codec::FPA1,
            48000,
            2,
            1024,
            timestamp_audio,
            vec![0u8; 2 * 1024 * 4],
        );
        assert_eq!(audio_frame.timestamp(), timestamp_audio);
        assert_eq!(audio_frame.frame_type(), FrameType::Audio);

        // Test metadata frame
        let timestamp_metadata = 11223344;
        let metadata_frame = MediaFrame::metadata("<test/>", timestamp_metadata).unwrap();
        assert_eq!(metadata_frame.timestamp(), timestamp_metadata);
        assert_eq!(metadata_frame.frame_type(), FrameType::Metadata);

        // Test with auto-generated timestamp (-1)
        let auto_timestamp_frame = MediaFrame::video(
            Codec::VMX1,
            640,
            480,
            640 * 4,
            VideoFlags::empty(),
            25,
            1,
            4.0 / 3.0,
            ColorSpace::BT709,
            -1,
            vec![0u8; 640 * 480 * 4],
        );
        assert_eq!(auto_timestamp_frame.timestamp(), -1);
    }
}
