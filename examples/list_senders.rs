use log::{error, info};
use omt::{
    helpers::discover_addresses, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver, Timeout,
    VideoFlags,
};
use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    // Use the helper function to discover sender addresses.
    let addresses = discover_addresses();

    if addresses.is_empty() {
        info!("No OMT senders discovered.");
        return;
    }

    info!("Discovered {} sender(s):", addresses.len());
    for address in addresses {
        info!("- {}", address);

        let mut receiver = match Receiver::create(
            &address,
            FrameType::Video,
            PreferredVideoFormat::UYVYorBGRA,
            ReceiveFlags::NONE,
        ) {
            Ok(r) => r,
            Err(err) => {
                error!("  -> Failed to create receiver: {}", err);
                continue;
            }
        };

        match receiver.get_sender_info() {
            Some(info) => {
                info!("  -> SenderInfo:");
                info!("     ProductName: {}", info.product_name);
                info!("     Manufacturer: {}", info.manufacturer);
                info!("     Version: {}", info.version);
                info!("     Reserved1: {}", info.reserved1);
                info!("     Reserved2: {}", info.reserved2);
                info!("     Reserved3: {}", info.reserved3);
            }
            None => {
                info!("  -> SenderInfo: <none>");
            }
        }

        // Fetch a single sample frame.
        match receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
            Ok(Some(frame)) => {
                let codec = frame.codec().fourcc_string();
                let flags = describe_video_flags(frame.flags());
                let (fr_n, fr_d) = frame.frame_rate();

                info!(
                    "  -> Video: {}x{} @ {}/{} fps, codec {}, flags [{}], colorspace {:?}",
                    frame.width(),
                    frame.height(),
                    fr_n,
                    fr_d,
                    codec,
                    flags,
                    frame.color_space()
                );
            }
            Ok(None) => {
                info!("  -> No video frame received (timeout).");
            }
            Err(err) => {
                error!("  -> Failed to receive frame: {}", err);
            }
        }
    }
}

fn describe_video_flags(flags: VideoFlags) -> String {
    let mut parts = Vec::new();

    if flags.contains(VideoFlags::INTERLACED) {
        parts.push("Interlaced");
    }
    if flags.contains(VideoFlags::ALPHA) {
        parts.push("Alpha");
    }
    if flags.contains(VideoFlags::PREMULTIPLIED) {
        parts.push("PreMultiplied");
    }
    if flags.contains(VideoFlags::PREVIEW) {
        parts.push("Preview");
    }
    if flags.contains(VideoFlags::HIGH_BIT_DEPTH) {
        parts.push("HighBitDepth");
    }

    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(", ")
    }
}
