use log::{error, info};
use omt::{
    fourcc_to_string, settings_get_string, settings_set_string, Address, Discovery, FrameType,
    PreferredVideoFormat, ReceiveFlags, Receiver, Timeout, VideoFlags,
};
use std::env;

fn main() {
    env_logger::init();
    if let Ok(server) = env::var("OMTRS_DISCOVERY_SERVER") {
        let server = server.trim().to_string();
        if !server.is_empty() {
            if let Err(err) = settings_set_string("DiscoveryServer", &server) {
                error!("Failed to set DiscoveryServer: {}", err);
            }
        }
    }

    let current = settings_get_string("DiscoveryServer").unwrap_or_else(|| "<default>".to_string());
    info!("DiscoveryServer: {}", current);

    let attempts = env::var("OMTRS_DISCOVERY_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    let initial_delay_ms = env::var("OMTRS_DISCOVERY_INITIAL_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(200);
    let max_delay_ms = env::var("OMTRS_DISCOVERY_MAX_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(initial_delay_ms);
    let backoff = env::var("OMTRS_DISCOVERY_BACKOFF")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);

    // Use the iterator-based discovery helper to collect sender addresses.
    let addresses: Vec<Address> = Discovery::addresses_with_backoff(
        attempts,
        Timeout::from_millis(initial_delay_ms).as_duration(),
        Timeout::from_millis(max_delay_ms).as_duration(),
        backoff,
    )
    .collect();

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
                if let Some(video) = frame.video() {
                    let codec = fourcc_to_string(frame.codec().fourcc());
                    let flags = describe_video_flags(video.flags());
                    let (fr_n, fr_d) = video.frame_rate();

                    info!(
                        "  -> Video: {}x{} @ {}/{} fps, codec {}, flags [{}], colorspace {:?}",
                        video.width(),
                        video.height(),
                        fr_n,
                        fr_d,
                        codec,
                        flags,
                        video.color_space()
                    );
                } else {
                    info!("  -> No video frame received (non-video).");
                }
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
        "None".to_string()
    } else {
        parts.join(", ")
    }
}
