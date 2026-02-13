//! Media frame types for video, audio, and metadata.

use crate::error::{Error, Result};
use crate::types::{Codec, ColorSpace, FrameType, VideoFlags};
use std::slice;

/// A media frame containing video, audio, or metadata.
///
/// This is a safe wrapper around the FFI `OMTMediaFrame` structure.
/// The frame data is valid until the next receive call or until the frame is dropped.
#[derive(Debug)]

pub struct MediaFrame {
    ffi: omt_sys::OMTMediaFrame,
}

impl MediaFrame {
    /// Creates a new zeroed media frame.
    pub(crate) fn new() -> Self {
        Self {
            ffi: unsafe { std::mem::zeroed() },
        }
    }

    /// Creates a frame from an FFI pointer (receive only).
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a properly initialized OMTMediaFrame.
    pub(crate) unsafe fn from_ffi_ptr(ptr: *const omt_sys::OMTMediaFrame) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ffi: unsafe { *ptr },
            })
        }
    }

    /// Returns a reference to the underlying FFI structure.
    pub(crate) fn as_ffi(&self) -> &omt_sys::OMTMediaFrame {
        &self.ffi
    }

    /// Returns a mutable reference to the underlying FFI structure.
    pub(crate) fn as_ffi_mut(&mut self) -> &mut omt_sys::OMTMediaFrame {
        &mut self.ffi
    }

    /// Returns the frame type.
    pub fn frame_type(&self) -> FrameType {
        FrameType::from_ffi(self.ffi.Type).unwrap_or(FrameType::NONE)
    }

    /// Returns the timestamp (where 1 second = 10,000,000 units).
    ///
    /// A value of -1 indicates auto-generated timestamps.
    pub fn timestamp(&self) -> i64 {
        self.ffi.Timestamp
    }

    /// Returns the codec.
    pub fn codec(&self) -> Option<Codec> {
        Codec::from_ffi(self.ffi.Codec)
    }

    /// Returns the frame data as a byte slice.
    pub fn data(&self) -> &[u8] {
        if self.ffi.Data.is_null() || self.ffi.DataLength <= 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
            }
        }
    }

    /// Returns the compressed data (VMX1) if available.
    pub fn compressed_data(&self) -> &[u8] {
        if self.ffi.CompressedData.is_null() || self.ffi.CompressedLength <= 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(
                    self.ffi.CompressedData as *const u8,
                    self.ffi.CompressedLength as usize,
                )
            }
        }
    }

    /// Returns the per-frame metadata if available.
    pub fn frame_metadata(&self) -> &[u8] {
        if self.ffi.FrameMetadata.is_null() || self.ffi.FrameMetadataLength <= 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(
                    self.ffi.FrameMetadata as *const u8,
                    self.ffi.FrameMetadataLength as usize,
                )
            }
        }
    }
}

/// A video frame with all video-specific properties.
#[derive(Debug)]
pub struct VideoFrame {
    frame: MediaFrame,
}

impl VideoFrame {
    /// Creates a video frame from a generic MediaFrame.
    pub(crate) fn from_media_frame(frame: MediaFrame) -> Result<Self> {
        if frame.frame_type() != FrameType::VIDEO {
            return Err(Error::InvalidFrameType);
        }
        Ok(Self { frame })
    }

    /// Returns the video width in pixels.
    pub fn width(&self) -> i32 {
        self.frame.ffi.Width
    }

    /// Returns the video height in pixels.
    pub fn height(&self) -> i32 {
        self.frame.ffi.Height
    }

    /// Returns the stride (row pitch) in bytes.
    pub fn stride(&self) -> i32 {
        self.frame.ffi.Stride
    }

    /// Returns the video flags.
    pub fn flags(&self) -> VideoFlags {
        VideoFlags::from_ffi(self.frame.ffi.Flags)
    }

    /// Returns the frame rate numerator.
    pub fn frame_rate_numerator(&self) -> i32 {
        self.frame.ffi.FrameRateN
    }

    /// Returns the frame rate denominator.
    pub fn frame_rate_denominator(&self) -> i32 {
        self.frame.ffi.FrameRateD
    }

