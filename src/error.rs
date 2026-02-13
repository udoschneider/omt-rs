//! Error types for the omt-rs library.

use thiserror::Error;

/// Errors that can occur when working with the OMT library.
#[derive(Debug, Error)]
pub enum Error {
    /// The underlying C library returned a null handle where a valid pointer was expected.
    #[error("libomt returned a null handle")]
    NullHandle,

    /// A string parameter contained an interior null byte, which is not allowed in C strings.
    #[error("string contained an interior null byte")]
    InvalidCString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle_display() {
        let error = Error::NullHandle;
        assert_eq!(error.to_string(), "libomt returned a null handle");
    }

    #[test]
    fn test_invalid_cstring_display() {
        let error = Error::InvalidCString;
        assert_eq!(error.to_string(), "string contained an interior null byte");
    }

    #[test]
    fn test_error_debug() {
        let error = Error::NullHandle;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NullHandle"));
    }
}
