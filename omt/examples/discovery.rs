//! Example demonstrating automatic network discovery of OMT sources.
//!
//! This example continuously scans the network for available OMT sources and, for
//! each newly seen one, connects briefly to read a single video frame so it can
//! display the source's resolution, frame rate, codec, and color space alongside
//! its address. It refreshes every 5 seconds to detect new sources or sources
//! that have gone offline.
//!
//! Successful probes are cached per address, so a source is only connected to
//! once no matter how long the scan runs. Video properties change rarely, and
//! reconnecting to every sender on the network every 5 seconds would be a
//! needless disturbance to them.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! cargo run --example discovery
//! ```
//!
//! The discovery service will scan the local network and display all available OMT
//! sources with their addresses in the format: `omt://hostname:port (Stream Name)`.
//!
//! # Features
//!
//! - Automatic network scanning for OMT sources
//! - Probes each source once for video resolution, frame rate, codec, and color
//!   space, without decoding a single pixel
//! - Continuous refresh every 5 seconds
//! - Press Ctrl+C to exit
//!
//! # Note
//!
//! Discovery uses multicast DNS (mDNS) to find sources on the local network.
//! Ensure your network allows mDNS traffic for discovery to work properly.

use omt::{Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver};
use std::collections::HashMap;

/// How long to wait for a source's first video frame before giving up on it.
const PROBE_TIMEOUT_MS: i32 = 1000;

/// One discovery sweep. Returns `None` — after reporting the cause — when the
/// lookup itself failed, which is distinct from finding no sources.
fn scan() -> Option<Vec<String>> {
    match Discovery::get_addresses() {
        Ok(sources) => Some(sources),
        Err(err) => {
            eprintln!("Error: discovery failed: {err}");
            None
        }
    }
}

fn main() {
    println!("Scanning network for OMT sources...\n");

    let mut first_attempt = true;
    // Address -> video details, for sources that have been probed successfully.
    let mut probed: HashMap<String, String> = HashMap::new();

    loop {
        // A discovery failure is reported and retried rather than mistaken for
        // an empty network — this loop runs forever, so it must not go quiet.
        let Some(mut sources) = scan() else {
            std::thread::sleep(std::time::Duration::from_secs(2));
            continue;
        };

        // Retry after 2 seconds on the first attempt if no sources found
        if first_attempt && sources.is_empty() {
            println!("No sources found on first attempt, retrying in 2 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(2));
            let Some(retried) = scan() else { continue };
            sources = retried;
            first_attempt = false;
        }

        if sources.is_empty() {
            println!("No sources found.");
        } else {
            println!("Found {} source(s):", sources.len());

            // Forget sources that went away, so a source that returns is
            // re-probed rather than reported from a stale cache entry.
            probed.retain(|address, _| sources.contains(address));

            for (i, source) in sources.iter().enumerate() {
                let details = match probed.get(source) {
                    Some(cached) => cached.clone(),
                    // Only successful probes are cached: a source that was idle
                    // or unreachable gets another chance on the next refresh.
                    None => match probe_source(source) {
                        Ok(details) => {
                            probed.insert(source.clone(), details.clone());
                            details
                        }
                        Err(failure) => failure,
                    },
                };

                println!("  {}. {} - {}", i + 1, source, details);
            }
            first_attempt = false;
        }

        println!("\nRefreshing in 5 seconds... (Ctrl+C to exit)");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!();
    }
}

/// Connects to `address`, receives a single video frame, and returns a one-line
/// description of its resolution, frame rate, codec, and color space.
///
/// Returns `Err` with a displayable reason when the source could not be probed;
/// callers should retry those later rather than caching the result.
fn probe_source(address: &str) -> Result<String, String> {
    // `COMPRESSED_ONLY` delivers the sender's original VMX1 payload without
    // decoding it. The probe therefore costs no color conversion, and
    // `codec()` reports the codec actually on the wire — decoding into a
    // preferred format would just report that format back. The frame header
    // (dimensions, frame rate, color space) is populated either way.
    let mut receiver = match Receiver::new(
        address,
        FrameType::VIDEO,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::COMPRESSED_ONLY,
    ) {
        Ok(receiver) => receiver,
        // `Receiver::new` only allocates the local handle, so this is a local
        // failure rather than the source being unreachable.
        Err(err) => return Err(format!("probe failed: {err}")),
    };

    match receiver.receive(FrameType::VIDEO, PROBE_TIMEOUT_MS) {
        Ok(Some(frame)) => {
            let fps = frame.frame_rate_rational().map_or_else(
                || format!("{:.2} fps", frame.frame_rate()),
                |r| r.to_string(),
            );
            let codec = frame
                .codec()
                .map_or_else(|| "unknown codec".to_string(), |c| c.to_string());
            let color_space = frame
                .color_space()
                .map_or_else(|| "unknown color space".to_string(), |c| c.to_string());

            Ok(format!(
                "{}x{} @ {}, {}, {}",
                frame.width(),
                frame.height(),
                fps,
                codec,
                color_space
            ))
        }
        // A timeout means no video arrived in time: the source may be
        // unreachable, or simply not sending video right now.
        Ok(None) => Err(format!("no video within {PROBE_TIMEOUT_MS}ms")),
        Err(err) => Err(format!("receive error: {err}")),
    }
}
