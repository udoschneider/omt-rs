//! Sender information metadata for Open Media Transport (OMT).
//!
//! Provides optional metadata describing the sender device/software, exchanged between
//! senders and receivers.

use crate::ffi;
use crate::ffi_utils::{c_char_array_to_string, write_c_char_array};

#[derive(Clone, Debug, Default)]
/// Optional metadata describing the sender device/software.
///
/// Retrieved via [`crate::receiver::Receiver::get_sender_info`]. Valid only when connected.
/// Returns `None` if disconnected or no sender information was provided by the sender.
///
/// Each string field has a maximum length of 1024 bytes.
pub struct SenderInfo {
    /// Product name of the sender device/software
    pub product_name: String,
    /// Manufacturer of the sender device/software
    pub manufacturer: String,
    /// Version string of the sender software
    pub version: String,
    /// Reserved for future use
    pub reserved1: String,
    /// Reserved for future use
    pub reserved2: String,
    /// Reserved for future use
    pub reserved3: String,
}

impl SenderInfo {
    /// Creates a new `SenderInfo` with all fields empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::SenderInfo;
    ///
    /// let info = SenderInfo::new()
    ///     .with_product_name("Professional Camera")
    ///     .with_manufacturer("ACME Corp")
    ///     .with_version("1.0.0");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the product name.
    pub fn with_product_name(mut self, product_name: impl Into<String>) -> Self {
        self.product_name = product_name.into();
        self
    }

    /// Sets the manufacturer.
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = manufacturer.into();
        self
    }

    /// Sets the version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets reserved field 1.
    pub fn with_reserved1(mut self, reserved1: impl Into<String>) -> Self {
        self.reserved1 = reserved1.into();
        self
    }

    /// Sets reserved field 2.
    pub fn with_reserved2(mut self, reserved2: impl Into<String>) -> Self {
        self.reserved2 = reserved2.into();
        self
    }

    /// Sets reserved field 3.
    pub fn with_reserved3(mut self, reserved3: impl Into<String>) -> Self {
        self.reserved3 = reserved3.into();
        self
    }
}

impl From<&ffi::OMTSenderInfo> for Option<SenderInfo> {
    fn from(info: &ffi::OMTSenderInfo) -> Self {
        let product_name = c_char_array_to_string(&info.ProductName[..]);
        let manufacturer = c_char_array_to_string(&info.Manufacturer[..]);
        let version = c_char_array_to_string(&info.Version[..]);
        let reserved1 = c_char_array_to_string(&info.Reserved1[..]);
        let reserved2 = c_char_array_to_string(&info.Reserved2[..]);
        let reserved3 = c_char_array_to_string(&info.Reserved3[..]);

        let has_any = !product_name.is_empty()
            || !manufacturer.is_empty()
            || !version.is_empty()
            || !reserved1.is_empty()
            || !reserved2.is_empty()
            || !reserved3.is_empty();

        if has_any {
            Some(SenderInfo {
                product_name,
                manufacturer,
                version,
                reserved1,
                reserved2,
                reserved3,
            })
        } else {
            None
        }
    }
}

impl From<&SenderInfo> for ffi::OMTSenderInfo {
    fn from(info: &SenderInfo) -> Self {
        use std::os::raw::c_char;
        let mut raw = ffi::OMTSenderInfo {
            ProductName: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Manufacturer: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Version: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved1: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved2: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
            Reserved3: [0 as c_char; ffi::OMT_MAX_STRING_LENGTH],
        };

        write_c_char_array(&mut raw.ProductName[..], &info.product_name);
        write_c_char_array(&mut raw.Manufacturer[..], &info.manufacturer);
        write_c_char_array(&mut raw.Version[..], &info.version);
        write_c_char_array(&mut raw.Reserved1[..], &info.reserved1);
        write_c_char_array(&mut raw.Reserved2[..], &info.reserved2);
        write_c_char_array(&mut raw.Reserved3[..], &info.reserved3);

        raw
    }
}
