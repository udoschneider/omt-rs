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
