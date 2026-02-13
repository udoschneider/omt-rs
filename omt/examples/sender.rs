//! Example OMT sender that broadcasts a test pattern.

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
