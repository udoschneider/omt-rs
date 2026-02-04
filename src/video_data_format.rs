/// Requested output format for video data conversion.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VideoDataFormat {
    /// 8-bit per component RGB.
    RGB,
    /// 8-bit per component RGBA, straight alpha.
    RGBAs,
    /// 8-bit per component RGBA, premultiplied alpha.
    RGBAp,
    /// 16-bit per component RGB.
    RGB16,
    /// 16-bit per component RGBA, straight alpha.
    RGBAs16,
    /// 16-bit per component RGBA, premultiplied alpha.
    RGBAp16,
}

/// Alias for `VideoDataFormat::RGBAs`.
pub const RGBA: VideoDataFormat = VideoDataFormat::RGBAs;

/// Alias for `VideoDataFormat::RGBAs16`.
pub const RGBA16: VideoDataFormat = VideoDataFormat::RGBAs16;
