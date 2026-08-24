//! Error types for the OMT library.

use std::ffi::NulError;
use std::fmt;

/// Result type alias for OMT operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the OMT library.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in a future
/// release without a major version bump, so downstream `match`es need a
/// wildcard arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Failed to create a CString due to an interior null byte.
    #[error("string contains null byte: {0}")]
    NulError(#[from] NulError),

    /// Invalid UTF-8 string encountered.
    #[error("invalid UTF-8 string")]
    InvalidUtf8,

    /// Failed to create sender.
    #[error("failed to create sender")]
    SenderCreateFailed,

    /// Failed to create receiver.
    #[error("failed to create receiver")]
    ReceiverCreateFailed,

    /// Buffer too small for operation.
    #[error("buffer too small: required {required}, provided {provided}")]
    BufferTooSmall {
        /// Required buffer size.
        required: usize,
        /// Provided buffer size.
        provided: usize,
    },

    /// Invalid parameter provided.
    #[error("invalid parameter '{parameter}': {reason}")]
    InvalidParameter {
        /// Parameter name.
        parameter: String,
        /// Reason for invalidity.
        reason: String,
    },

    /// Network discovery returned a result the C library cannot have meant.
    ///
    /// Reported when `omt_discovery_getaddresses` claims a source count far
    /// beyond anything a real network produces, which indicates the returned
    /// array is corrupt rather than merely empty. Indexing into it on that
    /// promise would be a wild read, so the whole result is rejected.
    #[error("discovery reported an implausible source count of {count} (maximum {max})")]
    DiscoveryCountImplausible {
        /// The count the C library reported.
        count: i32,
        /// The largest count this crate is willing to trust.
        max: i32,
    },

    /// Generic error with message.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Creates a new error with a custom message.
    pub fn other(msg: impl fmt::Display) -> Self {
        Self::Other(msg.to_string())
    }
}
