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
