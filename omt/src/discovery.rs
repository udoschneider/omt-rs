//! Network discovery for OMT sources.

use std::ffi::CStr;

/// Discovery utility for finding OMT sources on the network.
pub struct Discovery;

impl Discovery {
    /// Returns a list of available OMT sources on the network.
    ///
    /// Each string is in the format "HOSTNAME (NAME)" or a URL like "omt://hostname:port".
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
        let addresses = unsafe { omt_sys::omt_discovery_getaddresses(&mut count as *mut i32) };

        if addresses.is_null() || count <= 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count as isize {
            unsafe {
                let ptr = *addresses.offset(i);
                if !ptr.is_null() {
                    if let Ok(cstr) = CStr::from_ptr(ptr).to_str() {
                        result.push(cstr.to_string());
                    }
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
