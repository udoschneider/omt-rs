//! Discovery utilities for finding OMT senders on the local network.
//!
//! OMT publishes sender addresses via DNS-SD (Bonjour/Avahi). This module wraps
//! the C API's `omt_discovery_getaddresses` with retry/backoff helpers and iterators.
//!
//! # Background
//!
//! The underlying C API (`omt_discovery_getaddresses`) returns a list of sources
//! (senders) currently available on the network. Discovery runs in a background thread,
//! so the first call typically returns an empty or incomplete list as the discovery
//! process is still initializing.
//!
//! The returned array from the C API is valid only until the next call to
//! `omt_discovery_getaddresses`, so this module copies all addresses into owned
//! Rust `Vec<Address>` structures.
//!
//! # Usage Patterns
//!
//! For quick prototyping or interactive use, a single call may suffice:
//! ```rust,no_run
//! use omt::discovery::Discovery;
//!
//! let addresses = Discovery::get_addresses();
//! for addr in addresses {
//!     println!("Found sender: {}", addr);
//! }
//! ```
//!
//! For production use, prefer multiple attempts with delays to allow discovery
//! to complete:
//! ```rust,no_run
//! use omt::discovery::Discovery;
//! use std::time::Duration;
//!
//! let addresses = Discovery::get_addresses_with_options(
//!     3,                          // 3 attempts
//!     Duration::from_millis(500), // 500ms between attempts
//! );
//! ```
//!
//! For more sophisticated scenarios, use exponential backoff:
//! ```rust,no_run
//! use omt::discovery::Discovery;
//! use std::time::Duration;
//!
//! let addresses = Discovery::get_addresses_with_backoff(
//!     5,                            // 5 attempts
//!     Duration::from_millis(100),   // Start with 100ms
//!     Duration::from_millis(2000),  // Cap at 2s
//!     2.0,                          // Double delay each time
//! );
//! ```
//!
//! # Thread Safety
//!
//! The underlying C API uses internal synchronization. All methods in this module
//! are safe to call from multiple threads, though discovery results are shared
//! globally across the process.
//!
//! See <https://github.com/openmediatransport> for protocol details.

use crate::ffi;
use crate::types::Address;
use log::debug;
use std::collections::HashSet;
use std::ffi::CStr;
use std::time::Duration;

/// Discovery interface for OMT senders on the local network.
pub struct Discovery;

impl Discovery {
    /// Returns discovered sender addresses with a single attempt.
    ///
    /// This method performs a single call to the underlying C API's
    /// `omt_discovery_getaddresses` function, which returns an array of UTF-8
    /// char pointers representing available sources on the network.
    ///
    /// # Important Notes
    ///
    /// * **First call behavior:** The first call starts discovery in a background
    ///   thread, so results may be empty or incomplete. Subsequent calls will
    ///   return accumulated results.
    /// * **Array validity:** The C API's returned array is valid only until the
    ///   next call, but this method copies all addresses into an owned `Vec`.
    /// * **Production use:** For reliable discovery, use [`get_addresses_with_options`](Self::get_addresses_with_options)
    ///   or [`get_addresses_with_backoff`](Self::get_addresses_with_backoff) with multiple attempts and delays.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use omt::discovery::Discovery;
    ///
    /// let addresses = Discovery::get_addresses();
    /// println!("Found {} senders", addresses.len());
    /// ```
    ///
    /// # C API Reference
    ///
    /// Wraps `omt_discovery_getaddresses(int* count)` from `libomt.h`.
    pub fn get_addresses() -> Vec<Address> {
        let mut count: i32 = 0;
        let list_ptr = unsafe { ffi::omt_discovery_getaddresses(&mut count as *mut i32) };

        debug!("OMT discovery -> count={}", count);

        let mut result = Vec::new();

        if !list_ptr.is_null() && count > 0 {
            for i in 0..count {
                let entry_ptr = unsafe { *list_ptr.add(i as usize) };
                if entry_ptr.is_null() {
                    continue;
                }
                let address = Address::from(
                    unsafe { CStr::from_ptr(entry_ptr) }
                        .to_string_lossy()
                        .to_string(),
                );
                debug!("OMT discovery entry: {}", address);
                result.push(address);
            }
        }

        result
    }

