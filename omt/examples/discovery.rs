//! Example showing network discovery of OMT sources.

use omt::Discovery;

fn main() {
    println!("Scanning network for OMT sources...\n");
    
    loop {
        let sources = Discovery::get_addresses();
        
        if sources.is_empty() {
            println!("No sources found.");
        } else {
            println!("Found {} source(s):", sources.len());
            for (i, source) in sources.iter().enumerate() {
                println!("  {}. {}", i + 1, source);
            }
        }
        
        println!("\nRefreshing in 5 seconds... (Ctrl+C to exit)");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!();
    }
}
