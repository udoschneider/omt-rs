//! Configuration settings for the OMT library.
//!
//! Settings are stored in `~/.OMT/settings.xml` on Mac/Linux and
//! `C:\ProgramData\OMT\settings.xml` on Windows by default.
//!
//! To override the default folder, set the `OMT_STORAGE_PATH` environment
//! variable prior to calling any OMT functions.

use crate::MAX_STRING_LENGTH;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::RwLock;

/// The Rust closure invoked for every log line, dispatched by [`logging_trampoline`].
type LogCallback = Box<dyn Fn(&str) + Send + Sync + 'static>;

/// Storage for the currently registered logging closure.
///
/// The C API takes a bare function pointer with no user-data argument, so the
/// closure has to live in a global that the `extern "C"` trampoline can reach.
static LOG_CALLBACK: RwLock<Option<LogCallback>> = RwLock::new(None);

/// `extern "C"` shim handed to the OMT library. It forwards each log line to the
/// closure registered via [`Settings::set_logging_callback`].
extern "C" fn logging_trampoline(message: *const c_char) {
    if message.is_null() {
        return;
    }

    // SAFETY: The OMT library passes a valid, null-terminated C string that is
    // only borrowed for the duration of this call.
    let text = unsafe { CStr::from_ptr(message) }.to_string_lossy();

    // A read lock is enough; log lines may arrive concurrently from many threads.
    if let Ok(guard) = LOG_CALLBACK.read()
        && let Some(callback) = guard.as_ref()
    {
        // A panic must never unwind across the FFI boundary (undefined behavior).
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(&text)));
    }
}

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
        // `c_char` is `i8` on x86_64 but `u8` on Linux aarch64/arm; use it so the
        // buffer element type matches the FFI `*mut c_char` parameter portably.
        let mut buffer = vec![0 as c_char; MAX_STRING_LENGTH];

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

        // Clamp to the buffer we handed the C call: a contract-violating length
        // larger than `MAX_STRING_LENGTH` must not panic the slice (no-panics rule).
        let len = (len as usize).min(buffer.len());
        let bytes: Vec<u8> = buffer[..len].iter().map(|&b| b as u8).collect();

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
        // A `name` with an interior NUL cannot be represented as a C string.
        // Return 0 rather than querying an empty (wrong) key, which is what
        // `CString::new(name).unwrap_or_default()` would silently have done.
        match CString::new(name) {
            Ok(c_name) => unsafe { omt_sys::omt_settings_get_integer(c_name.as_ptr()) },
            Err(_) => 0,
        }
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

    /// Sets the logging filename for the OMT library.
    ///
    /// If this function is not called, a log file is created in the default location:
    /// - macOS/Linux: `~/.OMT/logs/` folder for this process
    /// - Windows: `C:\ProgramData\OMT\logs`
    ///
    /// To override the default folder used for logs, set the `OMT_STORAGE_PATH`
    /// environment variable prior to calling any OMT functions.
    ///
    /// # Arguments
    ///
    /// * `filename` - Full path to the log file, or `None` to disable logging
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// // Enable logging to a specific file
    /// Settings::set_logging_filename(Some("/tmp/omt.log"));
    ///
    /// // Disable logging
    /// Settings::set_logging_filename(None);
    /// ```
    pub fn set_logging_filename(filename: Option<&str>) {
        use std::ffi::CString;
        use std::ptr;

        unsafe {
            if let Some(name) = filename {
                if let Ok(c_name) = CString::new(name) {
                    omt_sys::omt_setloggingfilename(c_name.as_ptr());
                }
            } else {
                omt_sys::omt_setloggingfilename(ptr::null());
            }
        }
    }

    /// Registers a callback invoked for every log line emitted by the OMT library.
    ///
    /// This complements [`set_logging_filename`](Self::set_logging_filename): it lets
    /// you route log output into your own logging framework instead of (or in addition
    /// to) a file. Registering a new callback replaces any previously registered one.
    ///
    /// The callback may be invoked from arbitrary internal OMT threads, so it must be
    /// `Send + Sync`. Any panic inside the callback is caught and discarded rather than
    /// unwinding across the C boundary. Call
    /// [`clear_logging_callback`](Self::clear_logging_callback) to disable it again.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// Settings::set_logging_callback(|line| println!("[omt] {line}"));
    /// ```
    pub fn set_logging_callback<F>(callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        // Store the closure before registering, so the trampoline always finds it.
        if let Ok(mut guard) = LOG_CALLBACK.write() {
            *guard = Some(Box::new(callback));
        }

        // SAFETY: `logging_trampoline` is a valid `extern "C"` function pointer whose
        // dispatch target now lives in `LOG_CALLBACK`.
        unsafe {
            omt_sys::omt_setloggingcallback(Some(logging_trampoline));
        }
    }

    /// Disables the log line callback previously set with
    /// [`set_logging_callback`](Self::set_logging_callback).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Settings;
    ///
    /// Settings::clear_logging_callback();
    /// ```
    pub fn clear_logging_callback() {
        // Stop the library from invoking the trampoline before dropping the closure.
        // SAFETY: Passing `None` clears the callback in the C library.
        unsafe {
            omt_sys::omt_setloggingcallback(None);
        }

        if let Ok(mut guard) = LOG_CALLBACK.write() {
            *guard = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_integer() {
        // Test setting and getting an integer value
        let test_port = 7500;
        Settings::set_network_port_start(test_port);
        let retrieved_port = Settings::network_port_start();
        assert_eq!(retrieved_port, test_port);

        // Test setting and getting another value
        let test_end_port = 7600;
        Settings::set_network_port_end(test_end_port);
        let retrieved_end_port = Settings::network_port_end();
        assert_eq!(retrieved_end_port, test_end_port);
    }

    #[test]
    fn test_logging_callback_dispatch() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let received = Arc::new(AtomicUsize::new(0));
        let received_in_cb = Arc::clone(&received);

        Settings::set_logging_callback(move |line| {
            if line == "hello" {
                received_in_cb.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Invoke the trampoline exactly as the C library would.
        let message = CString::new("hello").unwrap();
        logging_trampoline(message.as_ptr());

        // A null pointer must be ignored without dispatching.
        logging_trampoline(std::ptr::null());

        assert_eq!(received.load(Ordering::SeqCst), 1);

        Settings::clear_logging_callback();

        // After clearing, further trampoline calls must not reach the closure.
        logging_trampoline(message.as_ptr());
        assert_eq!(received.load(Ordering::SeqCst), 1);
    }
}
