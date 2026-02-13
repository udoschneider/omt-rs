//! Example demonstrating how to create an OMT sender and monitor connections.
//!
//! This example creates an OMT sender that broadcasts on the network, sets sender
//! information metadata, and monitors for incoming connections. It also demonstrates
//! tally state monitoring and connection metadata.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! cargo run --example sender
//! ```
//!
//! The sender will display its address (e.g., `omt://hostname:port`) which can be used
//! by OMT receivers to connect. You can use the `receiver` or `view_stream` examples
//! to connect to this sender.
//!
//! # Features
//!
//! - Creates a high-quality OMT sender
//! - Sets sender information (name, manufacturer, version)
//! - Adds connection metadata for clients
//! - Monitors active connection count
//! - Polls for tally state changes
//!
//! # Note
//!
//! This example creates a sender but does not send actual video/audio frames.
//! See the `send_frames` example for a complete demonstration of frame transmission.

use omt::{Quality, Sender, SenderInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating OMT sender...");

    // Create sender
    let sender = Sender::new("Test Source", Quality::High)?;

    // Set sender information
    let info = SenderInfo::new(
        "OMT Rust Example".to_string(),
        "omt-rs".to_string(),
        "0.1.0".to_string(),
    );
    sender.set_sender_information(&info)?;

    // Get and display the address
    let address = sender.get_address()?;
    println!("Sender address: {}", address);
    println!("Waiting for connections...");

    // Add some connection metadata
    sender.add_connection_metadata("<metadata><test>Hello from Rust!</test></metadata>")?;

    // Monitor for connections
    loop {
        let connections = sender.connections();
        if connections > 0 {
            println!("Active connections: {}", connections);
        }

        // Check tally state
        if let Ok((tally, changed)) = sender.get_tally(1000) {
            if changed {
                println!("Tally changed: {}", tally);
            }
        }

        // In a real application, you would send video/audio frames here
        // For example:
        // sender.send(&video_frame)?;

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
