//! Helpers for copying strings returned by the libomt C API into owned strings.

use crate::error::{Error, Result};
use std::os::raw::c_char;

/// Copies a string returned by the C library out of `buffer` into an owned
/// [`String`].
///
/// The libomt string getters report the length of the UTF-8 value *including*
/// its null terminator (`libomt.h`: "Returns the length in bytes of the UTF-8
/// encoded value including null terminator"), so the payload is trimmed at the
/// first NUL byte before conversion.
///
/// Returns an empty string when the call reports no data (`len <= 0`, e.g. an
/// unset setting). A contract-violating length larger than `buffer` is clamped
/// rather than panicking (the crate's no-panics rule).
///
/// # Errors
///
/// Returns [`Error::InvalidUtf8`] if the buffer contents are not valid UTF-8.
pub(crate) fn from_buffer(buffer: &[c_char], len: i32) -> Result<String> {
    if len <= 0 {
        return Ok(String::new());
    }

    // Clamp to the buffer we handed the C call: a contract-violating length
    // larger than `buffer.len()` must not panic the slice (no-panics rule).
    let len = (len as usize).min(buffer.len());
    let bytes: Vec<u8> = buffer[..len].iter().map(|&b| b as u8).collect();

    // Trim at the first NUL: the reported length includes the terminator, and
    // anything after it is padding, not part of the value.
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

    String::from_utf8(bytes[..end].to_vec()).map_err(|_| Error::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_included_null_terminator() {
        let buffer: Vec<c_char> = b"omt://host\0padding"
            .iter()
            .map(|&b| b as c_char)
            .collect();
        assert_eq!(from_buffer(&buffer, 11).unwrap(), "omt://host");
    }

    #[test]
    fn empty_when_no_data_reported() {
        let buffer: Vec<c_char> = vec![0; 8];
        assert_eq!(from_buffer(&buffer, 0).unwrap(), "");
        assert_eq!(from_buffer(&buffer, -1).unwrap(), "");
    }

    #[test]
    fn clamps_length_exceeding_buffer() {
        let buffer: Vec<c_char> = vec![b'a' as c_char; 4];
        assert_eq!(from_buffer(&buffer, 1000).unwrap(), "aaaa");
    }

    #[test]
    fn rejects_invalid_utf8() {
        let bytes: [u8; 3] = [0xFF, 0xFE, 0x00];
        let buffer: Vec<c_char> = bytes.iter().map(|&b| b as c_char).collect();
        assert!(matches!(
            from_buffer(&buffer, bytes.len() as i32),
            Err(Error::InvalidUtf8)
        ));
    }
}
