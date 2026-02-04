//! High-level settings helpers for Open Media Transport (OMT).
//!
//! These helpers wrap global configuration and logging utilities exposed by
//! libomt. Note that other APIs in this crate use `Duration` for timeouts.
//! For protocol context, see:
//! <https://github.com/openmediatransport>
//!
//! Settings are stored in `~/.OMT/settings.xml` on Mac and Linux and
//! `C:\ProgramData\OMT\settings.xml` on Windows by default. To override the
//! default folder used for settings, set the `OMT_STORAGE_PATH` environment
//! variable prior to calling any OMT functions.
use crate::ffi;
use crate::OmtError;
use std::ffi::{CStr, CString};
use std::ptr;

/// Sets the optional log file path for libomt logging.
///
/// Use `None` to disable file logging.
pub fn set_logging_filename(path: Option<&str>) -> Result<(), OmtError> {
    match path {
        Some(p) => {
            let c_path = CString::new(p).map_err(|_| OmtError::InvalidCString)?;
            unsafe { ffi::omt_setloggingfilename(c_path.as_ptr()) };
        }
        None => unsafe { ffi::omt_setloggingfilename(ptr::null()) },
    }
    Ok(())
}

/// Reads a string setting by name.
pub fn settings_get_string(name: &str) -> Option<String> {
    let c_name = CString::new(name).ok()?;
    let mut buf = vec![0u8; ffi::OMT_MAX_STRING_LENGTH];
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
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char) };
    Some(cstr.to_string_lossy().to_string())
}

/// Sets a string setting by name.
pub fn settings_set_string(name: &str, value: &str) -> Result<(), OmtError> {
    let c_name = CString::new(name).map_err(|_| OmtError::InvalidCString)?;
    let c_value = CString::new(value).map_err(|_| OmtError::InvalidCString)?;
    unsafe { ffi::omt_settings_set_string(c_name.as_ptr(), c_value.as_ptr()) };
    Ok(())
}

/// Reads an integer setting by name.
pub fn settings_get_integer(name: &str) -> Option<i32> {
    let c_name = CString::new(name).ok()?;
    let val = unsafe { ffi::omt_settings_get_integer(c_name.as_ptr()) };
    Some(val as i32)
}

/// Sets an integer setting by name.
pub fn settings_set_integer(name: &str, value: i32) -> Result<(), OmtError> {
    let c_name = CString::new(name).map_err(|_| OmtError::InvalidCString)?;
    unsafe { ffi::omt_settings_set_integer(c_name.as_ptr(), value) };
    Ok(())
}

/// Gets the DiscoveryServer setting.
///
/// Returns a URL in the format `omt://hostname:port` to connect to for discovery.
/// If `None` is returned, default DNS-SD discovery behavior is enabled.
pub fn get_discovery_server() -> Option<String> {
    settings_get_string("DiscoveryServer")
}

/// Sets the DiscoveryServer setting.
///
/// # Arguments
/// * `server` - A URL in the format `omt://hostname:port` to connect to for discovery.
///   Pass an empty string to use default DNS-SD discovery.
pub fn set_discovery_server(server: &str) -> Result<(), OmtError> {
    settings_set_string("DiscoveryServer", server)
}

/// Gets the NetworkPortStart setting.
///
/// Returns the first port to create Send instances on.
/// Defaults to 6400 if not set.
pub fn get_network_port_start() -> i32 {
    match settings_get_integer("NetworkPortStart") {
        Some(0) => 6400, // 0 indicates the setting is not set
        Some(port) => port,
        None => 6400,
    }
}

/// Sets the NetworkPortStart setting.
///
/// # Arguments
/// * `port` - The first port to create Send instances on.
pub fn set_network_port_start(port: i32) -> Result<(), OmtError> {
    settings_set_integer("NetworkPortStart", port)
}

/// Gets the NetworkPortEnd setting.
///
/// Returns the last port to create Send instances on.
/// Defaults to 6600 if not set.
pub fn get_network_port_end() -> i32 {
    match settings_get_integer("NetworkPortEnd") {
        Some(0) => 6600, // 0 indicates the setting is not set
        Some(port) => port,
        None => 6600,
    }
}

/// Sets the NetworkPortEnd setting.
///
/// # Arguments
/// * `port` - The last port to create Send instances on.
pub fn set_network_port_end(port: i32) -> Result<(), OmtError> {
    settings_set_integer("NetworkPortEnd", port)
}

/// Gets the network port range as a tuple (start, end).
///
/// Returns the port range to create Send instances on.
/// Defaults to (6400, 6600) if not set.
pub fn get_network_port_range() -> (i32, i32) {
    (get_network_port_start(), get_network_port_end())
}

/// Sets the network port range.
///
/// # Arguments
/// * `start` - The first port to create Send instances on.
/// * `end` - The last port to create Send instances on.
pub fn set_network_port_range(start: i32, end: i32) -> Result<(), OmtError> {
    set_network_port_start(start)?;
    set_network_port_end(end)
}
