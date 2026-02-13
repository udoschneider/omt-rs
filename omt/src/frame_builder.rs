//! Builder for creating owned media frames.
//!
//! This module provides builders for creating video, audio, and metadata frames
//! that can be sent via the OMT protocol. Frames own their data and properly
//! manage memory allocation.

use crate::error::{Error, Result};
use crate::frame::MediaFrame;
use crate::types::{Codec, ColorSpace, FrameType, VideoFlags};
use std::ffi::CString;

/// Builder for creating video frames.
///
/// # Examples
///
/// ```no_run
/// use omt::{VideoFrameBuilder, Codec, VideoFlags};
///
/// let data = vec![0u8; 1920 * 1080 * 2]; // UYVY data
/// let frame = VideoFrameBuilder::new()
///     .codec(Codec::Uyvy)
///     .dimensions(1920, 1080)
///     .stride(1920 * 2)
///     .frame_rate(30, 1)
///     .aspect_ratio(16.0 / 9.0)
///     .data(data)
///     .build()?;
/// # Ok::<(), omt::Error>(())
/// ```
#[derive(Debug)]
pub struct VideoFrameBuilder {
    codec: Option<Codec>,
    width: i32,
    height: i32,
    stride: Option<i32>,
    flags: VideoFlags,
    frame_rate_n: i32,
    frame_rate_d: i32,
    aspect_ratio: f32,
    color_space: ColorSpace,
    timestamp: i64,
    data: Vec<u8>,
    frame_metadata: Option<String>,
}

impl VideoFrameBuilder {
    /// Creates a new video frame builder.
    pub fn new() -> Self {
        Self {
            codec: None,
            width: 0,
            height: 0,
            stride: None,
            flags: VideoFlags::NONE,
            frame_rate_n: 30,
            frame_rate_d: 1,
            aspect_ratio: 16.0 / 9.0,
            color_space: ColorSpace::Undefined,
            timestamp: -1,
            data: Vec::new(),
            frame_metadata: None,
        }
    }

    /// Sets the video codec.
    ///
    /// Supported codecs for sending:
    /// - `Codec::Uyvy` - 16bpp YUV format
    /// - `Codec::Yuy2` - 16bpp YUV format YUYV pixel order
    /// - `Codec::Bgra` - 32bpp RGBA format
    /// - `Codec::Uyva` - 16bpp YUV format with alpha plane
    /// - `Codec::Nv12` - Planar 4:2:0 YUV format
    /// - `Codec::Yv12` - Planar 4:2:0 YUV format
    /// - `Codec::P216` - Planar 4:2:2 YUV format (16-bit)
    /// - `Codec::Pa16` - P216 with alpha plane
    /// - `Codec::Vmx1` - Compressed VMX1 format
    pub fn codec(mut self, codec: Codec) -> Self {
        self.codec = Some(codec);
        self
    }

    /// Sets the video dimensions in pixels.
    pub fn dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the stride (row pitch) in bytes.
    ///
    /// If not set, will be automatically calculated based on codec and width:
    /// - UYVY/YUY2: width * 2
    /// - BGRA: width * 4
    /// - UYVA: width * 2
    /// - Planar formats: width
    pub fn stride(mut self, stride: i32) -> Self {
        self.stride = Some(stride);
        self
    }

    /// Sets video flags (interlaced, alpha, premultiplied, etc.).
    pub fn flags(mut self, flags: VideoFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the frame rate as numerator and denominator.
    ///
    /// For example: `frame_rate(60, 1)` for 60fps, `frame_rate(30000, 1001)` for 29.97fps.
    pub fn frame_rate(mut self, numerator: i32, denominator: i32) -> Self {
        self.frame_rate_n = numerator;
        self.frame_rate_d = denominator;
        self
    }

    /// Sets the display aspect ratio (e.g., 16.0/9.0 for 16:9).
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = ratio;
        self
    }

    /// Sets the color space.
    pub fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    /// Sets the timestamp in OMT units (1 second = 10,000,000 units).
    ///
    /// Use -1 for auto-generated timestamps (default).
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the frame data.
    ///
    /// The data should be in the format specified by the codec.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Sets per-frame metadata (UTF-8 string, max 65536 bytes).
    pub fn frame_metadata(mut self, metadata: String) -> Self {
        self.frame_metadata = Some(metadata);
        self
    }

