//! High-level settings helpers for Open Media Transport (OMT).
//!
//! These helpers wrap global configuration and logging utilities exposed by
//! libomt. Note that other APIs in this crate use `Duration` for timeouts.
//! For protocol context, see:
//! <https://github.com/openmediatransport>
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
