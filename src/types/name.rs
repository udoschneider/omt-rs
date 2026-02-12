//! Name type for OMT senders.
//!
//! A `Name` represents the name of an OMT sender that can be published,
//! discovered, and connected to by receivers via DNS-SD (Bonjour/mDNS).
//!
//! Sender names are used for discovery and identification. When discovered,
//! senders appear as "HOSTNAME (NAME)" where NAME is the sender name.
//! The name should not include hostname or port information.
//!
//! # Examples
//!
//! ```rust
//! use omt::types::Name;
//!
//! // Create from string literal
//! let name = Name::from("MyCamera1");
//!
//! // Create with dynamic value
//! let name = Name::new(format!("sender-{}", std::process::id()));
//! ```

use std::fmt;

/// A sender name for OMT network discovery and identification.
///
/// Names are UTF-8 encoded strings used for DNS-SD advertisement (service type `_omt._tcp`).
/// Maximum length is `OMT_MAX_STRING_LENGTH` (typically 256 bytes).
///
/// The sender name should not contain hostname or port information.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name(String);

impl Name {
    /// Creates a new `Name` from any type that converts to `String`.
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
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
