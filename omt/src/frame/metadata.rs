//! Metadata-specific methods for MediaFrame.

use crate::error::{Error, Result};
use crate::frame::MediaFrame;

impl MediaFrame {
    /// Returns the metadata as a UTF-8 string.
    ///
    /// This method is only meaningful for metadata frames.
    pub fn as_utf8(&self) -> Result<&str> {
        let data = self.data();
        // Remove null terminator if present
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        std::str::from_utf8(&data[..end]).map_err(|_| Error::InvalidUtf8)
    }
}
