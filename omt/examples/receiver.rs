//! Example demonstrating how to receive video and audio frames from an OMT source.
//!
//! This example discovers available OMT sources on the network, connects to the first
//! available source (or a default address if none found), and receives both video and
//! audio frames for 10 seconds while displaying frame information and statistics.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! cargo run --example receiver
//! ```
//!
//! The receiver will automatically discover sources or connect to `omt://localhost:6400`.
//!
//! # Features
//!
//! - Automatic network discovery of OMT sources
//! - Receives both video and audio frames simultaneously
//! - Displays frame information (dimensions, frame rate, codec, sample rate, etc.)
//! - Shows transmission statistics after receiving

use omt::{Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover available sources
    println!("Discovering OMT sources...");
    let mut sources = Discovery::get_addresses();

    if sources.is_empty() {
        println!("No sources found on first attempt, retrying in 2 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        sources = Discovery::get_addresses();
    }

    if sources.is_empty() {
        println!("No sources found. Using default address.");
        let address = "omt://localhost:6400";
        connect_and_receive(address)?;
    } else {
        println!("Found {} source(s):", sources.len());
        for source in &sources {
            println!("  - {}", source);
        }

        // Connect to the first source
        connect_and_receive(&sources[0])?;
    }

    Ok(())
}

fn connect_and_receive(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nConnecting to: {}", address);

    // Create receiver for video and audio
    let receiver = Receiver::new(
        address,
        FrameType::VIDEO | FrameType::AUDIO,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::NONE,
    )?;

    println!("Connected! Waiting for frames...");

    // Get sender information if available
    if let Some(info) = receiver.get_sender_information()? {
        println!("Sender: {}", info);
    }

    // Receive frames for 10 seconds
    let start = std::time::Instant::now();
    let mut video_count = 0;
    let mut audio_count = 0;

    while start.elapsed().as_secs() < 10 {
        // Try to receive video frame
        if let Some(frame) = receiver.receive(FrameType::VIDEO, 100)? {
            if frame.frame_type() == FrameType::VIDEO {
                video_count += 1;
                if video_count % 30 == 0 {
                    println!(
                        "Video: {}x{} @ {:.2} fps, codec: {:?}",
                        frame.width(),
                        frame.height(),
                        frame.frame_rate(),
                        frame.codec()
                    );
                }
            }
        }

        // Try to receive audio frame
        if let Some(frame) = receiver.receive(FrameType::AUDIO, 10)? {
            if frame.frame_type() == FrameType::AUDIO {
                audio_count += 1;
                if audio_count % 100 == 0 {
                    println!(
                        "Audio: {} channels @ {}Hz, {} samples",
                        frame.channels(),
                        frame.sample_rate(),
                        frame.samples_per_channel()
                    );
                }
            }
        }
    }

    println!(
        "\nReceived {} video frames and {} audio frames",
        video_count, audio_count
    );

    // Get statistics
    let video_stats = receiver.get_video_statistics();
    let audio_stats = receiver.get_audio_statistics();

    println!("\nVideo Stats: {}", video_stats);
    println!("Audio Stats: {}", audio_stats);

    Ok(())
}
