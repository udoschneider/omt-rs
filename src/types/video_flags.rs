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
