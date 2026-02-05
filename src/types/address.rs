//! OMT sender address type for connecting receivers to senders.
//!
//! Represents a network address that identifies an OMT sender. Addresses are typically
//! discovered via DNS-SD (Bonjour/Avahi) or a discovery server, and can be in one of
//! two formats:
//!
//! 1. **Full discovery name**: `"HOSTNAME (STREAM_NAME)"` - The format returned by
//!    OMT discovery services, where `HOSTNAME` is the network host and `STREAM_NAME`
//!    is the human-readable stream identifier.
//! 2. **Direct URL**: `"omt://hostname:port"` - A direct connection URL specifying
//!    the protocol, host, and port.
//!
//! This newtype wrapper distinguishes sender addresses from other strings in the API
//! and provides type safety when passing addresses to receiver creation methods.
//!
//! # Examples
//!
//! ```
//! use omt::Address;
//!
//! // From a discovery result
//! let address = Address::from("workstation-01 (Live Studio Feed)");
//!
//! // From a direct URL
//! let address = Address::from("omt://192.168.1.100:5000");
//!
//! // Using the constructor
//! let address = Address::new("omt://localhost:8080");
//! ```
//!
//! # See Also
//! - [`crate::Receiver::create`] - Uses `Address` to connect to a sender
//! - [`crate::Sender::get_address`] - Returns the published address of a sender
//! - [`crate::Discovery::get_addresses`] - Discovers available sender addresses
//! - `libomt.h` - C API documentation for address handling

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
    /// This is the primary constructor for creating addresses from string literals,
    /// `String` values, or other string-like types.
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
    /// This provides immutable access to the underlying address string without
    /// consuming the `Address`.
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
    /// This is useful when you need to take ownership of the address string,
    /// such as when storing it in a data structure that requires `String`.
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
    /// This allows for seamless conversion from owned strings to addresses,
    /// enabling ergonomic API usage.
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
    /// This allows string literals and borrowed strings to be used as addresses
    /// without explicit allocation when not needed.
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
    /// This implementation allows `Address` values to be used with formatting macros
    /// like `format!`, `println!`, and `write!`. The output is the raw address string
    /// without any additional formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("server-01 (Live Feed)");
    /// assert_eq!(format!("{}", address), "server-01 (Live Feed)");
    /// println!("Connected to: {}", address);
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Address {
    /// Returns a reference to the underlying string slice.
    ///
    /// This allows `Address` to be used in contexts that require a `&str`,
    /// such as string comparison methods or APIs that accept string slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Address;
    ///
    /// let address = Address::new("omt://localhost:8080");
    /// let str_ref: &str = address.as_ref();
    /// assert_eq!(str_ref, "omt://localhost:8080");
    ///
    /// // Can be used with string methods
    /// assert!(address.as_ref().contains("localhost"));
    /// ```
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
