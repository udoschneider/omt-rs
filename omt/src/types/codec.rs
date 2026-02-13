//! Codec type definitions and utilities.

/// Media codec types supported by OMT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Codec {
    /// VMX1 - Fast video codec.
    Vmx1 = omt_sys::OMTCodec_VMX1,
    /// FPA1 - Floating-point Planar Audio 32bit.
    Fpa1 = omt_sys::OMTCodec_FPA1,
    /// UYVY - 16bpp YUV format.
    Uyvy = omt_sys::OMTCodec_UYVY,
    /// YUY2 - 16bpp YUV format YUYV pixel order.
    Yuy2 = omt_sys::OMTCodec_YUY2,
    /// BGRA - 32bpp RGBA format (Same as ARGB32 on Win32).
    Bgra = omt_sys::OMTCodec_BGRA,
    /// NV12 - Planar 4:2:0 YUV format. Y plane followed by interleaved half height U/V plane.
    Nv12 = omt_sys::OMTCodec_NV12,
    /// YV12 - Planar 4:2:0 YUV format. Y plane followed by half height U and V planes.
    Yv12 = omt_sys::OMTCodec_YV12,
    /// UYVA - 16pp YUV format immediately followed by an alpha plane.
    Uyva = omt_sys::OMTCodec_UYVA,
    /// P216 - Planar 4:2:2 YUV format. 16bit Y plane followed by interlaved 16bit UV plane.
    P216 = omt_sys::OMTCodec_P216,
    /// PA16 - Same as P216 followed by an additional 16bit alpha plane.
    Pa16 = omt_sys::OMTCodec_PA16,
}

impl Codec {
    /// Creates a `Codec` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTCodec_VMX1 => Some(Self::Vmx1),
            omt_sys::OMTCodec_FPA1 => Some(Self::Fpa1),
            omt_sys::OMTCodec_UYVY => Some(Self::Uyvy),
            omt_sys::OMTCodec_YUY2 => Some(Self::Yuy2),
            omt_sys::OMTCodec_BGRA => Some(Self::Bgra),
            omt_sys::OMTCodec_NV12 => Some(Self::Nv12),
            omt_sys::OMTCodec_YV12 => Some(Self::Yv12),
            omt_sys::OMTCodec_UYVA => Some(Self::Uyva),
            omt_sys::OMTCodec_P216 => Some(Self::P216),
            omt_sys::OMTCodec_PA16 => Some(Self::Pa16),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }

    /// Returns true if this is a video codec.
    pub fn is_video(&self) -> bool {
        !matches!(self, Codec::Fpa1)
    }

    /// Returns true if this is an audio codec.
    pub fn is_audio(&self) -> bool {
        matches!(self, Codec::Fpa1)
    }

    /// Returns true if this codec supports alpha channel.
    pub fn supports_alpha(&self) -> bool {
        matches!(self, Codec::Bgra | Codec::Uyva | Codec::Pa16)
    }

    /// Returns true if this is a high bit depth codec.
    pub fn is_high_bit_depth(&self) -> bool {
        matches!(self, Codec::P216 | Codec::Pa16)
    }

    /// Returns true if this is a compressed codec.
    pub fn is_compressed(&self) -> bool {
        matches!(self, Codec::Vmx1)
    }

    /// Returns the bits per pixel for this codec (video only).
    ///
    /// Returns `None` for audio codecs.
    pub fn bits_per_pixel(&self) -> Option<u32> {
        match self {
            Codec::Uyvy | Codec::Yuy2 | Codec::Uyva => Some(16),
            Codec::Bgra => Some(32),
            Codec::Nv12 | Codec::Yv12 => Some(12), // 4:2:0 subsampling
            Codec::P216 | Codec::Pa16 => Some(32), // 16-bit per component
            Codec::Vmx1 => None,                   // Compressed, variable
            Codec::Fpa1 => None,                   // Audio codec
        }
    }

    /// Returns the FourCC code as a string.
    pub fn fourcc(&self) -> &'static str {
        match self {
            Codec::Vmx1 => "VMX1",
            Codec::Fpa1 => "FPA1",
            Codec::Uyvy => "UYVY",
            Codec::Yuy2 => "YUY2",
            Codec::Bgra => "BGRA",
            Codec::Nv12 => "NV12",
            Codec::Yv12 => "YV12",
            Codec::Uyva => "UYVA",
            Codec::P216 => "P216",
            Codec::Pa16 => "PA16",
        }
    }
}

impl std::fmt::Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fourcc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_properties() {
        assert!(Codec::Fpa1.is_audio());
        assert!(!Codec::Uyvy.is_audio());

        assert!(Codec::Uyvy.is_video());
        assert!(!Codec::Fpa1.is_video());

        assert!(Codec::Bgra.supports_alpha());
        assert!(Codec::Uyva.supports_alpha());
        assert!(!Codec::Uyvy.supports_alpha());

        assert!(Codec::P216.is_high_bit_depth());
        assert!(!Codec::Uyvy.is_high_bit_depth());

        assert!(Codec::Vmx1.is_compressed());
        assert!(!Codec::Uyvy.is_compressed());
    }

    #[test]
    fn test_bits_per_pixel() {
        assert_eq!(Codec::Uyvy.bits_per_pixel(), Some(16));
        assert_eq!(Codec::Bgra.bits_per_pixel(), Some(32));
        assert_eq!(Codec::Nv12.bits_per_pixel(), Some(12));
        assert_eq!(Codec::Fpa1.bits_per_pixel(), None);
    }

    #[test]
    fn test_fourcc() {
        assert_eq!(Codec::Uyvy.fourcc(), "UYVY");
        assert_eq!(Codec::Bgra.fourcc(), "BGRA");
        assert_eq!(Codec::Vmx1.fourcc(), "VMX1");
    }
}
