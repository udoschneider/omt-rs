//! Example demonstrating how to view an OMT video stream in the terminal.
//!
//! This example connects to an OMT video source and displays the video frames
//! directly in the terminal using the `viuer` crate. It automatically discovers
//! sources or accepts a manual address and renders approximately 1 frame per second.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! # Automatically discover and connect to the first available source
//! cargo run --example view_stream
//!
//! # Or specify a source address explicitly
//! cargo run --example view_stream -- "omt://hostname:6400"
//! ```
//!
//! # Features
//!
//! - Automatic discovery of OMT sources or manual address specification
//! - Receives UYVY video frames and converts them to RGB
//! - Renders video frames directly in the terminal with true color support
//! - Throttles display to approximately 1 frame per second for readability
//!
//! # Requirements
//!
//! This example requires a terminal that supports true color (24-bit color).
//! Most modern terminal emulators support this feature.
//!
//! # Note
//!
//! The frame rate is intentionally limited to 1 fps to make the terminal output
//! readable. The actual OMT stream may be running at a much higher frame rate.

use clap::Parser;
use omt::{Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver};
use std::time::{Duration, Instant};

/// View an OMT video stream in the terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// OMT address to connect to (e.g., "omt://hostname:6400" or discovery name).
    address: Option<String>,
}

fn main() {
    let args = Args::parse();

    let address = if let Some(addr) = &args.address {
        addr.clone()
    } else {
        match discover_first_sender() {
            Some(addr) => addr,
            None => {
                eprintln!("Error: No OMT senders discovered. Please provide an address.");
                std::process::exit(1);
            }
        }
    };

    println!("Connecting to: {}", address);

    let receiver = match Receiver::new(
        &address,
        FrameType::VIDEO,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::NONE,
    ) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Error: Failed to create receiver: {}", err);
            std::process::exit(1);
        }
    };

    let config = viuer::Config {
        truecolor: true,
        ..Default::default()
    };

    let frame_interval = Duration::from_secs(1);
    let mut last_frame = Instant::now() - frame_interval;

    loop {
        match receiver.receive(FrameType::VIDEO, 1000) {
            Ok(Some(frame)) => {
                if let Some(image) = frame_to_image(&frame) {
                    let _ = viuer::print(&image, &config);

                    let elapsed = last_frame.elapsed();
                    if elapsed < frame_interval {
                        std::thread::sleep(frame_interval - elapsed);
                    }
                    last_frame = Instant::now();
                }
            }
            Ok(None) => {}
            Err(err) => {
                eprintln!("Receive error: {}", err);
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn frame_to_image(frame: &omt::MediaFrame) -> Option<image::DynamicImage> {
    // Convert frame to RGB8 format
    let rgb_pixels = frame.to_rgb8()?;

    let width = frame.width() as u32;
    let height = frame.height() as u32;

    // Convert RGB8 pixels to byte vector
    let rgb_data: Vec<u8> = rgb_pixels.iter().flat_map(|p| [p.r, p.g, p.b]).collect();

    // Create RGB image from the converted data
    let image = image::RgbImage::from_raw(width, height, rgb_data)?;
    Some(image::DynamicImage::ImageRgb8(image))
}

fn discover_first_sender() -> Option<String> {
    println!("Discovering OMT sources...");
    let addresses = Discovery::get_addresses();

    if !addresses.is_empty() {
        return addresses.into_iter().next();
    }

    println!("No sources found on first attempt, retrying in 2 seconds...");
    std::thread::sleep(Duration::from_secs(2));

    let addresses = Discovery::get_addresses();
    addresses.into_iter().next()
}
