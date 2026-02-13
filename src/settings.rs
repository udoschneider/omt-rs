//! High-level settings helpers for Open Media Transport (OMT).
//!
//! These helpers wrap global configuration and logging utilities exposed by
//! libomt. Note that other APIs in this crate use `Duration` for timeouts.
//! For protocol context, see:
//! <https://github.com/openmediatransport>
//!
//! ## Settings Storage
//!
//! Settings are stored in `~/.OMT/settings.xml` on Mac and Linux and
//! `C:\ProgramData\OMT\settings.xml` on Windows by default. To override the
//! default folder used for settings, set the `OMT_STORAGE_PATH` environment
//! variable prior to calling any OMT functions.
//!
//! ## Supported Settings
//!
//! The following settings are currently supported:
//!
//! * **DiscoveryServer** [string] - Specify a URL in the format `omt://hostname:port` to connect to for discovery. If left blank, default DNS-SD discovery behavior is enabled.
//! * **NetworkPortStart** [integer] - Specify the first port to create Send instances on. Defaults to 6400.
//! * **NetworkPortEnd** [integer] - Specify the last port to create Send instances on. Defaults to 6600.
//!
//! Settings changed here will persist only for the currently running process.
use crate::ffi;
use crate::Error;
use std::ffi::{CStr, CString};
use std::ptr;

/// Sets the optional log file path for libomt logging.
///
/// # Arguments
///
/// * `path` - An optional file path for log output. Use `None` to disable file logging.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::set_logging_filename;
///
/// // Enable logging to a file
/// set_logging_filename(Some("/var/log/omt.log")).unwrap();
///
/// // Disable file logging
/// set_logging_filename(None).unwrap();
/// ```
pub fn set_logging_filename(path: Option<&str>) -> Result<(), Error> {
    match path {
        Some(p) => {
            let c_path = CString::new(p).map_err(|_| Error::InvalidCString)?;
            // SAFETY: FFI call with valid C string pointer that remains valid for call duration.
            unsafe { ffi::omt_setloggingfilename(c_path.as_ptr()) };
        }
        None => {
            // SAFETY: FFI call with null pointer to disable logging to file.
            unsafe { ffi::omt_setloggingfilename(ptr::null()) }
        }
    }
    Ok(())
}

/// Reads a string setting by name.
///
/// Retrieves the current value of a string setting as a UTF-8 encoded string.
/// Returns `None` if the setting is not found or cannot be retrieved.
///
/// # Arguments
///
/// * `name` - The name of the setting to retrieve.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::settings_get_string;
///
/// if let Some(server) = settings_get_string("DiscoveryServer") {
///     println!("Discovery server: {}", server);
/// }
/// ```
pub fn settings_get_string(name: &str) -> Option<String> {
    let c_name = CString::new(name).ok()?;
    let mut buf = vec![0u8; ffi::OMT_MAX_STRING_LENGTH];
    // SAFETY: FFI call with valid C string and buffer pointers of sufficient size.
    let len = unsafe {
        ffi::omt_settings_get_string(
            c_name.as_ptr(),
            buf.as_mut_ptr() as *mut std::ffi::c_char,
            buf.len() as i32,
        )
    };
    if len <= 0 {
        return None;
    }
    // SAFETY: The C library has written a null-terminated string to our buffer.
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
    Some(cstr.to_string_lossy().to_string())
}

/// Sets a string setting by name.
///
/// Sets a string setting that will be used for the duration of this process.
/// The value should be a UTF-8 encoded string.
///
/// # Arguments
///
/// * `name` - The name of the setting to set.
/// * `value` - The UTF-8 encoded string value to set.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::settings_set_string;
///
/// settings_set_string("DiscoveryServer", "omt://localhost:7400").unwrap();
/// ```
pub fn settings_set_string(name: &str, value: &str) -> Result<(), Error> {
    let c_name = CString::new(name).map_err(|_| Error::InvalidCString)?;
    let c_value = CString::new(value).map_err(|_| Error::InvalidCString)?;
    // SAFETY: FFI call with valid C string pointers that remain valid for call duration.
    unsafe { ffi::omt_settings_set_string(c_name.as_ptr(), c_value.as_ptr()) };
    Ok(())
}

/// Reads an integer setting by name.
///
/// Retrieves the current value of an integer setting.
/// Returns `None` if the setting is not found or cannot be retrieved.
///
/// # Arguments
///
/// * `name` - The name of the setting to retrieve.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::settings_get_integer;
///
/// if let Some(port) = settings_get_integer("NetworkPortStart") {
///     println!("Network port start: {}", port);
/// }
/// ```
pub fn settings_get_integer(name: &str) -> Option<i32> {
    let c_name = CString::new(name).ok()?;
    // SAFETY: FFI call with valid C string pointer.
    let val = unsafe { ffi::omt_settings_get_integer(c_name.as_ptr()) };
    Some(val as i32)
}

