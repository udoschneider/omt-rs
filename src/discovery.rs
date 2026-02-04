//! High-level discovery utilities for Open Media Transport (OMT).
//!
//! OMT publishes sender names on the LAN using DNS-SD (Bonjour/Avahi). When
//! multicast is unavailable, OMT can use a discovery server over TCP.
//! The C API exposes this via `omt_discovery_getaddresses` in `libomt.h`.
//! See <https://github.com/openmediatransport> for protocol background.
//!
//! This module wraps the low-level discovery API with retry/backoff helpers and
//! iterator conveniences.

use crate::ffi;
use crate::types::Address;
use std::collections::HashSet;
use std::ffi::CStr;
use std::time::Duration;

/// Discovers OMT senders advertised on the local network.
pub struct Discovery;

impl Discovery {
    /// Returns discovered sender addresses using a single attempt with no delay.
    ///
    /// This is a convenience wrapper around `get_addresses_with_options`.
    pub fn get_addresses() -> Vec<Address> {
        Self::get_addresses_with_options(1, Duration::from_millis(0), false)
    }

    /// Discovers sender addresses using a fixed delay between attempts.
    ///
    /// - `attempts`: number of discovery attempts (minimum 1).
    /// - `delay`: delay between attempts.
    /// - `debug`: prints diagnostic output during discovery when true.
    pub fn get_addresses_with_options(
        attempts: usize,
        delay: Duration,
        debug: bool,
    ) -> Vec<Address> {
        Self::get_addresses_with_backoff(attempts, delay, delay, 1.0, debug)
    }

    /// Discovers sender addresses with exponential backoff.
    ///
    /// - `attempts`: number of discovery attempts (minimum 1).
    /// - `initial_delay`: initial delay between attempts.
    /// - `max_delay`: upper bound on the delay between attempts.
    /// - `backoff_factor`: multiplier applied after each attempt (values < 1.0 are treated as 1.0).
    /// - `debug`: prints diagnostic output during discovery when true.
    ///
    /// This aggregates unique addresses across all attempts instead of returning
    /// on the first non-empty result.
    pub fn get_addresses_with_backoff(
        attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_factor: f64,
        debug: bool,
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
            let mut count: i32 = 0;
            let list_ptr = unsafe { ffi::omt_discovery_getaddresses(&mut count as *mut i32) };

            if debug {
                println!(
                    "OMT discovery attempt {}/{} -> count={} (delay_ms={})",
                    attempt, attempts, count, delay_ms
                );
            }

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
                    if debug {
                        println!("OMT discovery entry: {}", address);
                    }
                    if seen.insert(address.clone()) {
                        result.push(address);
                    }
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
    /// This collects the current discovery results and returns an owning iterator.
    pub fn addresses() -> impl Iterator<Item = Address> {
        Self::get_addresses().into_iter()
    }

    /// Returns an iterator over discovered sender addresses using backoff options.
    pub fn addresses_with_backoff(
        attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_factor: f64,
        debug: bool,
    ) -> impl Iterator<Item = Address> {
        Self::get_addresses_with_backoff(attempts, initial_delay, max_delay, backoff_factor, debug)
            .into_iter()
    }
}

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
