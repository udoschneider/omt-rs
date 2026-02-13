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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_metadata() {
        let ft = FrameType::from(ffi::OMTFrameType::Metadata);
        assert_eq!(ft, FrameType::Metadata);
    }

    #[test]
    fn test_from_ffi_video() {
        let ft = FrameType::from(ffi::OMTFrameType::Video);
        assert_eq!(ft, FrameType::Video);
    }

    #[test]
    fn test_from_ffi_audio() {
        let ft = FrameType::from(ffi::OMTFrameType::Audio);
        assert_eq!(ft, FrameType::Audio);
    }

    #[test]
    fn test_from_ffi_none() {
        let ft = FrameType::from(ffi::OMTFrameType::None);
        assert_eq!(ft, FrameType::None);
    }

    #[test]
    fn test_to_ffi_metadata() {
        let ffi_ft: ffi::OMTFrameType = FrameType::Metadata.into();
        assert_eq!(ffi_ft as i32, ffi::OMTFrameType::Metadata as i32);
    }

    #[test]
    fn test_to_ffi_video() {
        let ffi_ft: ffi::OMTFrameType = FrameType::Video.into();
        assert_eq!(ffi_ft as i32, ffi::OMTFrameType::Video as i32);
    }

    #[test]
    fn test_to_ffi_audio() {
        let ffi_ft: ffi::OMTFrameType = FrameType::Audio.into();
        assert_eq!(ffi_ft as i32, ffi::OMTFrameType::Audio as i32);
    }

    #[test]
    fn test_to_ffi_none() {
        let ffi_ft: ffi::OMTFrameType = FrameType::None.into();
        assert_eq!(ffi_ft as i32, ffi::OMTFrameType::None as i32);
    }

    #[test]
    fn test_clone() {
        let ft1 = FrameType::Video;
        let ft2 = ft1.clone();
        assert_eq!(ft1, ft2);
    }

    #[test]
    fn test_copy() {
        let ft1 = FrameType::Audio;
        let ft2 = ft1;
        assert_eq!(ft1, FrameType::Audio);
        assert_eq!(ft2, FrameType::Audio);
    }

    #[test]
    fn test_eq() {
        assert_eq!(FrameType::None, FrameType::None);
        assert_eq!(FrameType::Metadata, FrameType::Metadata);
        assert_eq!(FrameType::Video, FrameType::Video);
        assert_eq!(FrameType::Audio, FrameType::Audio);
        assert_ne!(FrameType::Video, FrameType::Audio);
        assert_ne!(FrameType::Metadata, FrameType::None);
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", FrameType::None), "None");
        assert_eq!(format!("{:?}", FrameType::Metadata), "Metadata");
        assert_eq!(format!("{:?}", FrameType::Video), "Video");
        assert_eq!(format!("{:?}", FrameType::Audio), "Audio");
    }
}
