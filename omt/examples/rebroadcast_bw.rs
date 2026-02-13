use clap::Parser;
use omt::{
    Codec, ColorSpace, Discovery, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, Receiver,
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
                let frame_rate_n = frame.frame_rate_numerator();
                let frame_rate_d = frame.frame_rate_denominator();
                let aspect_ratio = frame.aspect_ratio();
                let color_space = frame.color_space().unwrap_or(ColorSpace::Undefined);
                let flags = frame.flags();
                let width = frame.width();
                let height = frame.height();
                let stride = frame.stride();

                // Get the raw UYVY data
                let uyvy_data = frame.data();

                // Convert to grayscale UYVY by setting U and V to 128 (neutral)
                let bw_uyvy = uyvy_to_grayscale(uyvy_data, width, height, stride);

                // Build and send the grayscale frame
                if let Ok(owned_frame) = VideoFrameBuilder::new()
                    .codec(Codec::Uyvy)
                    .dimensions(width, height)
                    .stride(stride)
                    .flags(flags)
                    .frame_rate(frame_rate_n, frame_rate_d)
                    .aspect_ratio(aspect_ratio)
                    .color_space(color_space)
                    .timestamp(timestamp)
                    .data(bw_uyvy)
                    .build()
                {
                    let media_frame = owned_frame.as_media_frame();
                    if let Err(e) = sender.send(&media_frame) {
                        eprintln!("Error: Failed to send frame: {}", e);
                    }
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

fn discover_first_sender() -> Option<String> {
    let addresses = Discovery::get_addresses();
    addresses.into_iter().next()
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
fn uyvy_to_grayscale(uyvy_data: &[u8], width: i32, height: i32, stride: i32) -> Vec<u8> {
    let height = height as usize;
    let stride = stride as usize;
    let width = width as usize;

    // Calculate the actual data size we need
    let data_size = height * stride;
    let mut output = Vec::with_capacity(data_size);

    // Process each row
    for y in 0..height {
        let row_start = y * stride;
        let row_end = row_start + (width * 2).min(stride);

        if row_end <= uyvy_data.len() {
            let row = &uyvy_data[row_start..row_end];

            // Process UYVY macropixels (4 bytes at a time)
            for chunk in row.chunks_exact(4) {
                output.push(128); // U = 128 (neutral)
                output.push(chunk[1]); // Y0 (preserve luma)
                output.push(128); // V = 128 (neutral)
                output.push(chunk[3]); // Y1 (preserve luma)
            }

            // Handle any remaining bytes in the row (padding)
            let processed = (row.len() / 4) * 4;
            if processed < stride {
                output.extend_from_slice(&row[processed..]);
            }
        } else {
            // Safety: if we don't have enough data, fill with neutral values
            for _ in 0..(stride / 4) {
                output.extend_from_slice(&[128, 16, 128, 16]); // Neutral gray
            }
        }
    }

    output
}
