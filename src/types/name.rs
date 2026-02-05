//! Name type for OMT senders.
//!
//! A `Name` represents the name of an OMT sender that can be published,
//! discovered, and connected to by receivers.
//!
//! In the OMT protocol, a sender name is used for:
//! - DNS-SD (Bonjour/mDNS) advertisement and discovery
//! - Identifying senders in discovery results
//! - Creating sender instances via [`omt_send_create`](https://github.com/openmediatransport/libomt/blob/main/docs/libomt.h#L449)
//!
//! The sender name should not include hostname information. When discovered
//! via [`omt_discovery_getaddresses`](https://github.com/openmediatransport/libomt/blob/main/docs/libomt.h#L359),
//! senders are returned in the format "HOSTNAME (NAME)" where NAME is the
//! sender name specified during sender creation.
//!
//! # Examples
//!
//! ```rust
//! use omt::types::Name;
//!
//! // Create a name from a string literal
//! let name = Name::from("MyCamera1");
//!
//! // Create a name with a dynamic value
//! let name = Name::new(format!("test-sender-{}", std::process::id()));
//!
//! // Use the name to create a sender
//! // let sender = Sender::create(&name, Quality::Default)?;
//! ```

use std::fmt;

/// A sender name that can be published and discovered on the OMT network.
///
/// The sender name identifies an OMT sender instance and is used throughout
/// the OMT ecosystem:
///
/// - **Discovery**: Senders are advertised via DNS-SD (Bonjour/mDNS) and can
///   be discovered using [`omt_discovery_getaddresses`](https://github.com/openmediatransport/libomt/blob/main/docs/libomt.h#L359)
/// - **Sender Creation**: The name is passed to [`omt_send_create`](https://github.com/openmediatransport/libomt/blob/main/docs/libomt.h#L449)
///   to create a new sender instance
/// - **Address Format**: When discovered, senders appear as "HOSTNAME (NAME)"
///   where NAME is this sender name
///
/// The sender name should be unique within the local network segment to avoid
/// conflicts. It should not include hostname or port information.
///
/// # OMT Protocol Notes
///
/// According to `libomt.h`:
/// - Sender names are UTF-8 encoded strings
/// - Maximum length is defined by `OMT_MAX_STRING_LENGTH` (typically 256 bytes)
/// - Names are used for DNS-SD service type `_omt._tcp`
/// - Receivers connect to senders using the full address format returned by discovery
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name(String);

impl Name {
    /// Creates a new `Name` from any type that can be converted to `String`.
    ///
    /// The sender name will be used for DNS-SD advertisement and must be
    /// a valid UTF-8 string without null bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::new("Camera-1");
    /// let name = Name::new(format!("sender-{}", 123));
    /// ```
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the sender name as a string slice.
    ///
    /// This is the name that will be passed to [`omt_send_create`](https://github.com/openmediatransport/libomt/blob/main/docs/libomt.h#L449)
    /// and advertised via DNS-SD.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::from("MySender");
    /// assert_eq!(name.as_str(), "MySender");
    /// ```
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the `Name`, returning the underlying `String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::from("TestName");
    /// let string: String = name.into_inner();
    /// assert_eq!(string, "TestName");
    /// ```
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for Name {
    /// Creates a `Name` from a `String`.
    ///
    /// This conversion is used when you already have a `String` and want to
    /// use it as a sender name. The string must be valid UTF-8 and should not
    /// contain null bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name_string = String::from("Camera-1");
    /// let name = Name::from(name_string);
    /// ```
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Name {
    /// Creates a `Name` from a string slice.
    ///
    /// This is a convenient conversion for string literals and other `&str`
    /// references. The string will be copied into a new `String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::from("LiveStream");
    /// assert_eq!(name.as_str(), "LiveStream");
    /// ```
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for Name {
    /// Formats the sender name for display.
    ///
    /// This allows `Name` values to be used with formatting macros like
    /// `format!`, `println!`, and `write!`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::from("MyName");
    /// println!("Name: {}", name); // Prints: Name: MyName
    /// assert_eq!(format!("{}", name), "MyName");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Name {
    /// Returns a reference to the underlying string slice.
    ///
    /// This allows `Name` to be used anywhere a `&str` is expected,
    /// such as in string comparison functions or when passing to C FFI
    /// functions that expect null-terminated strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use omt::types::Name;
    ///
    /// let name = Name::from("Test");
    /// let slice: &str = name.as_ref();
    /// assert_eq!(slice, "Test");
    /// ```
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
