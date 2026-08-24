//! Flags for video frames.

use bitflags::bitflags;

bitflags! {
    /// Flags for video frames.
    ///
    /// This is a bitflags type that can be combined using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VideoFlags: u32 {
        /// No flags set.
        const NONE = omt_sys::OMTVideoFlags_None;
        /// Frames are interlaced.
        const INTERLACED = omt_sys::OMTVideoFlags_Interlaced;
        /// Frames contain an alpha channel.
        const ALPHA = omt_sys::OMTVideoFlags_Alpha;
        /// Alpha channel is premultiplied (when combined with ALPHA).
        const PRE_MULTIPLIED = omt_sys::OMTVideoFlags_PreMultiplied;
        /// Frame is a special 1/8th preview frame.
        const PREVIEW = omt_sys::OMTVideoFlags_Preview;
        /// High bit depth frame (P216 or PA16 formats).
        const HIGH_BIT_DEPTH = omt_sys::OMTVideoFlags_HighBitDepth;
    }
}

impl VideoFlags {
    /// Creates flags from FFI value.
    ///
    /// Unknown bits are retained rather than rejected: a sender may set flags
    /// this crate does not model yet, and dropping them would misreport the
    /// frame.
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
        let flags = VideoFlags::ALPHA | VideoFlags::PRE_MULTIPLIED;

        assert!(flags.contains(VideoFlags::ALPHA));
        assert!(flags.contains(VideoFlags::PRE_MULTIPLIED));
        assert!(!flags.contains(VideoFlags::INTERLACED));
        assert!(VideoFlags::NONE.is_empty());
    }

    #[test]
    fn ffi_roundtrip_preserves_unknown_bits() {
        let flags = VideoFlags::INTERLACED | VideoFlags::HIGH_BIT_DEPTH;
        assert_eq!(VideoFlags::from_ffi(flags.to_ffi()), flags);

        // A bit this crate does not model must survive the round trip.
        let future = VideoFlags::from_ffi(0x8000_0000);
        assert_eq!(future.to_ffi(), 0x8000_0000);
    }
}
