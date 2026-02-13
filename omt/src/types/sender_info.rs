//! Sender information type and conversion utilities.

use crate::MAX_STRING_LENGTH;
use crate::error::{Error, Result};
use std::fmt;

/// Information describing the sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderInfo {
    /// Product name.
    pub product_name: String,
    /// Manufacturer name.
    pub manufacturer: String,
    /// Version string.
    pub version: String,
}

impl SenderInfo {
    /// Creates a new `SenderInfo`.
    pub fn new(product_name: String, manufacturer: String, version: String) -> Self {
        Self {
            product_name,
            manufacturer,
            version,
        }
    }

    /// Creates from FFI struct.
    pub(crate) fn from_ffi(ffi: &omt_sys::OMTSenderInfo) -> Result<Self> {
        Ok(Self {
            product_name: Self::c_array_to_string(&ffi.ProductName)?,
            manufacturer: Self::c_array_to_string(&ffi.Manufacturer)?,
            version: Self::c_array_to_string(&ffi.Version)?,
        })
    }

    /// Converts to FFI struct.
    pub(crate) fn to_ffi(&self) -> Result<omt_sys::OMTSenderInfo> {
        let mut ffi = omt_sys::OMTSenderInfo {
            ProductName: [0; MAX_STRING_LENGTH],
            Manufacturer: [0; MAX_STRING_LENGTH],
            Version: [0; MAX_STRING_LENGTH],
            Reserved1: [0; MAX_STRING_LENGTH],
            Reserved2: [0; MAX_STRING_LENGTH],
            Reserved3: [0; MAX_STRING_LENGTH],
        };

        Self::string_to_c_array(&self.product_name, &mut ffi.ProductName)?;
        Self::string_to_c_array(&self.manufacturer, &mut ffi.Manufacturer)?;
        Self::string_to_c_array(&self.version, &mut ffi.Version)?;

        Ok(ffi)
    }

    fn c_array_to_string(arr: &[i8; MAX_STRING_LENGTH]) -> Result<String> {
        // SAFETY: Reinterpreting i8 array as u8 array for UTF-8 validation.
        // This is safe because i8 and u8 have the same size and alignment.
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len()) };

        // Find the null terminator. The C API should always null-terminate strings,
        // but we defensively use the full buffer length if no null is found.
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

        // Validate UTF-8 only up to the null terminator (or end of valid data)
        std::str::from_utf8(&bytes[..null_pos])
            .map(|s| s.to_string())
            .map_err(|_| Error::InvalidUtf8)
    }

    fn string_to_c_array(s: &str, arr: &mut [i8; MAX_STRING_LENGTH]) -> Result<()> {
        let bytes = s.as_bytes();

        // We need space for the string plus a null terminator
        if bytes.len() >= MAX_STRING_LENGTH {
            return Err(Error::BufferTooSmall {
                required: bytes.len() + 1,
                provided: MAX_STRING_LENGTH,
            });
        }

        // Copy string bytes into the array
        for (i, &byte) in bytes.iter().enumerate() {
            arr[i] = byte as i8;
        }

        // Null-terminate the string
        arr[bytes.len()] = 0;

        // Zero out the rest of the buffer for consistency and security
        // (prevents leaking old data)
        for i in (bytes.len() + 1)..MAX_STRING_LENGTH {
            arr[i] = 0;
        }

        Ok(())
    }
}

impl Default for SenderInfo {
    fn default() -> Self {
        Self {
            product_name: String::new(),
            manufacturer: String::new(),
            version: String::new(),
        }
    }
}

impl fmt::Display for SenderInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} by {} (v{})",
            self.product_name, self.manufacturer, self.version
        )
    }
}
