use libomt::{
    settings_get_string, settings_set_string, Codec, ColorSpace, Discovery, FrameType,
    OutgoingFrame, PreferredVideoFormat, Quality, ReceiveFlags, Receiver, Sender, Timeout,
    VideoFlags,
};
use std::env;
use std::time::Duration;

fn main() {
    if let Ok(server) = env::var("LIBOMT_DISCOVERY_SERVER") {
        let server = server.trim().to_string();
        if !server.is_empty() {
            if let Err(err) = settings_set_string("DiscoveryServer", &server) {
                eprintln!("Failed to set DiscoveryServer: {}", err);
            }
        }
    }

    let current = settings_get_string("DiscoveryServer").unwrap_or_else(|| "<default>".to_string());
    eprintln!("DiscoveryServer: {}", current);

    let (sender, stream, explicit_address) = parse_cli();

    let address = if let Some(addr) = explicit_address {
        addr
    } else if sender.is_some() || stream.is_some() {
        match discover_matching_sender(sender.as_deref(), stream.as_deref()) {
            Some(addr) => addr,
            None => {
                eprintln!("No matching OMT sender found for --sender/--stream.");
                std::process::exit(1);
            }
        }
    } else {
        match discover_first_sender() {
            Some(addr) => addr,
            None => {
                eprintln!("No OMT senders discovered. Use --sender/--stream or pass an address.");
                std::process::exit(1);
            }
        }
    };

    let rebroadcast_name = match stream.as_deref() {
        Some(name) => format!("{} (BW)", name),
        None => extract_stream_name(&address)
            .map(|name| format!("{} (BW)", name))
            .unwrap_or_else(|| "OMT Stream (BW)".to_string()),
    };

    eprintln!("Connecting to: {}", address);
    eprintln!("Rebroadcast name: {}", rebroadcast_name);

    let mut receiver = match Receiver::create(
        &address,
        FrameType::Video,
        PreferredVideoFormat::UYVYorBGRA,
        ReceiveFlags::NONE,
    ) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Failed to create receiver: {}", err);
            std::process::exit(1);
        }
    };

    let sender = match Sender::create(&rebroadcast_name, Quality::Default) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to create sender: {}", err);
            std::process::exit(1);
        }
    };

    loop {
        for frame_result in receiver.frames(FrameType::Video, Timeout::from_millis(1000)) {
            match frame_result {
                Ok(frame_ref) => {
                    if let Some(video) = frame_ref.video() {
                        let timestamp = frame_ref.timestamp();
                        let (fr_n, fr_d) = video.frame_rate();
                        let aspect_ratio = video.aspect_ratio();
                        let color_space = video.color_space();
                        let flags = video.flags();
                        let stride = video.stride();
                        let width = video.width();
                        let height = video.height();

                        let data = match video.data() {
                            Some(d) => d,
                            None => continue,
                        };

                        let outgoing = match frame_ref.codec() {
                            Codec::BGRA => bw_from_bgra(
                                data,
                                width,
                                height,
                                stride,
                                flags,
                                fr_n,
                                fr_d,
                                aspect_ratio,
                                color_space,
                                timestamp,
                            ),
                            Codec::UYVY => bw_from_uyvy(
                                data,
                                width,
                                height,
                                stride,
                                flags,
                                fr_n,
                                fr_d,
                                aspect_ratio,
                                color_space,
                                timestamp,
                            ),
                            _ => None,
                        };

                        if let Some(mut out) = outgoing {
                            let _ = sender.send(&mut out);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Receive error: {}", err);
                    std::thread::sleep(Duration::from_millis(200));
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn parse_cli() -> (Option<String>, Option<String>, Option<String>) {
    let mut sender: Option<String> = None;
    let mut stream: Option<String> = None;
    let mut address: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sender" => sender = args.next(),
            "--stream" => stream = args.next(),
            _ => {
                if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                } else if address.is_none() {
                    address = Some(arg);
                }
            }
        }
    }

    (sender, stream, address)
}

fn discover_matching_sender(sender: Option<&str>, stream: Option<&str>) -> Option<String> {
    let addresses = discover_addresses();

    let sender_lc = sender.map(|s| s.to_lowercase());
    let stream_lc = stream.map(|s| s.to_lowercase());

    for address in addresses {
        let address_lc = address.to_lowercase();

        if let Some(sender) = sender_lc.as_deref() {
            if !address_lc.starts_with(sender) {
                continue;
            }
        }

        if let Some(stream) = stream_lc.as_deref() {
            let needle = format!("({})", stream);
            if !address_lc.contains(&needle) {
                continue;
            }
        }

        return Some(address);
    }

    None
}

fn discover_addresses() -> Vec<String> {
    let attempts = env::var("LIBOMT_DISCOVERY_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    let initial_delay_ms = env::var("LIBOMT_DISCOVERY_INITIAL_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(200);
    let max_delay_ms = env::var("LIBOMT_DISCOVERY_MAX_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(initial_delay_ms);
    let backoff = env::var("LIBOMT_DISCOVERY_BACKOFF")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);
    let debug = env::var("LIBOMT_DISCOVERY_DEBUG")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Discovery::get_addresses_with_backoff(
        attempts,
        Timeout::from_millis(initial_delay_ms).as_duration(),
        Timeout::from_millis(max_delay_ms).as_duration(),
        backoff,
        debug,
    )
}

fn discover_first_sender() -> Option<String> {
    discover_addresses().into_iter().next()
}

fn extract_stream_name(address: &str) -> Option<String> {
    let start = address.find('(')?;
    let end = address.rfind(')')?;
    if start + 1 >= end {
        return None;
    }
    Some(address[start + 1..end].trim().to_string())
}

fn bw_from_bgra(
    data: &[u8],
    width: i32,
    height: i32,
    stride: i32,
    flags: VideoFlags,
    frame_rate_n: i32,
    frame_rate_d: i32,
    aspect_ratio: f32,
    color_space: ColorSpace,
    timestamp: i64,
) -> Option<OutgoingFrame> {
    let width = width as usize;
    let height = height as usize;
    let stride = stride as usize;

    if width == 0 || height == 0 || stride < width * 4 {
        return None;
    }

    let mut out = vec![0u8; height * stride];

    for y in 0..height {
        let row = &data[y * stride..y * stride + width * 4];
        let out_row = &mut out[y * stride..y * stride + width * 4];

        for x in 0..width {
            let i = x * 4;
            let b = row[i];
            let g = row[i + 1];
            let r = row[i + 2];
            let a = row[i + 3];

            let y_luma = luma_from_rgb(r, g, b);

            out_row[i] = y_luma;
            out_row[i + 1] = y_luma;
            out_row[i + 2] = y_luma;
            out_row[i + 3] = a;
        }
    }

    Some(OutgoingFrame::video(
        Codec::BGRA,
        width as i32,
        height as i32,
        stride as i32,
        flags,
        frame_rate_n,
        frame_rate_d,
        aspect_ratio,
        color_space,
        timestamp,
        out,
    ))
}

fn bw_from_uyvy(
    data: &[u8],
    width: i32,
    height: i32,
    stride: i32,
    flags: VideoFlags,
    frame_rate_n: i32,
    frame_rate_d: i32,
    aspect_ratio: f32,
    color_space: ColorSpace,
    timestamp: i64,
) -> Option<OutgoingFrame> {
    let width = width as usize;
    let height = height as usize;
    let stride = stride as usize;

    if width == 0 || height == 0 || stride < width * 2 {
        return None;
    }

    let mut out = vec![0u8; height * stride];

    for y in 0..height {
        let row = &data[y * stride..y * stride + width * 2];
        let out_row = &mut out[y * stride..y * stride + width * 2];
        out_row.copy_from_slice(row);

        let mut x = 0;
        while x + 1 < width {
            let i = x * 2;
            out_row[i] = 128; // U
            out_row[i + 2] = 128; // V
            x += 2;
        }
    }

    Some(OutgoingFrame::video(
        Codec::UYVY,
        width as i32,
        height as i32,
        stride as i32,
        flags,
        frame_rate_n,
        frame_rate_d,
        aspect_ratio,
        color_space,
        timestamp,
        out,
    ))
}

fn luma_from_rgb(r: u8, g: u8, b: u8) -> u8 {
    let y = (0.299 * r as f32) + (0.587 * g as f32) + (0.114 * b as f32);
    y.round().clamp(0.0, 255.0) as u8
}
