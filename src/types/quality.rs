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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ffi_default() {
        let quality = Quality::from(ffi::OMTQuality::Default);
        assert_eq!(quality, Quality::Default);
    }

    #[test]
    fn test_from_ffi_low() {
        let quality = Quality::from(ffi::OMTQuality::Low);
        assert_eq!(quality, Quality::Low);
    }

    #[test]
    fn test_from_ffi_medium() {
        let quality = Quality::from(ffi::OMTQuality::Medium);
        assert_eq!(quality, Quality::Medium);
    }

    #[test]
    fn test_from_ffi_high() {
        let quality = Quality::from(ffi::OMTQuality::High);
        assert_eq!(quality, Quality::High);
    }

    #[test]
    fn test_to_ffi_default() {
        let ffi_quality: ffi::OMTQuality = Quality::Default.into();
        assert_eq!(ffi_quality as i32, ffi::OMTQuality::Default as i32);
    }

    #[test]
    fn test_to_ffi_low() {
        let ffi_quality: ffi::OMTQuality = Quality::Low.into();
        assert_eq!(ffi_quality as i32, ffi::OMTQuality::Low as i32);
    }

    #[test]
    fn test_to_ffi_medium() {
        let ffi_quality: ffi::OMTQuality = Quality::Medium.into();
        assert_eq!(ffi_quality as i32, ffi::OMTQuality::Medium as i32);
    }

    #[test]
    fn test_to_ffi_high() {
        let ffi_quality: ffi::OMTQuality = Quality::High.into();
        assert_eq!(ffi_quality as i32, ffi::OMTQuality::High as i32);
    }

    #[test]
    fn test_clone() {
        let q1 = Quality::High;
        let q2 = q1.clone();
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_copy() {
        let q1 = Quality::Medium;
        let q2 = q1;
        assert_eq!(q1, Quality::Medium);
        assert_eq!(q2, Quality::Medium);
    }

    #[test]
    fn test_eq() {
        assert_eq!(Quality::Default, Quality::Default);
        assert_eq!(Quality::Low, Quality::Low);
        assert_eq!(Quality::Medium, Quality::Medium);
        assert_eq!(Quality::High, Quality::High);
        assert_ne!(Quality::Low, Quality::High);
        assert_ne!(Quality::Default, Quality::Medium);
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Quality::Default), "Default");
        assert_eq!(format!("{:?}", Quality::Low), "Low");
        assert_eq!(format!("{:?}", Quality::Medium), "Medium");
        assert_eq!(format!("{:?}", Quality::High), "High");
    }
}