    /// Returns the frame rate as a floating point value.
    pub fn frame_rate(&self) -> f64 {
        if self.frame.ffi.FrameRateD != 0 {
            self.frame.ffi.FrameRateN as f64 / self.frame.ffi.FrameRateD as f64
        } else {
            0.0
        }
    }

    /// Returns the display aspect ratio.
    pub fn aspect_ratio(&self) -> f32 {
        self.frame.ffi.AspectRatio
    }

    /// Returns the color space.
    pub fn color_space(&self) -> Option<ColorSpace> {
        ColorSpace::from_ffi(self.frame.ffi.ColorSpace)
    }

    /// Returns the codec.
    pub fn codec(&self) -> Option<Codec> {
        self.frame.codec()
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> i64 {
        self.frame.timestamp()
    }

    /// Returns the pixel data.
    pub fn data(&self) -> &[u8] {
        self.frame.data()
    }

    /// Returns the compressed VMX1 data if available.
    pub fn compressed_data(&self) -> &[u8] {
        self.frame.compressed_data()
    }

    /// Returns the per-frame metadata if available.
    pub fn frame_metadata(&self) -> &[u8] {
        self.frame.frame_metadata()
    }
}

/// An audio frame with all audio-specific properties.
#[derive(Debug)]
pub struct AudioFrame {
    frame: MediaFrame,
}

impl AudioFrame {
    /// Creates an audio frame from a generic MediaFrame.
    pub(crate) fn from_media_frame(frame: MediaFrame) -> Result<Self> {
        if frame.frame_type() != FrameType::AUDIO {
            return Err(Error::InvalidFrameType);
        }
        Ok(Self { frame })
    }

    /// Returns the sample rate (e.g., 48000, 44100).
    pub fn sample_rate(&self) -> i32 {
        self.frame.ffi.SampleRate
    }

    /// Returns the number of audio channels (maximum 32).
    pub fn channels(&self) -> i32 {
        self.frame.ffi.Channels
    }

    /// Returns the number of samples per channel.
    pub fn samples_per_channel(&self) -> i32 {
        self.frame.ffi.SamplesPerChannel
    }

    /// Returns the codec (should be FPA1 for audio).
    pub fn codec(&self) -> Option<Codec> {
        self.frame.codec()
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> i64 {
        self.frame.timestamp()
    }

    /// Returns the planar 32-bit floating point audio data.
    pub fn data(&self) -> &[u8] {
        self.frame.data()
    }

    /// Returns the audio data as f32 slices (one per channel).
    ///
    /// Each slice contains `samples_per_channel` samples.
    pub fn as_f32_planar(&self) -> Vec<&[f32]> {
        let data = self.data();
        let samples_per_channel = self.samples_per_channel() as usize;
        let channels = self.channels() as usize;
        let samples_per_plane = samples_per_channel * std::mem::size_of::<f32>();

        let mut result = Vec::with_capacity(channels);
        for ch in 0..channels {
            let offset = ch * samples_per_plane;
            if offset + samples_per_plane <= data.len() {
                let plane_data = &data[offset..offset + samples_per_plane];
                let f32_slice = unsafe {
                    slice::from_raw_parts(plane_data.as_ptr() as *const f32, samples_per_channel)
                };
                result.push(f32_slice);
            }
        }
        result
    }
}

/// A metadata frame containing UTF-8 encoded XML.
#[derive(Debug)]
pub struct MetadataFrame {
    frame: MediaFrame,
}

impl MetadataFrame {
    /// Creates a metadata frame from a generic MediaFrame.
    pub(crate) fn from_media_frame(frame: MediaFrame) -> Result<Self> {
        if frame.frame_type() != FrameType::METADATA {
            return Err(Error::InvalidFrameType);
        }
        Ok(Self { frame })
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> i64 {
        self.frame.timestamp()
    }

    /// Returns the metadata as a byte slice.
    pub fn data(&self) -> &[u8] {
        self.frame.data()
    }

    /// Returns the metadata as a UTF-8 string.
    pub fn as_str(&self) -> Result<&str> {
        let data = self.data();
        // Remove null terminator if present
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        std::str::from_utf8(&data[..end]).map_err(|_| Error::InvalidUtf8)
    }
}

unsafe impl Send for MediaFrame {}
