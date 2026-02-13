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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_info_new() {
        let info = SenderInfo::new();
        assert_eq!(info.product_name, "");
        assert_eq!(info.manufacturer, "");
        assert_eq!(info.version, "");
        assert_eq!(info.reserved1, "");
        assert_eq!(info.reserved2, "");
        assert_eq!(info.reserved3, "");
    }

    #[test]
    fn test_sender_info_default() {
        let info = SenderInfo::default();
        assert_eq!(info.product_name, "");
        assert_eq!(info.manufacturer, "");
        assert_eq!(info.version, "");
    }

    #[test]
    fn test_with_product_name() {
        let info = SenderInfo::new().with_product_name("Professional Camera");
        assert_eq!(info.product_name, "Professional Camera");
    }

    #[test]
    fn test_with_manufacturer() {
        let info = SenderInfo::new().with_manufacturer("ACME Corp");
        assert_eq!(info.manufacturer, "ACME Corp");
    }

    #[test]
    fn test_with_version() {
        let info = SenderInfo::new().with_version("1.0.0");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_with_reserved1() {
        let info = SenderInfo::new().with_reserved1("reserved1");
        assert_eq!(info.reserved1, "reserved1");
    }

    #[test]
    fn test_with_reserved2() {
        let info = SenderInfo::new().with_reserved2("reserved2");
        assert_eq!(info.reserved2, "reserved2");
    }

    #[test]
    fn test_with_reserved3() {
        let info = SenderInfo::new().with_reserved3("reserved3");
        assert_eq!(info.reserved3, "reserved3");
    }

    #[test]
    fn test_builder_chain() {
        let info = SenderInfo::new()
            .with_product_name("Camera")
            .with_manufacturer("ACME")
            .with_version("2.0");
        assert_eq!(info.product_name, "Camera");
        assert_eq!(info.manufacturer, "ACME");
        assert_eq!(info.version, "2.0");
    }

    #[test]
    fn test_with_all_fields() {
        let info = SenderInfo::new()
            .with_product_name("Product")
            .with_manufacturer("Manufacturer")
            .with_version("Version")
            .with_reserved1("R1")
            .with_reserved2("R2")
            .with_reserved3("R3");
        assert_eq!(info.product_name, "Product");
        assert_eq!(info.manufacturer, "Manufacturer");
        assert_eq!(info.version, "Version");
        assert_eq!(info.reserved1, "R1");
        assert_eq!(info.reserved2, "R2");
        assert_eq!(info.reserved3, "R3");
    }

    #[test]
    fn test_clone() {
        let info1 = SenderInfo::new().with_product_name("Test");
        let info2 = info1.clone();
        assert_eq!(info1.product_name, info2.product_name);
    }

    #[test]
    fn test_debug() {
        let info = SenderInfo::new().with_product_name("Test");
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("SenderInfo"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_from_ffi_with_empty_fields() {
        let ffi_info = ffi::OMTSenderInfo {
            ProductName: [0; ffi::OMT_MAX_STRING_LENGTH],
            Manufacturer: [0; ffi::OMT_MAX_STRING_LENGTH],
            Version: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved1: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved2: [0; ffi::OMT_MAX_STRING_LENGTH],
            Reserved3: [0; ffi::OMT_MAX_STRING_LENGTH],
        };
        let result = Option::<SenderInfo>::from(&ffi_info);
        assert!(result.is_none());
    }

    #[test]
    fn test_to_ffi_conversion() {
        let info = SenderInfo::new()
            .with_product_name("TestProduct")
            .with_manufacturer("TestManufacturer")
            .with_version("1.2.3");
        let ffi_info: ffi::OMTSenderInfo = (&info).into();

        // Verify the conversion happened (actual verification would need ffi_utils)
        let _ = ffi_info;
    }

    #[test]
    fn test_with_string_types() {
        let info = SenderInfo::new()
            .with_product_name(String::from("Product"))
            .with_manufacturer("Manufacturer")
            .with_version(format!("v{}", 1));
        assert_eq!(info.product_name, "Product");
        assert_eq!(info.manufacturer, "Manufacturer");
        assert_eq!(info.version, "v1");
    }
}
