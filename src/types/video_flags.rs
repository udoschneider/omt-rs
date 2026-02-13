//! Video frame flags for Open Media Transport.
//!
//! Defines bitflags for video frame properties and attributes such as interlacing,
//! alpha channels, and bit depth.

use crate::ffi;
use bitflags::bitflags;

bitflags! {
    /// Video frame properties and attributes.
    ///
    /// # Flags
    ///
    /// * `INTERLACED` - Frames are interlaced
    /// * `ALPHA` - Frames contain an alpha channel. If not set, BGRA will be encoded as BGRX
    ///   and UYVA will be encoded as UYVY.
    /// * `PREMULTIPLIED` - When combined with ALPHA, alpha channel is premultiplied, otherwise straight
    /// * `PREVIEW` - Frame is a special 1/8th preview frame
    /// * `HIGH_BIT_DEPTH` - Sender automatically adds this flag for frames encoded using P216
    ///   or PA16 pixel formats. Set this manually for VMX1 compressed data where the frame was
    ///   originally encoded using P216 or PA16. This determines which pixel format is selected
    ///   on the decode side.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[allow(non_snake_case)]
    pub struct VideoFlags: i32 {
        const NONE = 0;
        const INTERLACED = 1;
        const ALPHA = 2;
        const PREMULTIPLIED = 4;
        const PREVIEW = 8;
        const HIGH_BIT_DEPTH = 16;
    }
}

impl From<ffi::OMTVideoFlags> for VideoFlags {
    fn from(value: ffi::OMTVideoFlags) -> Self {
        VideoFlags::from_bits_truncate(value)
    }
}

impl From<VideoFlags> for i32 {
    fn from(value: VideoFlags) -> Self {
        value.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_none() {
        let flags = VideoFlags::from(0);
        assert_eq!(flags, VideoFlags::NONE);
    }

    #[test]
    fn test_from_ffi_interlaced() {
        let flags = VideoFlags::from(1);
        assert_eq!(flags, VideoFlags::INTERLACED);
    }

    #[test]
    fn test_from_ffi_alpha() {
        let flags = VideoFlags::from(2);
        assert_eq!(flags, VideoFlags::ALPHA);
    }

    #[test]
    fn test_from_ffi_premultiplied() {
        let flags = VideoFlags::from(4);
        assert_eq!(flags, VideoFlags::PREMULTIPLIED);
    }

    #[test]
    fn test_from_ffi_preview() {
        let flags = VideoFlags::from(8);
        assert_eq!(flags, VideoFlags::PREVIEW);
    }

    #[test]
    fn test_from_ffi_high_bit_depth() {
        let flags = VideoFlags::from(16);
        assert_eq!(flags, VideoFlags::HIGH_BIT_DEPTH);
    }

    #[test]
    fn test_to_i32_none() {
        let flags = VideoFlags::NONE;
        let val: i32 = flags.into();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_to_i32_interlaced() {
        let flags = VideoFlags::INTERLACED;
        let val: i32 = flags.into();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_to_i32_alpha() {
        let flags = VideoFlags::ALPHA;
        let val: i32 = flags.into();
        assert_eq!(val, 2);
    }

    #[test]
    fn test_to_i32_premultiplied() {
        let flags = VideoFlags::PREMULTIPLIED;
        let val: i32 = flags.into();
        assert_eq!(val, 4);
    }

    #[test]
    fn test_to_i32_preview() {
        let flags = VideoFlags::PREVIEW;
        let val: i32 = flags.into();
        assert_eq!(val, 8);
    }

    #[test]
    fn test_to_i32_high_bit_depth() {
        let flags = VideoFlags::HIGH_BIT_DEPTH;
        let val: i32 = flags.into();
        assert_eq!(val, 16);
    }

    #[test]
    fn test_bitflag_union() {
        let flags = VideoFlags::ALPHA | VideoFlags::PREMULTIPLIED;
        assert!(flags.contains(VideoFlags::ALPHA));
        assert!(flags.contains(VideoFlags::PREMULTIPLIED));
        assert!(!flags.contains(VideoFlags::INTERLACED));
    }

    #[test]
    fn test_bitflag_intersection() {
        let flags1 = VideoFlags::ALPHA | VideoFlags::INTERLACED;
        let flags2 = VideoFlags::ALPHA | VideoFlags::PREVIEW;
        let intersection = flags1 & flags2;
        assert_eq!(intersection, VideoFlags::ALPHA);
    }

    #[test]
    fn test_bitflag_contains() {
        let flags = VideoFlags::ALPHA | VideoFlags::HIGH_BIT_DEPTH;
        assert!(flags.contains(VideoFlags::ALPHA));
        assert!(flags.contains(VideoFlags::HIGH_BIT_DEPTH));
        assert!(!flags.contains(VideoFlags::PREVIEW));
        assert!(!flags.contains(VideoFlags::INTERLACED));
    }

    #[test]
    fn test_bitflag_is_empty() {
        assert!(VideoFlags::NONE.is_empty());
        assert!(!VideoFlags::ALPHA.is_empty());
    }

    #[test]
    fn test_bitflag_all() {
        let all = VideoFlags::all();
        assert!(all.contains(VideoFlags::INTERLACED));
        assert!(all.contains(VideoFlags::ALPHA));
        assert!(all.contains(VideoFlags::PREMULTIPLIED));
        assert!(all.contains(VideoFlags::PREVIEW));
        assert!(all.contains(VideoFlags::HIGH_BIT_DEPTH));
    }

    #[test]
    fn test_clone() {
        let flags1 = VideoFlags::ALPHA;
        let flags2 = flags1.clone();
        assert_eq!(flags1, flags2);
    }

    #[test]
    fn test_copy() {
        let flags1 = VideoFlags::INTERLACED;
        let flags2 = flags1;
        assert_eq!(flags1, VideoFlags::INTERLACED);
        assert_eq!(flags2, VideoFlags::INTERLACED);
    }

    #[test]
    fn test_eq() {
        assert_eq!(VideoFlags::NONE, VideoFlags::NONE);
        assert_eq!(VideoFlags::ALPHA, VideoFlags::ALPHA);
        assert_ne!(VideoFlags::ALPHA, VideoFlags::PREVIEW);
    }

    #[test]
    fn test_debug() {
        let flags = VideoFlags::ALPHA | VideoFlags::PREMULTIPLIED;
        let debug_str = format!("{:?}", flags);
        assert!(debug_str.contains("ALPHA"));
        assert!(debug_str.contains("PREMULTIPLIED"));
    }

    #[test]
    fn test_from_bits_truncate() {
        // Test with unknown bits set (should be truncated)
        let flags = VideoFlags::from_bits_truncate(0xFF);
        let val: i32 = flags.into();
        assert_eq!(val, 0x1F); // Only known bits (1 | 2 | 4 | 8 | 16)
    }

    #[test]
    fn test_combined_flags() {
        let flags = VideoFlags::ALPHA | VideoFlags::PREMULTIPLIED | VideoFlags::HIGH_BIT_DEPTH;
        let val: i32 = flags.into();
        assert_eq!(val, 22); // 2 | 4 | 16
    }
}
