//! Example demonstrating OMT settings convenience methods
//!
//! This example shows how to use the convenience methods for OMT settings
//! documented in libomt.h.

use omt::{
    get_discovery_server, get_network_port_end, get_network_port_range, get_network_port_start,
    set_discovery_server, set_network_port_range,
};

fn main() {
    println!("OMT Settings Example");
    println!("====================\n");

    // Demonstrate network port settings
    println!("Network Port Settings:");
    println!("----------------------");

    // Get current port range (defaults to 6400-6600)
    let (current_start, current_end) = get_network_port_range();
    println!("Current port range: {} - {}", current_start, current_end);

    // Get individual port settings
    println!("NetworkPortStart: {}", get_network_port_start());
    println!("NetworkPortEnd: {}", get_network_port_end());

    // Set new port range
    println!("\nSetting new port range: 6500 - 6700");
    if let Err(e) = set_network_port_range(6500, 6700) {
        eprintln!("Failed to set port range: {}", e);
    } else {
        println!("Successfully set port range");

        // Verify the new settings
        let (new_start, new_end) = get_network_port_range();
        println!("New port range: {} - {}", new_start, new_end);
        println!("NetworkPortStart: {}", get_network_port_start());
        println!("NetworkPortEnd: {}", get_network_port_end());

        // Restore original port range
        println!("\nRestoring original port range: 6400 - 6600");
        set_network_port_range(6400, 6600).unwrap();
    }

    // Demonstrate discovery server settings
    println!("\nDiscovery Server Settings:");
    println!("-------------------------");

    // Get current discovery server (defaults to None for DNS-SD)
    match get_discovery_server() {
        Some(server) if !server.is_empty() => {
            println!("Current DiscoveryServer: {}", server);
        }
        Some(_) => {
            println!("Current DiscoveryServer: <empty> (using DNS-SD)");
        }
        None => {
            println!("Current DiscoveryServer: <not set> (using DNS-SD)");
        }
    }

    // Set a discovery server
    println!("\nSetting DiscoveryServer to 'omt://discovery.example.com:6399'");
    if let Err(e) = set_discovery_server("omt://discovery.example.com:6399") {
        eprintln!("Failed to set DiscoveryServer: {}", e);
    } else {
        println!("Successfully set DiscoveryServer");

        // Verify the new setting
        match get_discovery_server() {
            Some(server) => println!("New DiscoveryServer: {}", server),
            None => println!("New DiscoveryServer: <not set>"),
        }

        // Clear the discovery server (use DNS-SD)
        println!("\nClearing DiscoveryServer (will use DNS-SD)");
        set_discovery_server("").unwrap();

        match get_discovery_server() {
            Some(server) if server.is_empty() => {
                println!("DiscoveryServer cleared (using DNS-SD)");
            }
            _ => {
                println!("DiscoveryServer: <using DNS-SD>");
            }
        }
    }

    println!("\nExample completed successfully!");
}
