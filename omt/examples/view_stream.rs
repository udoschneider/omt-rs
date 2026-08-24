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
//!
//! # Log diagnostics to stderr (helps when nothing renders, e.g. over SSH/tmux)
//! cargo run --example view_stream -- --verbose
//! ```
//!
//! # Features
//!
//! - Automatic discovery of OMT sources or manual address specification
//! - Receives UYVY video frames and converts them to RGB
//! - Renders video frames directly in the terminal, using 24-bit color when the
//!   terminal supports it and falling back to 256-color ANSI otherwise
//! - Throttles display to approximately 1 frame per second for readability
//!
//! # Requirements
//!
//! Any terminal that can render ANSI colors will work. For the best fidelity, use a
//! terminal with 24-bit true color support and ensure `$COLORTERM` is set to
//! `truecolor` or `24bit` (most modern terminal emulators do this automatically).
//!
//! # Note
//!
//! The frame rate is intentionally limited to 1 fps to make the terminal output
//! readable. The actual OMT stream may be running at a much higher frame rate.

mod common;

use clap::Parser;
use common::discover_first_sender;
use omt::{FrameType, PreferredVideoFormat, ReceiveFlags, Receiver};
use std::time::{Duration, Instant};

/// View an OMT video stream in the terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// OMT address to connect to (e.g., "omt://hostname:6400" or discovery name).
    address: Option<String>,

    /// Log diagnostics to stderr (received frames, conversion/render problems).
    ///
    /// Useful for figuring out whether nothing renders because no frames are
    /// arriving or because the terminal cannot display them (e.g. over SSH/tmux).
    #[arg(short, long)]
    verbose: bool,
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

    // `Config::default()` auto-detects truecolor from `$COLORTERM` (falling back to
    // 256-color ANSI) and scales each frame to fit the terminal (width/height = None).
    //
    // Terminal image protocols (kitty/iTerm) do not survive terminal multiplexers, and
    // SSH commonly forwards `LC_TERMINAL`/`TERM_PROGRAM`, which makes viuer believe it is
    // talking to iTerm/kitty directly. Inside tmux/screen those escape codes are swallowed
    // and nothing appears (with no error returned). Detect a multiplexer and force the
    // ANSI block renderer, which always displays.
    let inside_multiplexer = std::env::var_os("TMUX").is_some()
        || std::env::var("TERM")
            .map(|term| term.starts_with("screen") || term.starts_with("tmux"))
            .unwrap_or(false);

    if inside_multiplexer {
        eprintln!(
            "Detected tmux/screen: using ANSI block rendering (terminal image protocols are not forwarded)."
        );
    }

    let config = viuer::Config {
        use_kitty: !inside_multiplexer,
        use_iterm: !inside_multiplexer,
        ..Default::default()
    };

    let frame_interval = Duration::from_secs(1);
    let mut last_frame = Instant::now() - frame_interval;
    let mut frames_seen: u64 = 0;
    let mut last_wait_log = Instant::now();

    loop {
        match receiver.receive(FrameType::VIDEO, 1000) {
            Ok(Some(frame)) => {
                frames_seen += 1;
                if args.verbose {
                    eprintln!(
                        "frame #{frames_seen}: {}x{}, codec {:?}",
                        frame.width(),
                        frame.height(),
                        frame.codec()
                    );
                }

                match frame_to_image(&frame) {
                    Some(image) => {
                        // Surface render failures instead of swallowing them: over
                        // SSH/tmux or on terminals without image support this is the
                        // difference between "no frames" and "can't paint frames".
                        if let Err(err) = viuer::print(&image, &config) {
                            eprintln!("Failed to render frame in this terminal: {err}");
                        }
                    }
                    None if args.verbose => {
                        eprintln!("Skipping frame #{frames_seen}: could not convert to RGB");
                    }
                    None => {}
                }

                let elapsed = last_frame.elapsed();
                if elapsed < frame_interval {
                    std::thread::sleep(frame_interval - elapsed);
                }
                last_frame = Instant::now();
            }
            Ok(None) => {
                if args.verbose && last_wait_log.elapsed() >= Duration::from_secs(2) {
                    eprintln!("Waiting for video frames from {address} (none received yet)...");
                    last_wait_log = Instant::now();
                }
            }
            Err(err) => {
                eprintln!("Receive error: {err}");
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