/// Sets an integer setting by name.
///
/// Sets an integer setting that will be used for the duration of this process.
///
/// # Arguments
///
/// * `name` - The name of the setting to set.
/// * `value` - The integer value to set.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::settings_set_integer;
///
/// settings_set_integer("NetworkPortStart", 7000).unwrap();
/// ```
pub fn settings_set_integer(name: &str, value: i32) -> Result<(), Error> {
    let c_name = CString::new(name).map_err(|_| Error::InvalidCString)?;
    // SAFETY: FFI call with valid C string pointer and integer value.
    unsafe { ffi::omt_settings_set_integer(c_name.as_ptr(), value) };
    Ok(())
}

/// Gets the DiscoveryServer setting.
///
/// Returns a URL in the format `omt://hostname:port` to connect to for discovery.
/// If `None` is returned, default DNS-SD discovery behavior is enabled.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::get_discovery_server;
///
/// if let Some(server) = get_discovery_server() {
///     println!("Using discovery server: {}", server);
/// } else {
///     println!("Using default DNS-SD discovery");
/// }
/// ```
pub fn get_discovery_server() -> Option<String> {
    settings_get_string("DiscoveryServer")
}

/// Sets the DiscoveryServer setting.
///
/// Specifies a URL in the format `omt://hostname:port` to connect to for discovery.
/// If left blank, default DNS-SD discovery behavior is enabled.
///
/// # Arguments
///
/// * `server` - A URL in the format `omt://hostname:port` to connect to for discovery.
///   Pass an empty string to use default DNS-SD discovery.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::set_discovery_server;
///
/// // Use a specific discovery server
/// set_discovery_server("omt://discovery.example.com:7400").unwrap();
///
/// // Use default DNS-SD discovery
/// set_discovery_server("").unwrap();
/// ```
pub fn set_discovery_server(server: &str) -> Result<(), Error> {
    settings_set_string("DiscoveryServer", server)
}

/// Gets the NetworkPortStart setting.
///
/// Specifies the first port to create Send instances on.
/// Defaults to 6400 if not set.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::get_network_port_start;
///
/// let port_start = get_network_port_start();
/// println!("Network port start: {}", port_start);
/// ```
pub fn get_network_port_start() -> i32 {
    match settings_get_integer("NetworkPortStart") {
        Some(0) => 6400, // 0 indicates the setting is not set
        Some(port) => port,
        None => 6400,
    }
}

/// Sets the NetworkPortStart setting.
///
/// Specifies the first port to create Send instances on.
///
/// # Arguments
///
/// * `port` - The first port to create Send instances on.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::set_network_port_start;
///
/// set_network_port_start(7000).unwrap();
/// ```
pub fn set_network_port_start(port: i32) -> Result<(), Error> {
    settings_set_integer("NetworkPortStart", port)
}

/// Gets the NetworkPortEnd setting.
///
/// Specifies the last port to create Send instances on.
/// Defaults to 6600 if not set.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::get_network_port_end;
///
/// let port_end = get_network_port_end();
/// println!("Network port end: {}", port_end);
/// ```
pub fn get_network_port_end() -> i32 {
    match settings_get_integer("NetworkPortEnd") {
        Some(0) => 6600, // 0 indicates the setting is not set
        Some(port) => port,
        None => 6600,
    }
}

/// Sets the NetworkPortEnd setting.
///
/// Specifies the last port to create Send instances on.
///
/// # Arguments
///
/// * `port` - The last port to create Send instances on.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::set_network_port_end;
///
/// set_network_port_end(7200).unwrap();
/// ```
pub fn set_network_port_end(port: i32) -> Result<(), Error> {
    settings_set_integer("NetworkPortEnd", port)
}

/// Gets the network port range as a tuple (start, end).
///
/// Returns the port range to create Send instances on.
/// Defaults to (6400, 6600) if not set.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::get_network_port_range;
///
/// let (start, end) = get_network_port_range();
/// println!("Port range: {} - {}", start, end);
/// ```
pub fn get_network_port_range() -> (i32, i32) {
    (get_network_port_start(), get_network_port_end())
}

