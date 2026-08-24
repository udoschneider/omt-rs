//! Example demonstrating how to receive, process, and rebroadcast an OMT video stream.
//!
//! This example connects to an OMT video source, converts each frame to grayscale
//! (black and white), and rebroadcasts it as a new OMT stream. It demonstrates
//! real-time video processing and the use of both receiver and sender simultaneously.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! # Automatically discover and connect to the first available source
//! cargo run --example rebroadcast_bw
//!
//! # Or specify a source address explicitly
//! cargo run --example rebroadcast_bw -- "omt://hostname:6400"
//! ```
//!
//! The rebroadcast stream will be available with " (BW)" appended to the original
//! stream name, or as "OMT Stream (BW)" if the name cannot be determined.
//!
//! # Features
//!
//! - Automatic discovery of sources or manual address specification
//! - Receives UYVY video frames from source
//! - Converts frames to grayscale by neutralizing chrominance (U and V components)
//! - Rebroadcasts processed frames as a new OMT stream
//! - Preserves original frame rate, aspect ratio, and timing
//!
//! # How It Works
//!
//! The grayscale conversion works by setting the U and V (chrominance) components
//! of UYVY frames to 128 (neutral), while preserving the Y (luma) values. This
//! removes all color information while maintaining brightness levels.

mod common;

use clap::Parser;
use common::discover_first_sender;
use omt::{
    Codec, ColorSpace, FrameRate, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, Receiver,
    Sender, VideoFrameBuilder,
};

use std::time::Duration;

/// Rebroadcast an OMT video stream in black and white
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

    let rebroadcast_name = extract_stream_name(&address)
        .map(|name| format!("{} (BW)", name))
        .unwrap_or_else(|| "OMT Stream (BW)".to_string());

    println!("Connecting to: {}", address);
    println!("Rebroadcast name: {}", rebroadcast_name);

    let mut receiver = match Receiver::new(
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

    let sender = match Sender::new(&rebroadcast_name, Quality::Default) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error: Failed to create sender: {}", err);
            std::process::exit(1);
        }
    };

    println!(
        "Rebroadcasting at: {}",
        sender.get_address().unwrap_or_default()
    );

    loop {
        match receiver.receive(FrameType::VIDEO, 1000) {
            Ok(Some(frame)) => {
                // Check if we got UYVY format
                if frame.codec() != Some(Codec::Uyvy) {
                    eprintln!("Warning: Expected UYVY codec, got {:?}", frame.codec());
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let timestamp = frame.timestamp();
                let frame_rate = frame.frame_rate_rational().unwrap_or(FrameRate::fps_30());
                let aspect_ratio = frame.aspect_ratio();
                let color_space = frame.color_space().unwrap_or(ColorSpace::Undefined);
                let flags = frame.flags();
                let width = frame.width();
                let height = frame.height();
                let stride = frame.stride();

                // Get the raw UYVY data
                let uyvy_data = frame.data();

                // Convert to grayscale UYVY by setting U and V to 128 (neutral)
                let Some(bw_uyvy) = uyvy_to_grayscale(uyvy_data, width, height, stride) else {
                    eprintln!(
                        "Warning: frame buffer ({} bytes) too small for {}x{} at stride {}; skipping",
                        uyvy_data.len(),
                        width,
                        height,
                        stride
                    );
                    continue;
                };

                // Build and send the grayscale frame. Both steps report their
                // errors: swallowing them here made a mis-sized buffer look like
                // a working rebroadcast that silently sent nothing.
                match VideoFrameBuilder::new()
                    .codec(Codec::Uyvy)
                    .dimensions(width, height)
                    .stride(stride)
                    .flags(flags)
                    .frame_rate(frame_rate)
                    .aspect_ratio(aspect_ratio)
                    .color_space(color_space)
                    .timestamp(timestamp)
                    .data(bw_uyvy)
                    .build()
                {
                    Ok(owned_frame) => {
                        let media_frame = owned_frame.as_media_frame();
                        if let Err(e) = sender.send(&media_frame) {
                            eprintln!("Error: Failed to send frame: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Error: Failed to build grayscale frame: {}", e),
                }
            }
            Ok(None) => {
                // Timeout: no frame this cycle.
            }
            Err(err) => {
                eprintln!("Error: Receive error: {}", err);
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

fn extract_stream_name(address: &str) -> Option<String> {
    let start = address.find('(')?;
    let end = address.rfind(')')?;
    if start + 1 >= end {
        return None;
    }
    Some(address[start + 1..end].trim().to_string())
}

/// Converts UYVY data to grayscale by setting U and V components to 128 (neutral chrominance).
///
/// UYVY format layout: [U0 Y0 V0 Y1] [U1 Y2 V1 Y3] ...
/// Each macropixel (4 bytes) contains chroma for 2 pixels and luma for each pixel.
///
/// To create grayscale, we keep the Y (luma) values and set U and V to 128,
/// which represents zero chrominance (no color information).
///
/// The output keeps the source row pitch, padding included, so it still matches
/// the `stride` the rebuilt frame declares. Returns `None` if the buffer is too
/// small for the declared geometry.
fn uyvy_to_grayscale(uyvy_data: &[u8], width: i32, height: i32, stride: i32) -> Option<Vec<u8>> {
    let width = usize::try_from(width).ok()?;
    let height = usize::try_from(height).ok()?;
    let stride = usize::try_from(stride).ok()?;

    // Copy the whole plane — padding bytes included — then neutralize chroma in
    // place. Rebuilding the rows from scratch is what previously dropped the
    // inter-row padding, leaving a buffer too short for the declared stride.
    let plane = stride.checked_mul(height)?;
    let mut output = uyvy_data.get(..plane)?.to_vec();

    // Live pixels occupy `width * 2` bytes per row; anything beyond that is
    // padding and is left exactly as it arrived.
    let live = width.checked_mul(2)?.min(stride);
    for row in output.chunks_mut(stride) {
        for macropixel in row[..live].as_chunks_mut::<4>().0 {
            macropixel[0] = 128; // U = 128 (neutral)
            macropixel[2] = 128; // V = 128 (neutral)
        }
    }

    Some(output)
}
