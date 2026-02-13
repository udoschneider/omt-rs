//! Error types for the OMT library.

use std::ffi::NulError;
use std::fmt;

/// Result type alias for OMT operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the OMT library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A null pointer was encountered where a valid pointer was expected.
    #[error("null pointer encountered")]
    NullPointer,

    /// Failed to create a CString due to an interior null byte.
    #[error("string contains null byte: {0}")]
    NulError(#[from] NulError),

    /// Invalid UTF-8 string encountered.
    #[error("invalid UTF-8 string")]
    InvalidUtf8,

    /// Operation timed out.
    #[error("operation timed out")]
    Timeout,

    /// Failed to create sender.
    #[error("failed to create sender")]
    SenderCreateFailed,

    /// Failed to create receiver.
    #[error("failed to create receiver")]
    ReceiverCreateFailed,

    /// Invalid frame type.
    #[error("invalid frame type")]
    InvalidFrameType,

    /// Invalid codec.
    #[error("invalid codec: {0}")]
    InvalidCodec(String),

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
