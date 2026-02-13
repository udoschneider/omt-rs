//! Configuration settings for the OMT library.
//!
//! Settings are stored in `~/.OMT/settings.xml` on Mac/Linux and
//! `C:\ProgramData\OMT\settings.xml` on Windows by default.
//!
//! To override the default folder, set the `OMT_STORAGE_PATH` environment
//! variable prior to calling any OMT functions.

use crate::error::{Error, Result};
use crate::MAX_STRING_LENGTH;
use std::ffi::CString;

/// Configuration settings manager.
///
/// Provides access to OMT library settings such as discovery server,
/// network port ranges, and other configuration options.
pub struct Settings;

impl Settings {
    /// Gets a string setting value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// if let Ok(server) = Settings::get_string("DiscoveryServer") {
    ///     println!("Discovery server: {}", server);
    /// }
    /// ```
    pub fn get_string(name: &str) -> Result<String> {
        let c_name = CString::new(name)?;
        let mut buffer = vec![0i8; MAX_STRING_LENGTH];

        let len = unsafe {
            omt_sys::omt_settings_get_string(
                c_name.as_ptr(),
                buffer.as_mut_ptr(),
                MAX_STRING_LENGTH as i32,
            )
        };

        if len <= 0 {
            return Ok(String::new());
        }

        let bytes: Vec<u8> = buffer[..len as usize]
            .iter()
            .map(|&b| b as u8)
            .collect();

        String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
    }

    /// Sets a string setting value.
    ///
    /// The value persists only for the current process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// Settings::set_string("DiscoveryServer", "omt://server:6400")?;
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn set_string(name: &str, value: &str) -> Result<()> {
        let c_name = CString::new(name)?;
        let c_value = CString::new(value)?;

        unsafe {
            omt_sys::omt_settings_set_string(c_name.as_ptr(), c_value.as_ptr());
        }

        Ok(())
    }

    /// Gets an integer setting value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// let port_start = Settings::get_integer("NetworkPortStart");
    /// println!("Port start: {}", port_start);
    /// ```
    pub fn get_integer(name: &str) -> i32 {
        let c_name = CString::new(name).unwrap_or_default();
        unsafe { omt_sys::omt_settings_get_integer(c_name.as_ptr()) }
    }

    /// Sets an integer setting value.
    ///
    /// The value persists only for the current process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// Settings::set_integer("NetworkPortStart", 7000);
    /// ```
    pub fn set_integer(name: &str, value: i32) {
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                omt_sys::omt_settings_set_integer(c_name.as_ptr(), value);
            }
        }
    }

    /// Gets the discovery server URL.
    ///
    /// If blank, default DNS-SD discovery is enabled.
    pub fn discovery_server() -> Result<String> {
        Self::get_string("DiscoveryServer")
    }

    /// Sets the discovery server URL.
    ///
    /// Format: `omt://hostname:port`
    pub fn set_discovery_server(url: &str) -> Result<()> {
        Self::set_string("DiscoveryServer", url)
    }

    /// Gets the network port range start.
    ///
    /// Default: 6400
    pub fn network_port_start() -> i32 {
        Self::get_integer("NetworkPortStart")
    }

    /// Sets the network port range start.
    pub fn set_network_port_start(port: i32) {
        Self::set_integer("NetworkPortStart", port);
    }

    /// Gets the network port range end.
    ///
    /// Default: 6600
    pub fn network_port_end() -> i32 {
        Self::get_integer("NetworkPortEnd")
    }

    /// Sets the network port range end.
    pub fn set_network_port_end(port: i32) {
        Self::set_integer("NetworkPortEnd", port);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_integer() {
        // Test getting default port
        let port = Settings::network_port_start();
        assert!(port > 0);
    }
}
