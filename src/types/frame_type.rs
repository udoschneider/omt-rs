//! Frame type enumeration for Open Media Transport.
//!
//! Defines the types of media frames that can be transmitted over OMT streams.

use crate::ffi;

/// Stream type for frames.
///
/// Specifies the type of media frame, which determines which fields in the frame structure
/// are valid and used.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FrameType {
    /// No frame type
    None,
    /// Metadata frame
    Metadata,
    /// Video frame
    Video,
    /// Audio frame
    Audio,
}

impl From<ffi::OMTFrameType> for FrameType {
    fn from(value: ffi::OMTFrameType) -> Self {
        match value {
            ffi::OMTFrameType::Metadata => FrameType::Metadata,
            ffi::OMTFrameType::Video => FrameType::Video,
            ffi::OMTFrameType::Audio => FrameType::Audio,
            _ => FrameType::None,
        }
    }
}

impl From<FrameType> for ffi::OMTFrameType {
    fn from(value: FrameType) -> Self {
        match value {
            FrameType::Metadata => ffi::OMTFrameType::Metadata,
            FrameType::Video => ffi::OMTFrameType::Video,
            FrameType::Audio => ffi::OMTFrameType::Audio,
            FrameType::None => ffi::OMTFrameType::None,
        }
    }
}
