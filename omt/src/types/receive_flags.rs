//! Flags for receiver configuration.

use bitflags::bitflags;

bitflags! {
    /// Flags for receiver configuration.
    ///
    /// This is a bitflags type that can be combined using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ReceiveFlags: u32 {
        /// No flags set.
        const NONE = omt_sys::OMTReceiveFlags_None;
        /// Receive only a 1/8th preview of the video.
        const PREVIEW = omt_sys::OMTReceiveFlags_Preview;
        /// Include a copy of the compressed VMX1 video frames.
        const INCLUDE_COMPRESSED = omt_sys::OMTReceiveFlags_IncludeCompressed;
        /// Include only the compressed VMX1 video frame without decoding.
        const COMPRESSED_ONLY = omt_sys::OMTReceiveFlags_CompressedOnly;
    }
}

impl ReceiveFlags {
    /// Creates flags from FFI value.
    ///
    /// Unknown bits are retained rather than rejected: the C library may gain
    /// flags this crate does not know about yet, and dropping them would
    /// silently change the caller's request.
    #[allow(dead_code)] // Inverse of `to_ffi`; kept for API symmetry.
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
        let flags = ReceiveFlags::PREVIEW | ReceiveFlags::INCLUDE_COMPRESSED;

        assert!(flags.contains(ReceiveFlags::PREVIEW));
        assert!(flags.contains(ReceiveFlags::INCLUDE_COMPRESSED));
        assert!(!flags.contains(ReceiveFlags::COMPRESSED_ONLY));
        assert!(ReceiveFlags::NONE.is_empty());
    }

    #[test]
    fn ffi_roundtrip_preserves_unknown_bits() {
        let flags = ReceiveFlags::PREVIEW | ReceiveFlags::COMPRESSED_ONLY;
        assert_eq!(ReceiveFlags::from_ffi(flags.to_ffi()), flags);

        // A bit this crate does not model must survive the round trip.
        let future = ReceiveFlags::from_ffi(0x8000_0000);
        assert_eq!(future.to_ffi(), 0x8000_0000);
    }
}
