//! Tally state for Open Media Transport (OMT).
//!
//! Tally information is exchanged bidirectionally between receivers and senders:
//! - Receivers can inform senders of their tally state
//! - Receivers can query the aggregated tally state across all connections to a sender

use crate::ffi;

#[derive(Clone, Debug, Default)]
/// On-air tally state where `false` = off, `true` = on.
///
/// Tally information is exchanged bidirectionally between receivers and senders:
/// - Use [`crate::receiver::Receiver::set_tally`] to inform the sender of this receiver's tally state
/// - Use [`crate::receiver::Receiver::get_tally`] to receive the aggregated tally state across all
///   connections to a sender (not just this receiver)
pub struct Tally {
    /// Preview tally state
    pub preview: bool,
    /// Program tally state
    pub program: bool,
}

impl From<&Tally> for ffi::OMTTally {
    fn from(tally: &Tally) -> Self {
        ffi::OMTTally {
            preview: if tally.preview { 1 } else { 0 },
            program: if tally.program { 1 } else { 0 },
        }
    }
}

impl From<&ffi::OMTTally> for Tally {
    fn from(tally: &ffi::OMTTally) -> Self {
        Tally {
            preview: tally.preview != 0,
            program: tally.program != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tally_default() {
        let tally = Tally::default();
        assert_eq!(tally.preview, false);
        assert_eq!(tally.program, false);
    }

    #[test]
    fn test_tally_from_ffi_both_false() {
        let ffi_tally = ffi::OMTTally {
            preview: 0,
            program: 0,
        };
        let tally = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, false);
        assert_eq!(tally.program, false);
    }

    #[test]
    fn test_tally_from_ffi_both_true() {
        let ffi_tally = ffi::OMTTally {
            preview: 1,
            program: 1,
        };
        let tally = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, true);
        assert_eq!(tally.program, true);
    }

    #[test]
    fn test_tally_from_ffi_preview_only() {
        let ffi_tally = ffi::OMTTally {
            preview: 1,
            program: 0,
        };
        let tally = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, true);
        assert_eq!(tally.program, false);
    }

    #[test]
    fn test_tally_from_ffi_program_only() {
        let ffi_tally = ffi::OMTTally {
            preview: 0,
            program: 1,
        };
        let tally = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, false);
        assert_eq!(tally.program, true);
    }

    #[test]
    fn test_tally_from_ffi_non_zero_values() {
        // Test that any non-zero value is treated as true
        let ffi_tally = ffi::OMTTally {
            preview: 42,
            program: 255,
        };
        let tally = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, true);
        assert_eq!(tally.program, true);
    }

    #[test]
    fn test_tally_to_ffi_both_false() {
        let tally = Tally {
            preview: false,
            program: false,
        };
        let ffi_tally = ffi::OMTTally::from(&tally);
        assert_eq!(ffi_tally.preview, 0);
        assert_eq!(ffi_tally.program, 0);
    }

    #[test]
    fn test_tally_to_ffi_both_true() {
        let tally = Tally {
            preview: true,
            program: true,
        };
        let ffi_tally = ffi::OMTTally::from(&tally);
        assert_eq!(ffi_tally.preview, 1);
        assert_eq!(ffi_tally.program, 1);
    }

    #[test]
    fn test_tally_to_ffi_preview_only() {
        let tally = Tally {
            preview: true,
            program: false,
        };
        let ffi_tally = ffi::OMTTally::from(&tally);
        assert_eq!(ffi_tally.preview, 1);
        assert_eq!(ffi_tally.program, 0);
    }

    #[test]
    fn test_tally_to_ffi_program_only() {
        let tally = Tally {
            preview: false,
            program: true,
        };
        let ffi_tally = ffi::OMTTally::from(&tally);
        assert_eq!(ffi_tally.preview, 0);
        assert_eq!(ffi_tally.program, 1);
    }

    #[test]
    fn test_clone() {
        let tally1 = Tally {
            preview: true,
            program: false,
        };
        let tally2 = tally1.clone();
        assert_eq!(tally1.preview, tally2.preview);
        assert_eq!(tally1.program, tally2.program);
    }

    #[test]
    fn test_debug() {
        let tally = Tally {
            preview: true,
            program: false,
        };
        let debug_str = format!("{:?}", tally);
        assert!(debug_str.contains("Tally"));
        assert!(debug_str.contains("preview"));
        assert!(debug_str.contains("program"));
    }

    #[test]
    fn test_roundtrip_conversion() {
        let tally = Tally {
            preview: true,
            program: false,
        };
        let ffi_tally = ffi::OMTTally::from(&tally);
        let tally2 = Tally::from(&ffi_tally);
        assert_eq!(tally.preview, tally2.preview);
        assert_eq!(tally.program, tally2.program);
    }
}
