//! Flags for receiver configuration.

/// Flags for receiver configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReceiveFlags(u32);

impl ReceiveFlags {
    /// No flags set.
    pub const NONE: Self = Self(omt_sys::OMTReceiveFlags_None);
    /// Receive only a 1/8th preview of the video.
    pub const PREVIEW: Self = Self(omt_sys::OMTReceiveFlags_Preview);
    /// Include a copy of the compressed VMX1 video frames.
    pub const INCLUDE_COMPRESSED: Self = Self(omt_sys::OMTReceiveFlags_IncludeCompressed);
    /// Include only the compressed VMX1 video frame without decoding.
    pub const COMPRESSED_ONLY: Self = Self(omt_sys::OMTReceiveFlags_CompressedOnly);

    /// Creates flags from raw bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Checks if the flags contain the given flag.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Creates flags from FFI value.
    #[allow(dead_code)] // Inverse of `to_ffi`; kept for API symmetry.
    pub(crate) fn from_ffi(value: u32) -> Self {
        Self(value)
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for ReceiveFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ReceiveFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ReceiveFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
