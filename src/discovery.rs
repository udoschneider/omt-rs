//! Discovery utilities for finding OMT senders on the local network.
//!
//! OMT publishes sender addresses via DNS-SD (Bonjour/Avahi). This module wraps
//! the C API's `omt_discovery_getaddresses` with retry/backoff helpers and iterators.
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
    /// **Note:** The first call starts discovery in a background thread, so results
    /// may be incomplete. Use `get_addresses_with_options` for multiple attempts.
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
    /// # Arguments
    /// * `attempts` - Number of discovery attempts (minimum 1)
    /// * `delay` - Fixed delay between attempts
    pub fn get_addresses_with_options(attempts: usize, delay: Duration) -> Vec<Address> {
        Self::get_addresses_with_backoff(attempts, delay, delay, 1.0)
    }

    /// Returns sender addresses with exponential backoff between attempts.
    ///
    /// Aggregates unique addresses across all attempts.
    ///
    /// # Arguments
    /// * `attempts` - Number of discovery attempts (minimum 1)
    /// * `initial_delay` - Initial delay between attempts
    /// * `max_delay` - Maximum delay between attempts
    /// * `backoff_factor` - Delay multiplier per attempt (minimum 1.0)
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
    pub fn addresses() -> impl Iterator<Item = Address> {
        Self::get_addresses().into_iter()
    }

    /// Returns an iterator over discovered sender addresses with exponential backoff.
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

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
