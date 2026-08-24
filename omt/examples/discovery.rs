//! Example demonstrating automatic network discovery of OMT sources.
//!
//! This example continuously scans the network for available OMT sources and, for
//! each one, connects briefly to read a single video frame so it can display the
//! source's resolution, frame rate, codec, and color space alongside its address.
//! It refreshes every 5 seconds to detect new sources or sources that have gone
//! offline.
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
//! - Probes each source for video resolution, frame rate, codec, and color space
//! - Continuous refresh every 5 seconds
//! - Press Ctrl+C to exit
//!
//! # Note
//!
//! Discovery uses multicast DNS (mDNS) to find sources on the local network.
//! Ensure your network allows mDNS traffic for discovery to work properly.

use omt::{ColorSpace, Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver};

fn main() {
    println!("Scanning network for OMT sources...\n");

    let mut first_attempt = true;

    loop {
        let mut sources = Discovery::get_addresses();

        // Retry after 2 seconds on the first attempt if no sources found
        if first_attempt && sources.is_empty() {
            println!("No sources found on first attempt, retrying in 2 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(2));
            sources = Discovery::get_addresses();
            first_attempt = false;
        }

        if sources.is_empty() {
            println!("No sources found.");
        } else {
            println!("Found {} source(s):", sources.len());
            for (i, source) in sources.iter().enumerate() {
                println!("  {}. {} - {}", i + 1, source, probe_source(source));
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
fn probe_source(address: &str) -> String {
    let mut receiver = match Receiver::new(
        address,
        FrameType::VIDEO,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::NONE,
    ) {
        Ok(receiver) => receiver,
        Err(_) => return "unreachable".to_string(),
    };

    match receiver.receive(FrameType::VIDEO, 1000) {
        Ok(Some(frame)) => {
            let fps = frame
                .frame_rate_rational()
                .map(|rate| rate.to_string())
                .unwrap_or_else(|| format!("{:.2} fps", frame.frame_rate()));
            let codec = frame
                .codec()
                .map(|codec| codec.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            format!(
                "{}x{} @ {}, {}, {}",
                frame.width(),
                frame.height(),
                fps,
                codec,
                format_color_space(frame.color_space())
            )
        }
        Ok(None) => "no video frame".to_string(),
        Err(_) => "receive error".to_string(),
    }
}

/// Formats a color space for display.
fn format_color_space(color_space: Option<ColorSpace>) -> &'static str {
    match color_space {
        Some(ColorSpace::Bt601) => "BT.601",
        Some(ColorSpace::Bt709) => "BT.709",
        Some(ColorSpace::Undefined) | None => "undefined",
    }
}
