//! Network discovery for OMT sources.

use std::ffi::CStr;
use std::sync::Mutex;

/// Serializes calls to `omt_discovery_getaddresses` and the copying of its
/// result.
///
/// `libomt.h` documents the returned `char**` as "valid until the next call to
/// getaddresses" — i.e. it is *process-global* state, not per-instance. Unlike
/// the receive path (which hangs the same hazard on a `&mut self` borrow),
/// [`Discovery::get_addresses`] has no receiver to serialize on, so without
/// this lock two threads calling it concurrently would be a use-after-free
/// reachable from entirely safe code: one thread's call frees and replaces the
/// array while the other is still walking it with `CStr::from_ptr`.
///
/// The lock must be held across *both* the C call and the string copies, not
/// just the call. It guards no data of its own, hence `Mutex<()>`.
///
/// This serializes calls made through this crate. A different library in the
/// same process calling `omt_discovery_getaddresses` directly would still race;
/// that is outside what a wrapper can enforce.
static DISCOVERY_LOCK: Mutex<()> = Mutex::new(());

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
    /// Because that state is *global* rather than per-instance, the C call and
    /// the copying of its result are serialized behind an internal mutex. Calls
    /// from multiple threads are therefore safe; they simply queue. The returned
    /// `String`s are owned and outlive the lock.
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
        // Held until every string has been copied out — see `DISCOVERY_LOCK`.
        // A poisoned lock is recovered rather than propagated: this guard
        // protects no invariant of its own (nothing observable is left
        // half-updated by a panic while it is held), and the crate's no-panics
        // rule rules out unwrapping.
        let _guard = DISCOVERY_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let mut count: i32 = 0;

        // SAFETY: omt_discovery_getaddresses is a C function that returns a pointer to an array
        // of C strings. The function writes the array length to the count parameter.
        // `_guard` serializes this call against any concurrent one, so the
        // returned array cannot be freed out from under the loop below.
        let addresses = unsafe { omt_sys::omt_discovery_getaddresses(&mut count as *mut i32) };

        // Validate inputs from C
        if addresses.is_null() || count <= 0 {
            return Vec::new();
        }

        // Guard against unreasonably large counts that might indicate corruption.
        // A library must not write to stderr on the caller's behalf, so this is
        // handled silently: an implausible count is treated as "no results"
        // rather than trusted enough to index into.
        if count > 10000 {
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
        // Should not panic; the list might be empty, but any returned address
        // must be a non-empty, owned string.
        for address in &addresses {
            assert!(!address.is_empty());
        }
    }
}
