//! OMT sender address type for connecting receivers to senders.
//!
//! See [`Address`] for detailed documentation.

/// An OMT sender address for connecting receivers to senders.
///
/// Represents a network address that identifies an OMT sender. Addresses are typically
/// discovered via DNS-SD (Bonjour/Avahi) or a discovery server, and can be in one of
/// two formats:
///
/// 1. **Full discovery name**: `"HOSTNAME (STREAM_NAME)"` - The format returned by
///    OMT discovery services, where `HOSTNAME` is the network host and `STREAM_NAME`
///    is the human-readable stream identifier.
/// 2. **Direct URL**: `"omt://hostname:port"` - A direct connection URL specifying
///    the protocol, host, and port.
///
/// This newtype wrapper distinguishes sender addresses from other strings in the API
/// and provides type safety when passing addresses to receiver creation methods.
///
/// # Examples
///
/// ```
/// use omt::Address;
///
/// // From a discovery result
/// let address = Address::from("workstation-01 (Live Studio Feed)");
///
/// // From a direct URL
/// let address = Address::from("omt://192.168.1.100:5000");
///
/// // Using the constructor
/// let address = Address::new("omt://localhost:8080");
/// ```
///
/// # See Also
/// - [`crate::Receiver::create`] - Uses `Address` to connect to a sender
/// - [`crate::Sender::get_address`] - Returns the published address of a sender
/// - [`crate::Discovery::get_addresses`] - Discovers available sender addresses
/// - `libomt.h` - C API documentation for address handling
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Address(String);

impl Address {
    /// Creates a new `Address` from any type that can be converted to `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let addr1 = Address::new("omt://192.168.1.100:5000");
    /// let addr2 = Address::new(String::from("workstation (Live Stream)"));
    /// ```
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the address as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("omt://localhost:8080");
    /// assert_eq!(address.as_str(), "omt://localhost:8080");
    /// ```
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the `Address` and returns the underlying `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("workstation (Stream)");
    /// let string: String = address.into_inner();
    /// assert_eq!(string, "workstation (Stream)");
    /// ```
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for Address {
    /// Converts a `String` into an `Address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let string = String::from("omt://example.com:9000");
    /// let address: Address = Address::from(string);
    /// ```
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Address {
    /// Converts a string slice into an `Address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address: Address = "server-01 (Production Feed)".into();
    /// ```
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl std::fmt::Display for Address {
    /// Formats the address for display purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("server-01 (Live Feed)");
    /// assert_eq!(format!("{}", address), "server-01 (Live Feed)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Address {
    /// Returns a reference to the underlying string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("omt://localhost:8080");
    /// let str_ref: &str = address.as_ref();
    /// assert_eq!(str_ref, "omt://localhost:8080");
    /// ```
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_str() {
        let address = Address::new("omt://192.168.1.100:5000");
        assert_eq!(address.as_str(), "omt://192.168.1.100:5000");
    }

    #[test]
    fn test_new_from_string() {
        let s = String::from("workstation (Live Stream)");
        let address = Address::new(s);
        assert_eq!(address.as_str(), "workstation (Live Stream)");
    }

    #[test]
    fn test_as_str() {
        let address = Address::new("omt://localhost:8080");
        assert_eq!(address.as_str(), "omt://localhost:8080");
    }

    #[test]
    fn test_into_inner() {
        let address = Address::new("workstation (Stream)");
        let string: String = address.into_inner();
        assert_eq!(string, "workstation (Stream)");
    }

    #[test]
    fn test_from_string() {
        let string = String::from("omt://example.com:9000");
        let address = Address::from(string);
        assert_eq!(address.as_str(), "omt://example.com:9000");
    }

    #[test]
    fn test_from_str_ref() {
        let address: Address = "server-01 (Production Feed)".into();
        assert_eq!(address.as_str(), "server-01 (Production Feed)");
    }

    #[test]
    fn test_display() {
        let address = Address::new("server-01 (Live Feed)");
        assert_eq!(format!("{}", address), "server-01 (Live Feed)");
    }

    #[test]
    fn test_as_ref() {
        let address = Address::new("omt://localhost:8080");
        let str_ref: &str = address.as_ref();
        assert_eq!(str_ref, "omt://localhost:8080");
    }

    #[test]
    fn test_clone() {
        let address1 = Address::new("test address");
        let address2 = address1.clone();
        assert_eq!(address1, address2);
    }

    #[test]
    fn test_eq() {
        let address1 = Address::new("omt://test:5000");
        let address2 = Address::new("omt://test:5000");
        let address3 = Address::new("omt://other:5000");
        assert_eq!(address1, address2);
        assert_ne!(address1, address3);
    }

    #[test]
    fn test_debug() {
        let address = Address::new("test");
        let debug_str = format!("{:?}", address);
        assert!(debug_str.contains("Address"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_ord() {
        let addr1 = Address::new("aaa");
        let addr2 = Address::new("bbb");
        let addr3 = Address::new("ccc");
        assert!(addr1 < addr2);
        assert!(addr2 < addr3);
        assert!(addr1 < addr3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let addr1 = Address::new("test1");
        let addr2 = Address::new("test2");
        let addr3 = Address::new("test1");

        set.insert(addr1.clone());
        set.insert(addr2);
        set.insert(addr3);

        assert_eq!(set.len(), 2); // addr1 and addr3 are equal
    }

    #[test]
    fn test_empty_address() {
        let address = Address::new("");
        assert_eq!(address.as_str(), "");
    }

    #[test]
    fn test_unicode_address() {
        let address = Address::new("服务器 (直播)");
        assert_eq!(address.as_str(), "服务器 (直播)");
    }
}