    /// Builds the video frame.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No codec is specified
    /// - Width or height is zero
    /// - Data is empty
    /// - Frame metadata exceeds 65536 bytes
    pub fn build(self) -> Result<OwnedMediaFrame> {
        let codec = self.codec.ok_or(Error::InvalidParameter {
            parameter: "codec".to_string(),
            reason: "codec must be specified".to_string(),
        })?;

        if self.width <= 0 || self.height <= 0 {
            return Err(Error::InvalidParameter {
                parameter: "dimensions".to_string(),
                reason: "width and height must be greater than zero".to_string(),
            });
        }

        if self.data.is_empty() {
            return Err(Error::InvalidParameter {
                parameter: "data".to_string(),
                reason: "data cannot be empty".to_string(),
            });
        }

        // Calculate stride if not specified
        let stride = self.stride.unwrap_or_else(|| match codec {
            Codec::Uyvy | Codec::Yuy2 | Codec::Uyva => self.width * 2,
            Codec::Bgra => self.width * 4,
            Codec::P216 | Codec::Pa16 => self.width * 2,
            _ => self.width,
        });

        let frame_metadata_cstring = if let Some(ref metadata) = self.frame_metadata {
            if metadata.len() > 65535 {
                return Err(Error::BufferTooSmall {
                    required: metadata.len(),
                    provided: 65536,
                });
            }
            Some(CString::new(metadata.as_str())?)
        } else {
            None
        };

        Ok(OwnedMediaFrame {
            frame_type: FrameType::VIDEO,
            codec,
            timestamp: self.timestamp,
            width: self.width,
            height: self.height,
            stride,
            flags: self.flags,
            frame_rate_n: self.frame_rate_n,
            frame_rate_d: self.frame_rate_d,
            aspect_ratio: self.aspect_ratio,
            color_space: self.color_space,
            sample_rate: 0,
            channels: 0,
            samples_per_channel: 0,
            data: self.data,
            frame_metadata: frame_metadata_cstring,
        })
    }
}

impl Default for VideoFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating audio frames.
///
/// # Examples
///
/// ```no_run
/// use omt::AudioFrameBuilder;
///
/// let samples_per_channel = 1024;
/// let channels = 2;
/// let data = vec![0.0f32; samples_per_channel * channels];
/// let data_bytes = data.iter()
///     .flat_map(|&f| f.to_ne_bytes())
///     .collect::<Vec<u8>>();
///
/// let frame = AudioFrameBuilder::new()
///     .sample_rate(48000)
///     .channels(channels as i32)
///     .samples_per_channel(samples_per_channel as i32)
///     .data(data_bytes)
///     .build()?;
/// # Ok::<(), omt::Error>(())
/// ```
#[derive(Debug)]
pub struct AudioFrameBuilder {
    sample_rate: i32,
    channels: i32,
    samples_per_channel: i32,
    timestamp: i64,
    data: Vec<u8>,
    frame_metadata: Option<String>,
}

impl AudioFrameBuilder {
    /// Creates a new audio frame builder.
    pub fn new() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            samples_per_channel: 0,
            timestamp: -1,
            data: Vec::new(),
            frame_metadata: None,
        }
    }

    /// Sets the sample rate (e.g., 48000, 44100).
    pub fn sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Sets the number of audio channels (max 32).
    pub fn channels(mut self, channels: i32) -> Self {
        self.channels = channels;
        self
    }

    /// Sets the number of samples per channel.
    pub fn samples_per_channel(mut self, samples: i32) -> Self {
        self.samples_per_channel = samples;
        self
    }

    /// Sets the timestamp in OMT units (1 second = 10,000,000 units).
    ///
    /// Use -1 for auto-generated timestamps (default).
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the audio data (32-bit floating point planar format).
    ///
    /// The data should be organized as planar audio: all samples for channel 0,
    /// then all samples for channel 1, etc.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Sets per-frame metadata (UTF-8 string, max 65536 bytes).
    pub fn frame_metadata(mut self, metadata: String) -> Self {
        self.frame_metadata = Some(metadata);
        self
    }

    /// Builds the audio frame.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Sample rate is zero
    /// - Channels is zero or exceeds 32
    /// - Samples per channel is zero
    /// - Data is empty or size doesn't match samples_per_channel * channels * 4
    /// - Frame metadata exceeds 65536 bytes
    pub fn build(self) -> Result<OwnedMediaFrame> {
        if self.sample_rate <= 0 {
            return Err(Error::InvalidParameter {
                parameter: "sample_rate".to_string(),
                reason: "sample rate must be greater than zero".to_string(),
            });
        }

        if self.channels <= 0 || self.channels > 32 {
            return Err(Error::InvalidParameter {
                parameter: "channels".to_string(),
                reason: "channels must be between 1 and 32".to_string(),
            });
        }

        if self.samples_per_channel <= 0 {
            return Err(Error::InvalidParameter {
                parameter: "samples_per_channel".to_string(),
                reason: "samples per channel must be greater than zero".to_string(),
            });
        }

        let expected_size = (self.samples_per_channel * self.channels * 4) as usize;
        if self.data.len() != expected_size {
            return Err(Error::InvalidParameter {
                parameter: "data".to_string(),
                reason: format!(
                    "data size ({}) doesn't match expected size ({}) for {} channels, {} samples per channel",
                    self.data.len(),
                    expected_size,
                    self.channels,
                    self.samples_per_channel
                ),
            });
        }

        let frame_metadata_cstring = if let Some(ref metadata) = self.frame_metadata {
            if metadata.len() > 65535 {
                return Err(Error::BufferTooSmall {
                    required: metadata.len(),
                    provided: 65536,
                });
            }
            Some(CString::new(metadata.as_str())?)
        } else {
            None
        };

        Ok(OwnedMediaFrame {
            frame_type: FrameType::AUDIO,
            codec: Codec::Fpa1,
            timestamp: self.timestamp,
            width: 0,
            height: 0,
            stride: 0,
            flags: VideoFlags::NONE,
            frame_rate_n: 0,
            frame_rate_d: 0,
            aspect_ratio: 0.0,
            color_space: ColorSpace::Undefined,
            sample_rate: self.sample_rate,
            channels: self.channels,
            samples_per_channel: self.samples_per_channel,
            data: self.data,
            frame_metadata: frame_metadata_cstring,
        })
    }
}

