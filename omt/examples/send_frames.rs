//! Example demonstrating how to create and send video, audio, and metadata frames.
//!
//! This example loads `testcard.jpg` from the examples directory and sends it as a video stream
//! at 30fps, along with a 1kHz sine wave audio signal at 48kHz sample rate.
//!
//! # Usage
//!
//! Run the example from the workspace root:
//!
//! ```sh
//! cargo run --example send_frames
//! ```
//!
//! The sender will display its address (e.g., `omt://hostname:port`) which can be used
//! by OMT receivers to connect and receive the stream.
//!
//! # Features
//!
//! - Loads JPEG image and converts to BGRA format using the `yuv` crate
//! - Generates continuous 1kHz sine wave audio with proper phase continuity
//! - Sends metadata frames every second with frame count and stream information
//! - Displays connection count and transmission statistics

use image::GenericImageView;
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

    // Load the testcard image once
    let testcard_data = load_testcard_image()?;

    let mut frame_count = 0u64;
    let start_time = std::time::Instant::now();
    let mut audio_sample_offset = 0usize;

    loop {
        // Calculate timestamp (10,000,000 units = 1 second)
        let elapsed = start_time.elapsed();
        let timestamp = (elapsed.as_secs_f64() * 10_000_000.0) as i64;

        // Send a video frame every iteration
        if frame_count % 1 == 0 {
            send_video_frame(&sender, timestamp, &testcard_data)?;
        }

        // Send an audio frame every iteration (with more samples than video frames)
        audio_sample_offset = send_audio_frame(&sender, timestamp, audio_sample_offset)?;

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

/// Struct to hold testcard image data and metadata
struct TestcardData {
    width: i32,
    height: i32,
    bgra_data: Vec<u8>,
}

/// Loads testcard.jpg and converts it to BGRA format.
fn load_testcard_image() -> Result<TestcardData, Box<dyn std::error::Error>> {
    println!("Loading testcard.jpg...");

    // Load the image
    let img = image::open("omt/examples/testcard.jpg")?;
    let (width, height) = img.dimensions();

    println!("Loaded testcard: {}x{} pixels", width, height);

    // Convert to RGBA8, then to BGRA format
    let rgba_img = img.to_rgba8();
    let bgra_data = rgba_to_bgra(&rgba_img, width as i32, height as i32);

    Ok(TestcardData {
        width: width as i32,
        height: height as i32,
        bgra_data,
    })
}

/// Converts RGBA image data to BGRA format using the yuv crate.
fn rgba_to_bgra(rgba_data: &image::RgbaImage, width: i32, height: i32) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;

    // Allocate BGRA output buffer (4 bytes per pixel)
    let mut bgra_data = vec![0u8; width * height * 4];
    let stride = (width * 4) as u32;

    // Shuffle RGBA to BGRA (just swap R and B channels)
    yuv::rgba_to_bgra(
        rgba_data.as_raw(),
        stride,
        &mut bgra_data,
        stride,
        width as u32,
        height as u32,
    )
    .expect("RGBA to BGRA conversion failed");

    bgra_data
}

/// Creates and sends a video frame with the testcard image.
fn send_video_frame(
    sender: &Sender,
    timestamp: i64,
    testcard: &TestcardData,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the video frame
    let frame = VideoFrameBuilder::new()
        .codec(Codec::Bgra)
        .dimensions(testcard.width, testcard.height)
        .stride(testcard.width * 4)
        .frame_rate(30, 1)
        .aspect_ratio(testcard.width as f32 / testcard.height as f32)
        .flags(VideoFlags::NONE)
        .timestamp(timestamp)
        .data(testcard.bgra_data.clone())
        .build()?;

    // Send the frame
    let media_frame = frame.as_media_frame();
    sender.send(&media_frame)?;

    Ok(())
}

/// Creates and sends an audio frame with a 1kHz sine wave.
fn send_audio_frame(
    sender: &Sender,
    timestamp: i64,
    sample_offset: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1600i32; // ~33ms at 48kHz

    // Create planar audio data with 1kHz sine wave
    // In planar format: all samples for channel 0, then all for channel 1, etc.
    let mut audio_samples = vec![0.0f32; (samples_per_channel * channels) as usize];

    // Generate 1kHz sine wave
    let frequency = 1000.0; // 1kHz
    let amplitude = 0.3; // 30% volume to avoid clipping

    for i in 0..samples_per_channel as usize {
        let global_sample = sample_offset + i;
        let t = global_sample as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * amplitude;

        // Write to both channels (left and right)
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

    // Return updated sample offset for continuous phase
    Ok(sample_offset + samples_per_channel as usize)
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
    <video>testcard.jpg</video>
    <audio>1kHz sine wave</audio>
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
