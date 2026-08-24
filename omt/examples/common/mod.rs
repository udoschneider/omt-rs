//! Shared helpers for the example programs.

use omt::Discovery;
use std::time::Duration;

/// Polls discovery for the first available OMT source.
///
/// Discovery runs in a background thread, so the first call usually returns an
/// empty list; this helper retries once after a short wait. Returns `None` if
/// no source is found.
///
/// A discovery *error* is distinct from an empty result and is reported to
/// stderr rather than silently treated as "no sources".
pub fn discover_first_sender() -> Option<String> {
    println!("Discovering OMT sources...");

    if let Some(address) = first_source() {
        return Some(address);
    }

    println!("No sources found on first attempt, retrying in 2 seconds...");
    std::thread::sleep(Duration::from_secs(2));

    first_source()
}

/// One discovery sweep, reporting failures instead of collapsing them into
/// "nothing found".
fn first_source() -> Option<String> {
    match Discovery::get_addresses() {
        Ok(addresses) => addresses.into_iter().next(),
        Err(err) => {
            eprintln!("Warning: discovery failed: {err}");
            None
        }
    }
}