impl Default for AudioFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating metadata frames.
///
/// # Examples
///
/// ```no_run
/// use omt::MetadataFrameBuilder;
///
/// let frame = MetadataFrameBuilder::new()
///     .metadata("<metadata>Example</metadata>")
///     .build()?;
/// # Ok::<(), omt::Error>(())
/// ```
#[derive(Debug)]
pub struct MetadataFrameBuilder {
    timestamp: i64,
    metadata: String,
}

impl MetadataFrameBuilder {
    /// Creates a new metadata frame builder.
    pub fn new() -> Self {
        Self {
            timestamp: -1,
            metadata: String::new(),
        }
    }

    /// Sets the timestamp in OMT units (1 second = 10,000,000 units).
    ///
    /// Use -1 for auto-generated timestamps (default).
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the metadata content (UTF-8 encoded XML string).
    pub fn metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = metadata.into();
        self
    }

    /// Builds the metadata frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata is empty.
    pub fn build(self) -> Result<OwnedMediaFrame> {
        if self.metadata.is_empty() {
            return Err(Error::InvalidParameter {
                parameter: "metadata".to_string(),
                reason: "metadata cannot be empty".to_string(),
            });
        }

        let c_string = CString::new(self.metadata)?;
        let data = c_string.as_bytes_with_nul().to_vec();

        Ok(OwnedMediaFrame {
            frame_type: FrameType::METADATA,
            codec: Codec::Vmx1, // Codec doesn't matter for metadata
            timestamp: self.timestamp,
            width: 0,
            height: 0,
            stride: 0,
            flags: VideoFlags::NONE,
            frame_rate_n: 0,
            frame_rate_d: 0,
            aspect_ratio: 0.0,
            color_space: ColorSpace::Undefined,
            sample_rate: 0,
            channels: 0,
            samples_per_channel: 0,
            data,
            frame_metadata: None,
        })
    }
}

impl Default for MetadataFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// An owned media frame that manages its own memory.
///
/// This frame can be sent via [`Sender::send()`](crate::Sender::send) and
/// owns all its data, ensuring memory safety.
#[derive(Debug)]
pub struct OwnedMediaFrame {
    frame_type: FrameType,
    codec: Codec,
    timestamp: i64,
    width: i32,
    height: i32,
    stride: i32,
    flags: VideoFlags,
    frame_rate_n: i32,
    frame_rate_d: i32,
    aspect_ratio: f32,
    color_space: ColorSpace,
    sample_rate: i32,
    channels: i32,
    samples_per_channel: i32,
    data: Vec<u8>,
    frame_metadata: Option<CString>,
}

impl OwnedMediaFrame {
    /// Converts this owned frame to a borrowed `MediaFrame` for sending.
    ///
    /// The returned frame borrows data from this owned frame, so the owned
    /// frame must remain valid while the borrowed frame is in use.
    pub fn as_media_frame(&self) -> MediaFrame {
        let mut ffi = omt_sys::OMTMediaFrame {
            Type: self.frame_type.to_ffi(),
            Timestamp: self.timestamp,
            Codec: self.codec.to_ffi(),
            Width: self.width,
            Height: self.height,
            Stride: self.stride,
            Flags: self.flags.to_ffi(),
            FrameRateN: self.frame_rate_n,
            FrameRateD: self.frame_rate_d,
            AspectRatio: self.aspect_ratio,
            ColorSpace: self.color_space.to_ffi(),
            SampleRate: self.sample_rate,
            Channels: self.channels,
            SamplesPerChannel: self.samples_per_channel,
            Data: self.data.as_ptr() as *mut _,
            DataLength: self.data.len() as i32,
            CompressedData: std::ptr::null_mut(),
            CompressedLength: 0,
            FrameMetadata: std::ptr::null_mut(),
            FrameMetadataLength: 0,
        };

        if let Some(ref metadata) = self.frame_metadata {
            ffi.FrameMetadata = metadata.as_ptr() as *mut _;
            ffi.FrameMetadataLength = metadata.as_bytes_with_nul().len() as i32;
        }

        // SAFETY: We're creating a MediaFrame from a valid FFI structure
        // The data is borrowed from self, which must outlive the MediaFrame
        unsafe { MediaFrame::from_owned_ffi(ffi) }
    }

    /// Returns the frame type.
    pub fn frame_type(&self) -> FrameType {
        self.frame_type
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Sets the timestamp.
    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    /// Returns the codec.
    pub fn codec(&self) -> Codec {
        self.codec
    }

    /// Returns a reference to the frame data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable reference to the frame data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

// SAFETY: All data is owned and properly synchronized
unsafe impl Send for OwnedMediaFrame {}
unsafe impl Sync for OwnedMediaFrame {}
