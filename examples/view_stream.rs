use clap::Parser;
use log::{error, info};
use omt::{
    helpers::{discover_first_sender, discover_matching_sender},
    Address, FrameRef, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver, Timeout,
};
use std::env;
use std::time::{Duration, Instant};

/// View an OMT video stream in the terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sender name to filter discovery (case-insensitive)
    #[arg(long)]
    sender: Option<String>,

    /// Stream name to filter discovery (case-insensitive)
    #[arg(long)]
    name: Option<String>,

    /// Explicit OMT address to connect to (overrides discovery)
    #[arg(long)]
    address: Option<String>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let address = if let Some(addr) = args.address {
        Address::from(addr)
    } else if args.sender.is_some() || args.name.is_some() {
        match discover_matching_sender(args.sender.as_deref(), args.name.as_deref()) {
            Some(addr) => addr,
            None => {
                error!("No matching OMT sender found for --sender/--name.");
                return;
            }
        }
    } else {
        match discover_first_sender() {
            Some(addr) => addr,
            None => {
                error!("No OMT senders discovered. Use --sender/--name or pass an address.");
                return;
            }
        }
    };

    info!("Connecting to: {}", address);

    let mut receiver = match Receiver::create(
        &address,
        FrameType::Video,
        PreferredVideoFormat::UYVYorBGRA,
        ReceiveFlags::NONE,
    ) {
        Ok(r) => r,
        Err(err) => {
            error!("Failed to create receiver: {}", err);
            std::process::exit(1);
        }
    };

    let config = viuer::Config {
        truecolor: true,
        ..Default::default()
    };

    let fps = env::var("OMTRS_VIEW_FPS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .unwrap_or(1.0);
    let frame_interval = Duration::from_secs_f64(1.0 / fps);
    let mut last_frame = Instant::now() - frame_interval;

    loop {
        match receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
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
                error!("Receive error: {}", err);
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn frame_to_image(frame: &FrameRef) -> Option<image::DynamicImage> {
    let video = frame.video()?;

    // Use VideoFrame.rgb8_data() API to convert to RGB format
    let rgb_data = video.rgb8_data()?;

    let width = video.width() as u32;
    let height = video.height() as u32;

    // RGB data should be 3 bytes per pixel
    if rgb_data.len() < (width * height * 3) as usize {
        return None;
    }

    // Create RGB image from the converted data
    let image = image::RgbImage::from_raw(width, height, rgb_data)?;
    Some(image::DynamicImage::ImageRgb8(image))
}
