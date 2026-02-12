//! Quality settings for video encoding in Open Media Transport.
//!
//! Defines compression quality levels that can be negotiated between senders and receivers.

use crate::ffi;

/// Compression quality setting for video encoding.
///
/// If set to `Default`, the Sender is configured to allow suggestions from all Receivers.
/// The highest suggestion amongst all receivers is then selected.
///
/// If a Receiver is set to `Default`, then it will defer the quality to whatever is set
/// amongst other Receivers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Quality {
    /// Default quality (0) - defers to other receivers' suggestions
    Default,
    /// Low quality (1) - minimal compression overhead
    Low,
    /// Medium quality (50) - balanced compression
    Medium,
    /// High quality (100) - maximum quality, higher compression overhead
    High,
}

impl From<ffi::OMTQuality> for Quality {
    fn from(value: ffi::OMTQuality) -> Self {
        match value {
            ffi::OMTQuality::Low => Quality::Low,
            ffi::OMTQuality::Medium => Quality::Medium,
            ffi::OMTQuality::High => Quality::High,
            _ => Quality::Default,
        }
    }
}

impl From<Quality> for ffi::OMTQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Low => ffi::OMTQuality::Low,
            Quality::Medium => ffi::OMTQuality::Medium,
            Quality::High => ffi::OMTQuality::High,
            Quality::Default => ffi::OMTQuality::Default,
        }
    }
}
