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
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        Self::from_bits(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.bits()
    }
}
