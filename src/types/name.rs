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
use uuid::Uuid;

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

    pub fn new_unique(value: impl Into<String>) -> Self {
        Self(format!("{}-{}", value.into(), Uuid::now_v7()))
    }

    pub fn make_unique(&mut self) {
        self.0 = format!("{}-{}", self.0, Uuid::now_v7());
    }

    /// Consumes the `Name`, returning the underlying `String`.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
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
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_str() {
        let name = Name::new_unique("MyCamera1");
        assert!(name.as_str().starts_with("MyCamera1-"));
    }

    #[test]
    fn test_new_from_string() {
        let s = String::from("sender-123");
        let name = Name::new_unique(s);
        assert!(name.as_str().starts_with("sender-123-"));
    }

    #[test]
    fn test_as_str() {
        let name = Name::from("MySender");
        assert_eq!(name.as_str(), "MySender");
    }

    #[test]
    fn test_into_inner() {
        let name = Name::new_unique("TestName");
        let string = name.into_inner();
        assert!(string.starts_with("TestName-"));
    }

    #[test]
    fn test_from_string() {
        let string = String::from("StreamName");
        let name = Name::from(string);
        assert_eq!(name.as_str(), "StreamName");
    }

    #[test]
    fn test_from_str_ref() {
        let name: Name = "Camera-1".into();
        assert_eq!(name.as_str(), "Camera-1");
    }

    #[test]
    fn test_display() {
        let name = Name::new_unique("LiveFeed");
        let display = format!("{}", name);
        assert!(display.starts_with("LiveFeed-"));
    }

    #[test]
    fn test_as_ref() {
        let name = Name::new_unique("TestStream");
        let str_ref: &str = name.as_ref();
        assert!(str_ref.starts_with("TestStream-"));
    }

    #[test]
    fn test_clone() {
        let name1 = Name::new_unique("test name");
        let name2 = name1.clone();
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_eq() {
        let name1 = Name::new_unique("sender1");
        let name2 = name1.clone();
        let name3 = Name::new_unique("sender2");
        assert_eq!(name1, name2);
        assert_ne!(name1, name3);
    }

    #[test]
    fn test_debug() {
        let name = Name::new_unique("test");
        let debug_str = format!("{:?}", name);
        assert!(debug_str.contains("Name"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_ord() {
        let name1 = Name::new_unique("aaa");
        let name2 = Name::new_unique("bbb");
        let name3 = Name::new_unique("ccc");
        assert!(name1 < name2);
        assert!(name2 < name3);
        assert!(name1 < name3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let name1 = Name::new_unique("test1");
        let name2 = Name::new_unique("test2");
        let name3 = name1.clone();

        set.insert(name1.clone());
        set.insert(name2);
        set.insert(name3);

        assert_eq!(set.len(), 2); // name1 and name3 are equal
    }

    #[test]
    fn test_empty_name() {
        let name = Name::new_unique("");
        assert!(name.as_str().starts_with("-"));
    }

    #[test]
    fn test_unicode_name() {
        let name = Name::new_unique("摄像机一号");
        assert!(name.as_str().starts_with("摄像机一号-"));
    }

    #[test]
    fn test_special_characters() {
        let name = Name::new_unique("Camera-1_Test");
        assert!(name.as_str().starts_with("Camera-1_Test-"));
    }
}
