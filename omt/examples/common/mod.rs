//! Shared helpers for the example programs.

use omt::Discovery;
use std::time::Duration;

/// Polls discovery for the first available OMT source.
///
/// Discovery runs in a background thread, so the first call usually returns an
/// empty list; this helper retries once after a short wait. Returns `None` if
/// no source is found.
pub fn discover_first_sender() -> Option<String> {
    println!("Discovering OMT sources...");
    let addresses = Discovery::get_addresses();

    if !addresses.is_empty() {
        return addresses.into_iter().next();
    }

    println!("No sources found on first attempt, retrying in 2 seconds...");
    std::thread::sleep(Duration::from_secs(2));

    Discovery::get_addresses().into_iter().next()
}
