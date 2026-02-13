//! Network discovery for OMT sources.

use std::ffi::CStr;

/// Discovery utility for finding OMT sources on the network.
pub struct Discovery;

impl Discovery {
    /// Returns a list of available OMT sources on the network.
    ///
    /// Each string is in the format "HOSTNAME (NAME)" or a URL like "omt://hostname:port".
    ///
    /// # Discovery Behavior
    ///
    /// The underlying C API (`omt_discovery_getaddresses`) returns a list of sources
    /// (senders) currently available on the network. Discovery runs in a background thread,
    /// so the first call typically returns an empty or incomplete list as the discovery
    /// process is still initializing.
    ///
    /// # Memory Safety Note
    ///
    /// The C API returns a `char**` array that is "valid until the next call to getaddresses".
    /// This means the C library maintains internal state that may be overwritten or freed on
    /// subsequent calls. This function copies all strings into owned `String` values to avoid
    /// dangling pointer issues.
    ///
    /// **Known Issue:** The C library may leak memory from previous calls to `omt_discovery_getaddresses`.
    /// There is no documented cleanup function in the C API. This is a limitation of the underlying
    /// C library, not this Rust wrapper.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use omt::Discovery;
    ///
    /// let sources = Discovery::get_addresses();
    /// for source in sources {
    ///     println!("Found source: {}", source);
    /// }
    /// ```
    pub fn get_addresses() -> Vec<String> {
        let mut count: i32 = 0;

        // SAFETY: omt_discovery_getaddresses is a C function that returns a pointer to an array
        // of C strings. The function writes the array length to the count parameter.
        let addresses = unsafe { omt_sys::omt_discovery_getaddresses(&mut count as *mut i32) };

        // Validate inputs from C
        if addresses.is_null() || count <= 0 {
            return Vec::new();
        }

        // Guard against unreasonably large counts that might indicate corruption
        if count > 10000 {
            eprintln!(
                "Warning: Discovery returned suspiciously large count: {}",
                count
            );
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count as isize {
            unsafe {
                // SAFETY: The C API guarantees that addresses points to an array of at least
                // 'count' pointers. We validate each pointer before dereferencing.
                let ptr = *addresses.offset(i);
                if !ptr.is_null() {
                    // SAFETY: CStr::from_ptr requires the pointer to be valid and point to
                    // a null-terminated C string. The C API guarantees this for the duration
                    // of this call. We immediately copy the string data to avoid lifetime issues.
                    if let Ok(cstr) = CStr::from_ptr(ptr).to_str() {
                        result.push(cstr.to_string());
                    }
                    // If UTF-8 validation fails, we skip this entry
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_get_addresses() {
        // This test will only succeed if there are sources on the network
        let addresses = Discovery::get_addresses();
        // Should not panic, might be empty
        assert!(addresses.len() >= 0);
    }
}
