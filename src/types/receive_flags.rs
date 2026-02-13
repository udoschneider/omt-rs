//! Receiver configuration flags for Open Media Transport.
//!
//! Defines bitflags for receiver features such as preview mode and compressed frame handling.

use crate::ffi;
use bitflags::bitflags;

bitflags! {
    /// Receiver configuration flags.
    ///
    /// Flags to enable certain features on a Receiver.
    ///
    /// # Flags
    ///
    /// * `PREVIEW` - Receive only a 1/8th preview of the video
    /// * `INCLUDE_COMPRESSED` - Include a copy of the compressed VMX1 video frames for
    ///   further processing or recording
    /// * `COMPRESSED_ONLY` - Include only the compressed VMX1 video frame without decoding.
    ///   In this instance DataLength will always be 0.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[allow(non_snake_case)]
    pub struct ReceiveFlags: i32 {
        const NONE = 0;
        const PREVIEW = 1;
        const INCLUDE_COMPRESSED = 2;
        const COMPRESSED_ONLY = 4;
    }
}

impl From<ffi::OMTReceiveFlags> for ReceiveFlags {
    fn from(value: ffi::OMTReceiveFlags) -> Self {
        ReceiveFlags::from_bits_truncate(value)
    }
}

impl From<ReceiveFlags> for i32 {
    fn from(value: ReceiveFlags) -> Self {
        value.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_none() {
        let flags = ReceiveFlags::from(0);
        assert_eq!(flags, ReceiveFlags::NONE);
    }

    #[test]
    fn test_from_ffi_preview() {
        let flags = ReceiveFlags::from(1);
        assert_eq!(flags, ReceiveFlags::PREVIEW);
    }

    #[test]
    fn test_from_ffi_include_compressed() {
        let flags = ReceiveFlags::from(2);
        assert_eq!(flags, ReceiveFlags::INCLUDE_COMPRESSED);
    }

    #[test]
    fn test_from_ffi_compressed_only() {
        let flags = ReceiveFlags::from(4);
        assert_eq!(flags, ReceiveFlags::COMPRESSED_ONLY);
    }

    #[test]
    fn test_to_i32_none() {
        let flags = ReceiveFlags::NONE;
        let val: i32 = flags.into();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_to_i32_preview() {
        let flags = ReceiveFlags::PREVIEW;
        let val: i32 = flags.into();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_to_i32_include_compressed() {
        let flags = ReceiveFlags::INCLUDE_COMPRESSED;
        let val: i32 = flags.into();
        assert_eq!(val, 2);
    }

    #[test]
    fn test_to_i32_compressed_only() {
        let flags = ReceiveFlags::COMPRESSED_ONLY;
        let val: i32 = flags.into();
        assert_eq!(val, 4);
    }

    #[test]
    fn test_bitflag_union() {
        let flags = ReceiveFlags::PREVIEW | ReceiveFlags::INCLUDE_COMPRESSED;
        assert!(flags.contains(ReceiveFlags::PREVIEW));
        assert!(flags.contains(ReceiveFlags::INCLUDE_COMPRESSED));
        assert!(!flags.contains(ReceiveFlags::COMPRESSED_ONLY));
    }

    #[test]
    fn test_bitflag_intersection() {
        let flags1 = ReceiveFlags::PREVIEW | ReceiveFlags::INCLUDE_COMPRESSED;
        let flags2 = ReceiveFlags::PREVIEW | ReceiveFlags::COMPRESSED_ONLY;
        let intersection = flags1 & flags2;
        assert_eq!(intersection, ReceiveFlags::PREVIEW);
    }

    #[test]
    fn test_bitflag_contains() {
        let flags = ReceiveFlags::PREVIEW | ReceiveFlags::INCLUDE_COMPRESSED;
        assert!(flags.contains(ReceiveFlags::PREVIEW));
        assert!(flags.contains(ReceiveFlags::INCLUDE_COMPRESSED));
        assert!(!flags.contains(ReceiveFlags::COMPRESSED_ONLY));
        // Note: bitflags always contains NONE (empty set is subset of any set)
    }

    #[test]
    fn test_bitflag_is_empty() {
        assert!(ReceiveFlags::NONE.is_empty());
        assert!(!ReceiveFlags::PREVIEW.is_empty());
    }

    #[test]
    fn test_bitflag_all() {
        let all = ReceiveFlags::all();
        assert!(all.contains(ReceiveFlags::PREVIEW));
        assert!(all.contains(ReceiveFlags::INCLUDE_COMPRESSED));
        assert!(all.contains(ReceiveFlags::COMPRESSED_ONLY));
    }

    #[test]
    fn test_clone() {
        let flags1 = ReceiveFlags::PREVIEW;
        let flags2 = flags1.clone();
        assert_eq!(flags1, flags2);
    }

    #[test]
    fn test_copy() {
        let flags1 = ReceiveFlags::INCLUDE_COMPRESSED;
        let flags2 = flags1;
        assert_eq!(flags1, ReceiveFlags::INCLUDE_COMPRESSED);
        assert_eq!(flags2, ReceiveFlags::INCLUDE_COMPRESSED);
    }

    #[test]
    fn test_eq() {
        assert_eq!(ReceiveFlags::NONE, ReceiveFlags::NONE);
        assert_eq!(ReceiveFlags::PREVIEW, ReceiveFlags::PREVIEW);
        assert_ne!(ReceiveFlags::PREVIEW, ReceiveFlags::COMPRESSED_ONLY);
    }

    #[test]
    fn test_debug() {
        let flags = ReceiveFlags::PREVIEW | ReceiveFlags::INCLUDE_COMPRESSED;
        let debug_str = format!("{:?}", flags);
        assert!(debug_str.contains("PREVIEW"));
        assert!(debug_str.contains("INCLUDE_COMPRESSED"));
    }

    #[test]
    fn test_from_bits_truncate() {
        // Test with unknown bits set (should be truncated)
        let flags = ReceiveFlags::from_bits_truncate(0xFF);
        let val: i32 = flags.into();
        assert_eq!(val, 0x7); // Only known bits (1 | 2 | 4)
    }
}