    /// Returns sender addresses with fixed delay between attempts.
    ///
    /// This method calls the discovery API multiple times with a fixed delay
    /// between each attempt, aggregating unique addresses from all attempts.
    /// This approach is useful when you need a simple retry mechanism without
    /// complex backoff logic.
    ///
    /// # Arguments
    /// * `attempts` - Number of discovery attempts (minimum 1, enforced)
    /// * `delay` - Fixed delay between attempts (e.g., `Duration::from_millis(500)`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use omt::discovery::Discovery;
    /// use std::time::Duration;
    ///
    /// // Try 3 times with 500ms between attempts
    /// let addresses = Discovery::get_addresses_with_options(
    ///     3,
    ///     Duration::from_millis(500),
    /// );
    /// ```
    pub fn get_addresses_with_options(attempts: usize, delay: Duration) -> Vec<Address> {
        Self::get_addresses_with_backoff(attempts, delay, delay, 1.0)
    }

    /// Returns sender addresses with exponential backoff between attempts.
    ///
    /// This method calls the discovery API multiple times with exponentially
    /// increasing delays between attempts, aggregating unique addresses from
    /// all attempts. This approach is recommended for production use as it
    /// balances quick initial discovery with patience for slow-appearing sources.
    ///
    /// The delay follows this pattern:
    /// * Attempt 1: no delay
    /// * Attempt 2: `initial_delay`
    /// * Attempt 3: `initial_delay * backoff_factor`
    /// * Attempt 4: `(initial_delay * backoff_factor²)`, capped at `max_delay`
    /// * ...and so on
    ///
    /// # Arguments
    /// * `attempts` - Number of discovery attempts (minimum 1, enforced)
    /// * `initial_delay` - Initial delay between attempts
    /// * `max_delay` - Maximum delay between attempts (should be ≥ `initial_delay`)
    /// * `backoff_factor` - Delay multiplier per attempt (minimum 1.0, enforced)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use omt::discovery::Discovery;
    /// use std::time::Duration;
    ///
    /// // Recommended for production: 5 attempts with exponential backoff
    /// let addresses = Discovery::get_addresses_with_backoff(
    ///     5,                            // 5 attempts
    ///     Duration::from_millis(100),   // Start with 100ms
    ///     Duration::from_millis(2000),  // Cap at 2s
    ///     2.0,                          // Double each time
    /// );
    /// // Delays will be: 0ms, 100ms, 200ms, 400ms, 800ms
    /// ```
    pub fn get_addresses_with_backoff(
        attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_factor: f64,
    ) -> Vec<Address> {
        let attempts = attempts.max(1);
        let mut delay_ms = duration_ms(initial_delay);
        let max_delay_ms = duration_ms(max_delay).max(delay_ms);
        let backoff_factor = if backoff_factor < 1.0 {
            1.0
        } else {
            backoff_factor
        };

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for attempt in 1..=attempts {
            let addresses = Self::get_addresses();

            debug!(
                "OMT discovery attempt {}/{} -> count={} (delay_ms={})",
                attempt,
                attempts,
                addresses.len(),
                delay_ms
            );

            for address in addresses {
                if seen.insert(address.clone()) {
                    result.push(address);
                }
            }

            if attempt < attempts && delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms));
                if backoff_factor > 1.0 {
                    let next = (delay_ms as f64 * backoff_factor).round() as u64;
                    delay_ms = next.min(max_delay_ms);
                }
            }
        }

        result
    }

    /// Returns an iterator over discovered sender addresses.
    ///
    /// This is a convenience method equivalent to calling `get_addresses().into_iter()`.
    /// It performs a single discovery attempt.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use omt::discovery::Discovery;
    ///
    /// for address in Discovery::addresses() {
    ///     println!("Sender at: {}", address);
    /// }
    /// ```
    pub fn addresses() -> impl Iterator<Item = Address> {
        Self::get_addresses().into_iter()
    }

    /// Returns an iterator over discovered sender addresses with exponential backoff.
    ///
    /// This is a convenience method that combines [`get_addresses_with_backoff`](Self::get_addresses_with_backoff)
    /// with iterator conversion.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use omt::discovery::Discovery;
    /// use std::time::Duration;
    ///
    /// for address in Discovery::addresses_with_backoff(
    ///     3,
    ///     Duration::from_millis(200),
    ///     Duration::from_secs(1),
    ///     1.5,
    /// ) {
    ///     println!("Sender at: {}", address);
    /// }
    /// ```
    pub fn addresses_with_backoff(
        attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_factor: f64,
    ) -> impl Iterator<Item = Address> {
        Self::get_addresses_with_backoff(attempts, initial_delay, max_delay, backoff_factor)
            .into_iter()
    }
}

/// Converts a Duration to milliseconds, capping at u64::MAX.
fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
