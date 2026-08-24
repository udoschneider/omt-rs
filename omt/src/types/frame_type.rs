//! Frame type definitions.

use bitflags::bitflags;

bitflags! {
    /// Type of media frame.
    ///
    /// This is a bitflags type that can be combined using bitwise OR operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameType: u32 {
        /// No frame type.
        const NONE = omt_sys::OMTFrameType_None;
        /// Metadata frame.
        const METADATA = omt_sys::OMTFrameType_Metadata;
        /// Video frame.
        const VIDEO = omt_sys::OMTFrameType_Video;
        /// Audio frame.
        const AUDIO = omt_sys::OMTFrameType_Audio;
    }
}

impl FrameType {
    /// All frame types (Video, Audio, and Metadata).
    pub const ALL: Self = Self::VIDEO.union(Self::AUDIO).union(Self::METADATA);

    /// Video and Audio frames.
    pub const VIDEO_AUDIO: Self = Self::VIDEO.union(Self::AUDIO);

    /// Creates a `FrameType` from raw FFI value.
    ///
    /// Unknown bits are retained rather than rejected, matching
    /// [`ReceiveFlags`](crate::ReceiveFlags) and [`VideoFlags`](crate::VideoFlags):
    /// the C library may gain frame types this crate does not model yet, and
    /// dropping them would misreport the frame as having no type at all.
    pub(crate) fn from_ffi(value: u32) -> Self {
        Self::from_bits_retain(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_and_tests_flags() {
        assert!(FrameType::ALL.contains(FrameType::VIDEO));
        assert!(FrameType::ALL.contains(FrameType::AUDIO));
        assert!(FrameType::ALL.contains(FrameType::METADATA));
        assert!(!FrameType::VIDEO_AUDIO.contains(FrameType::METADATA));
        assert!(FrameType::NONE.is_empty());
    }

    #[test]
    fn ffi_roundtrip_preserves_unknown_bits() {
        let flags = FrameType::VIDEO | FrameType::AUDIO;
        assert_eq!(FrameType::from_ffi(flags.to_ffi()), flags);

        // A frame type this crate does not model must survive the round trip
        // rather than collapsing to `NONE`.
        let future = FrameType::from_ffi(0x8000_0000);
        assert_eq!(future.to_ffi(), 0x8000_0000);
        assert!(!future.is_empty());
    }
}
