use crate::{Address, Discovery, Timeout};
use std::env;

/// Discover OMT sender addresses with configurable backoff.
///
/// Reads environment variables to configure discovery behavior:
/// - `OMTRS_DISCOVERY_ATTEMPTS`: number of discovery attempts (default: 5)
/// - `OMTRS_DISCOVERY_INITIAL_DELAY_MS`: initial delay between attempts in milliseconds (default: 200)
/// - `OMTRS_DISCOVERY_MAX_DELAY_MS`: maximum delay between attempts in milliseconds (default: same as initial_delay_ms)
/// - `OMTRS_DISCOVERY_BACKOFF`: backoff factor (default: 1.0)
pub fn discover_addresses() -> Vec<Address> {
    let attempts = env::var("OMTRS_DISCOVERY_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    let initial_delay_ms = env::var("OMTRS_DISCOVERY_INITIAL_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(200);
    let max_delay_ms = env::var("OMTRS_DISCOVERY_MAX_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(initial_delay_ms);
    let backoff = env::var("OMTRS_DISCOVERY_BACKOFF")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);

    Discovery::get_addresses_with_backoff(
        attempts,
        Timeout::from_millis(initial_delay_ms).as_duration(),
        Timeout::from_millis(max_delay_ms).as_duration(),
        backoff,
    )
}

/// Find the first discovered sender address.
pub fn discover_first_sender() -> Option<Address> {
    discover_addresses().into_iter().next()
}

/// Find a sender address matching optional sender and stream name filters.
///
/// - `sender`: if provided, the address must start with this string (case-insensitive)
/// - `stream`: if provided, the address must contain `(stream)` (case-insensitive)
pub fn discover_matching_sender(sender: Option<&str>, stream: Option<&str>) -> Option<Address> {
    let addresses = discover_addresses();

    let sender_lc = sender.map(|s| s.to_lowercase());
    let stream_lc = stream.map(|s| s.to_lowercase());

    for address in addresses {
        let address_lc = address.as_str().to_lowercase();

        if let Some(sender) = sender_lc.as_deref() {
            if !address_lc.starts_with(sender) {
                continue;
            }
        }

        if let Some(stream) = stream_lc.as_deref() {
            let needle = format!("({})", stream);
            if !address_lc.contains(&needle) {
                continue;
            }
        }

        return Some(address);
    }

    None
}
