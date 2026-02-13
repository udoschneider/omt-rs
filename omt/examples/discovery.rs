//! Example demonstrating automatic network discovery of OMT sources.
//!
//! This example continuously scans the network for available OMT sources and displays
//! their addresses. It refreshes every 5 seconds to detect new sources or sources that
//! have gone offline.
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
//! sources with their addresses in the format: `omt://hostname:port (Stream Name)`
//!
//! # Features
//!
//! - Automatic network scanning for OMT sources
//! - Continuous refresh every 5 seconds
//! - Displays source count and addresses
//! - Press Ctrl+C to exit
//!
//! # Note
//!
//! Discovery uses multicast DNS (mDNS) to find sources on the local network.
//! Ensure your network allows mDNS traffic for discovery to work properly.

use omt::Discovery;

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
                println!("  {}. {}", i + 1, source);
            }
            first_attempt = false;
        }

        println!("\nRefreshing in 5 seconds... (Ctrl+C to exit)");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!();
    }
}
