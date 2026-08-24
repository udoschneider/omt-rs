//! Network discovery for OMT sources.

use crate::error::{Error, Result};
use std::ffi::CStr;
use std::sync::Mutex;

/// Largest source count [`Discovery::get_addresses`] will trust from the C API.
///
/// `omt_discovery_getaddresses` reports how many pointers its array holds, and
/// that count is the only bound on how far this crate walks it. A real network
/// does not carry thousands of simultaneous OMT senders, so a count beyond this
/// is far more likely to be a corrupt or uninitialized value than a real
/// result — and trusting it would mean dereferencing arbitrary memory. The
/// exact figure is a sanity ceiling, not a protocol limit.
pub const MAX_PLAUSIBLE_SOURCES: i32 = 10_000;

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
    /// `Ok(vec![])` therefore means "nothing found *yet*", not "nothing exists"
    /// — callers are expected to poll. It is a normal result, distinct from the
    /// error below.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DiscoveryCountImplausible`] when the C library reports a
    /// source count larger than [`MAX_PLAUSIBLE_SOURCES`]. That is not a busy
    /// network; it means the returned array is corrupt, and walking `count`
    /// pointers into it would be a wild read. This used to be reported as an
    /// empty list, which is exactly the value callers are trained to ignore —
    /// a hard failure was indistinguishable from a cold discovery cache.
    ///
    /// A null array or a non-positive count is *not* an error: that is how the
    /// C API says "no sources", and it yields `Ok(vec![])`.
    ///
    /// # Skipped Entries
    ///
    /// Individual entries that are null, or whose bytes are not valid UTF-8, are
    /// dropped from the result rather than failing the call. One malformed peer
    /// announcing itself on the network must not blind the caller to every other
    /// source — but note that such an address is consequently invisible, and
    /// there is no way to connect to it through this crate.
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
    /// for source in Discovery::get_addresses()? {
    ///     println!("Found source: {}", source);
    /// }
    /// # Ok::<(), omt::Error>(())
    /// ```
    pub fn get_addresses() -> Result<Vec<String>> {
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

        // A null array or non-positive count is the C API's "no sources": a
        // normal, expected answer, especially while discovery is still warming
        // up in its background thread.
        if addresses.is_null() || count <= 0 {
            return Ok(Vec::new());
        }

        // An implausible count means the array is corrupt, not that the network
        // is busy. Report it instead of returning an empty list: silently
        // yielding `[]` made a hard failure look identical to a cold cache.
        if count > MAX_PLAUSIBLE_SOURCES {
            return Err(Error::DiscoveryCountImplausible {
                count,
                max: MAX_PLAUSIBLE_SOURCES,
            });
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
                    // Not valid UTF-8: skip this entry (see "Skipped Entries").
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_get_addresses() {
        // A live network is not required: with no sources this returns an empty
        // list, which is `Ok` rather than an error. What must hold either way is
        // that the call succeeds and every address it does return is a
        // non-empty, owned string.
        let addresses = Discovery::get_addresses().expect("discovery must not fail on a real host");
        for address in &addresses {
            assert!(!address.is_empty());
        }
    }
}