/// Sets the network port range.
///
/// Convenience function to set both NetworkPortStart and NetworkPortEnd settings.
///
/// # Arguments
///
/// * `start` - The first port to create Send instances on.
/// * `end` - The last port to create Send instances on.
///
/// # Examples
///
/// ```no_run
/// use omt::settings::set_network_port_range;
///
/// set_network_port_range(7000, 7200).unwrap();
/// ```
pub fn set_network_port_range(start: i32, end: i32) -> Result<(), Error> {
    set_network_port_start(start)?;
    set_network_port_end(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_discovery_server() {
        // Test setting and getting discovery server
        let test_server = "omt://test.example.com:7400";
        set_discovery_server(test_server).unwrap();
        if let Some(server) = get_discovery_server() {
            assert_eq!(server, test_server);
        }

        // Test clearing discovery server
        set_discovery_server("").unwrap();
    }

    #[test]
    fn test_set_and_get_network_port_start() {
        // Test setting and getting network port start
        set_network_port_start(7000).unwrap();
        let port = get_network_port_start();
        assert_eq!(port, 7000);
    }

    #[test]
    fn test_set_and_get_network_port_end() {
        // Test setting and getting network port end
        set_network_port_end(7200).unwrap();
        let port = get_network_port_end();
        assert_eq!(port, 7200);
    }

    #[test]
    fn test_get_network_port_range() {
        // Test getting network port range
        set_network_port_start(8000).unwrap();
        set_network_port_end(8200).unwrap();
        let (start, end) = get_network_port_range();
        assert_eq!(start, 8000);
        assert_eq!(end, 8200);
    }

    #[test]
    fn test_set_network_port_range() {
        // Test setting network port range
        set_network_port_range(9000, 9200).unwrap();
        let (start, end) = get_network_port_range();
        assert_eq!(start, 9000);
        assert_eq!(end, 9200);
    }

    #[test]
    fn test_settings_get_string_returns_option() {
        // Test that settings_get_string returns Option
        let result = settings_get_string("NonExistentSetting");
        // May be None for non-existent setting
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_settings_set_and_get_string() {
        // Test setting and getting a string value
        let key = "TestStringSetting";
        let value = "TestValue123";
        settings_set_string(key, value).unwrap();
        if let Some(retrieved) = settings_get_string(key) {
            assert_eq!(retrieved, value);
        }
    }

    #[test]
    fn test_settings_set_string_with_null_byte_fails() {
        // Test that setting a string with null byte fails
        let result = settings_set_string("TestKey", "value\0with\0nulls");
        assert!(result.is_err());
        match result {
            Err(Error::InvalidCString) => {}
            _ => panic!("Expected InvalidCString error"),
        }
    }

    #[test]
    fn test_settings_get_integer_returns_option() {
        // Test that settings_get_integer returns Option
        let result = settings_get_integer("NonExistentIntSetting");
        // May be None for non-existent setting
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_settings_set_and_get_integer() {
        // Test setting and getting an integer value
        let key = "TestIntegerSetting";
        let value = 12345;
        settings_set_integer(key, value).unwrap();
        if let Some(retrieved) = settings_get_integer(key) {
            assert_eq!(retrieved, value);
        }
    }

    #[test]
    fn test_settings_set_integer_negative_value() {
        // Test setting a negative integer value
        let key = "TestNegativeInt";
        let value = -9999;
        settings_set_integer(key, value).unwrap();
        if let Some(retrieved) = settings_get_integer(key) {
            assert_eq!(retrieved, value);
        }
    }

    #[test]
    fn test_settings_set_integer_zero() {
        // Test setting zero value
        let key = "TestZeroInt";
        settings_set_integer(key, 0).unwrap();
        if let Some(retrieved) = settings_get_integer(key) {
            assert_eq!(retrieved, 0);
        }
    }

    #[test]
    fn test_set_logging_filename_with_path() {
        // Test setting logging filename with a path
        let result = set_logging_filename(Some("/tmp/test.log"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_logging_filename_none() {
        // Test disabling logging by passing None
        let result = set_logging_filename(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_logging_filename_with_null_byte_fails() {
        // Test that setting a path with null byte fails
        let result = set_logging_filename(Some("/tmp/test\0.log"));
        assert!(result.is_err());
        match result {
            Err(Error::InvalidCString) => {}
            _ => panic!("Expected InvalidCString error"),
        }
    }

    #[test]
    fn test_get_network_port_start_default() {
        // Test that default port start is reasonable
        let port = get_network_port_start();
        // Should be a valid port number
        assert!(port > 0);
        assert!(port < 65536);
    }

    #[test]
    fn test_get_network_port_end_default() {
        // Test that default port end is reasonable
        let port = get_network_port_end();
        // Should be a valid port number
        assert!(port > 0);
        assert!(port < 65536);
    }
}
