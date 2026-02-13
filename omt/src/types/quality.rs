//! Video encoding quality level definitions.

/// Video encoding quality level.
///
/// If set to `Default`, the Sender is configured to allow suggestions from all Receivers.
/// The highest suggest amongst all receivers is then selected.
///
/// If a Receiver is set to `Default`, then it will defer the quality to whatever is set
/// amongst other Receivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Quality {
    /// Default quality (allows receiver suggestions).
    Default = omt_sys::OMTQuality_Default,
    /// Low quality encoding.
    Low = omt_sys::OMTQuality_Low,
    /// Medium quality encoding.
    Medium = omt_sys::OMTQuality_Medium,
    /// High quality encoding.
    High = omt_sys::OMTQuality_High,
}

impl Quality {
    /// Creates a `Quality` from raw FFI value.
    pub(crate) fn from_ffi(value: u32) -> Option<Self> {
        match value {
            omt_sys::OMTQuality_Default => Some(Self::Default),
            omt_sys::OMTQuality_Low => Some(Self::Low),
            omt_sys::OMTQuality_Medium => Some(Self::Medium),
            omt_sys::OMTQuality_High => Some(Self::High),
            _ => None,
        }
    }

    /// Converts to FFI value.
    pub(crate) fn to_ffi(self) -> u32 {
        self as u32
    }
}
