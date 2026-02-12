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
