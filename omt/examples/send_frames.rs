//! Example demonstrating how to create and send video, audio, and metadata frames.

use omt::{
    AudioFrameBuilder, Codec, MetadataFrameBuilder, Quality, Sender, SenderInfo, VideoFlags,
    VideoFrameBuilder,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating OMT sender...");

    // Create sender
    let sender = Sender::new("Frame Builder Example", Quality::High)?;

    // Set sender information
    let info = SenderInfo::new(
        "OMT Rust Frame Example".to_string(),
        "omt-rs".to_string(),
        "0.1.0".to_string(),
    );
    sender.set_sender_information(&info)?;

    println!("Sender created at: {}", sender.get_address()?);
    println!("\nPress Ctrl+C to stop sending frames...\n");

    let mut frame_count = 0u64;
    let start_time = std::time::Instant::now();

    loop {
        // Calculate timestamp (10,000,000 units = 1 second)
        let elapsed = start_time.elapsed();
        let timestamp = (elapsed.as_secs_f64() * 10_000_000.0) as i64;

        // Send a video frame every iteration
        if frame_count % 1 == 0 {
            send_video_frame(&sender, timestamp, frame_count)?;
        }

        // Send an audio frame every iteration (with more samples than video frames)
        send_audio_frame(&sender, timestamp)?;

        // Send metadata every 30 frames
        if frame_count % 30 == 0 {
            send_metadata_frame(&sender, timestamp, frame_count)?;
        }

        frame_count += 1;

        // Print status
        if frame_count % 30 == 0 {
            println!(
                "Sent {} frames, {} connections",
                frame_count,
                sender.connections()
            );

            // Print statistics
            let video_stats = sender.get_video_statistics();
            let audio_stats = sender.get_audio_statistics();
            println!(
                "  Video: {} frames, {} bytes sent",
                video_stats.frames, video_stats.bytes_sent
            );
            println!(
                "  Audio: {} frames, {} bytes sent",
                audio_stats.frames, audio_stats.bytes_sent
            );
        }

        // Sleep to maintain ~30fps
        std::thread::sleep(Duration::from_millis(33));
    }
}

/// Creates and sends a test pattern video frame.
fn send_video_frame(
    sender: &Sender,
    timestamp: i64,
    frame_count: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = 1280;
    let height = 720;

    // Create a simple test pattern (UYVY format)
    // UYVY is 2 bytes per pixel (16bpp)
    let data = create_uyvy_test_pattern(width, height, frame_count);

    // Build the video frame
    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .stride(width * 2)
        .frame_rate(30, 1)
        .aspect_ratio(16.0 / 9.0)
        .flags(VideoFlags::NONE)
        .timestamp(timestamp)
        .data(data)
        .build()?;

    // Send the frame
    let media_frame = frame.as_media_frame();
    sender.send(&media_frame)?;

    Ok(())
}

/// Creates and sends an audio frame with silence or test tone.
fn send_audio_frame(sender: &Sender, timestamp: i64) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1600i32; // ~33ms at 48kHz

    // Create planar audio data (silence for this example)
    // In planar format: all samples for channel 0, then all for channel 1, etc.
    let mut audio_samples = vec![0.0f32; (samples_per_channel * channels) as usize];

    // Optionally add a simple test tone (440 Hz sine wave)
    let frequency = 440.0; // A4 note
    for i in 0..samples_per_channel as usize {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1;
        audio_samples[i] = sample; // Left channel
        audio_samples[samples_per_channel as usize + i] = sample; // Right channel
    }

    // Convert f32 samples to bytes
    let data = audio_samples
        .iter()
        .flat_map(|&f| f.to_ne_bytes())
        .collect::<Vec<u8>>();

    // Build the audio frame
    let frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .timestamp(timestamp)
        .data(data)
        .build()?;

    // Send the frame
    let media_frame = frame.as_media_frame();
    sender.send(&media_frame)?;

    Ok(())
}

/// Creates and sends a metadata frame.
fn send_metadata_frame(
    sender: &Sender,
    timestamp: i64,
    frame_count: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create XML metadata
    let metadata = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
    <frame_count>{}</frame_count>
    <timestamp>{}</timestamp>
    <source>OMT Rust Frame Builder Example</source>
</metadata>"#,
        frame_count, timestamp
    );

    // Build the metadata frame
    let frame = MetadataFrameBuilder::new()
        .timestamp(timestamp)
        .metadata(metadata)
        .build()?;

    // Send the frame
    let media_frame = frame.as_media_frame();
    sender.send(&media_frame)?;

    Ok(())
}

/// Creates a simple UYVY test pattern (color bars that animate).
fn create_uyvy_test_pattern(width: i32, height: i32, frame_count: u64) -> Vec<u8> {
    let stride = width * 2; // UYVY is 2 bytes per pixel
    let mut data = vec![0u8; (stride * height) as usize];

    // Animate the pattern based on frame count
    let offset = (frame_count % 255) as u8;

    for y in 0..height {
        for x in 0..(width / 2) {
            // UYVY packs 2 pixels into 4 bytes: U Y V Y
            let pixel_index = (y * stride + x * 4) as usize;

            // Create color bars that change over time
            let bar = (x * 8 / width) as u8;
            let u = bar.wrapping_mul(32).wrapping_add(offset);
            let y_val = 128u8.wrapping_add((y as u8).wrapping_mul(bar).wrapping_div(4));
            let v = (255 - bar.wrapping_mul(32)).wrapping_add(offset);

            if pixel_index + 3 < data.len() {
                data[pixel_index] = u; // U
                data[pixel_index + 1] = y_val; // Y0
                data[pixel_index + 2] = v; // V
                data[pixel_index + 3] = y_val; // Y1
            }
        }
    }

    data
}
