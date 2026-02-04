use libomt::{
    helpers::parse_cli, settings_get_string, settings_set_string, Address, Codec, ColorSpace,
    Discovery, FrameRef, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver, Timeout,
    VideoFrame,
};
use log::{error, info};
use std::env;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    if let Ok(server) = env::var("LIBOMT_DISCOVERY_SERVER") {
        let server = server.trim().to_string();
        if !server.is_empty() {
            if let Err(err) = settings_set_string("DiscoveryServer", &server) {
                error!("Failed to set DiscoveryServer: {}", err);
            }
        }
    }

    let current = settings_get_string("DiscoveryServer").unwrap_or_else(|| "<default>".to_string());
    info!("DiscoveryServer: {}", current);

    let (sender, stream, explicit_address) = parse_cli();

    let address = if let Some(addr) = explicit_address {
        addr
    } else if sender.is_some() || stream.is_some() {
        match discover_matching_sender(sender.as_deref(), stream.as_deref()) {
            Some(addr) => addr,
            None => {
                error!("No matching OMT sender found for --sender/--stream.");
                std::process::exit(1);
            }
        }
    } else {
        match discover_first_sender() {
            Some(addr) => addr,
            None => {
                error!("No OMT senders discovered. Use --sender/--stream or pass an address.");
                std::process::exit(1);
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

    let fps = env::var("LIBOMT_VIEW_FPS")
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

fn discover_matching_sender(sender: Option<&str>, stream: Option<&str>) -> Option<Address> {
    let addresses = discover_addresses();

    let sender_lc = sender.map(|s| s.to_lowercase());
    let stream_lc = stream.map(|s| s.to_lowercase());

    for address in addresses {
        let address_lc = address.as_str().to_lowercase();

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

fn discover_addresses() -> Vec<Address> {
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

    Discovery::get_addresses_with_backoff(
        attempts,
        Timeout::from_millis(initial_delay_ms).as_duration(),
        Timeout::from_millis(max_delay_ms).as_duration(),
        backoff,
    )
}

fn discover_first_sender() -> Option<Address> {
    discover_addresses().into_iter().next()
}

fn frame_to_image(frame: &FrameRef) -> Option<image::DynamicImage> {
    let video = frame.video()?;
    let data = video.raw_data()?;

    match frame.codec() {
        Codec::BGRA => bgra_to_image(&video, data),
        Codec::UYVY => uyvy_to_image(&video, data),
        _ => None,
    }
}

fn bgra_to_image(video: &VideoFrame, data: &[u8]) -> Option<image::DynamicImage> {
    let width = video.width() as usize;
    let height = video.height() as usize;
    let stride = video.stride() as usize;

    if width == 0 || height == 0 || stride < width * 4 {
        return None;
    }

    let mut rgb = vec![0u8; width * height * 3];

    for y in 0..height {
        let row = &data[y * stride..y * stride + width * 4];
        for x in 0..width {
            let i = x * 4;
            let b = row[i];
            let g = row[i + 1];
            let r = row[i + 2];
            let out = (y * width + x) * 3;
            rgb[out] = r;
            rgb[out + 1] = g;
            rgb[out + 2] = b;
        }
    }

    let image = image::RgbImage::from_raw(width as u32, height as u32, rgb)?;
    Some(image::DynamicImage::ImageRgb8(image))
}

fn uyvy_to_image(video: &VideoFrame, data: &[u8]) -> Option<image::DynamicImage> {
    let width = video.width() as usize;
    let height = video.height() as usize;
    let stride = video.stride() as usize;

    if width == 0 || height == 0 || stride < width * 2 {
        return None;
    }

    let mut rgb = vec![0u8; width * height * 3];

    for y in 0..height {
        let row = &data[y * stride..y * stride + width * 2];
        let mut x = 0;
        while x + 1 < width {
            let i = x * 2;
            let u = row[i];
            let y0 = row[i + 1];
            let v = row[i + 2];
            let y1 = row[i + 3];

            let (r0, g0, b0) = yuv_to_rgb(y0, u, v, video.color_space());
            let (r1, g1, b1) = yuv_to_rgb(y1, u, v, video.color_space());

            let out0 = (y * width + x) * 3;
            rgb[out0] = r0;
            rgb[out0 + 1] = g0;
            rgb[out0 + 2] = b0;

            let out1 = (y * width + x + 1) * 3;
            rgb[out1] = r1;
            rgb[out1 + 1] = g1;
            rgb[out1 + 2] = b1;

            x += 2;
        }
    }

    let image = image::RgbImage::from_raw(width as u32, height as u32, rgb)?;
    Some(image::DynamicImage::ImageRgb8(image))
}

fn yuv_to_rgb(y: u8, u: u8, v: u8, _cs: ColorSpace) -> (u8, u8, u8) {
    let c = y as i32 - 16;
    let d = u as i32 - 128;
    let e = v as i32 - 128;

    let r = (298 * c + 409 * e + 128) / 256;
    let g = (298 * c - 100 * d - 208 * e + 128) / 256;
    let b = (298 * c + 516 * d + 128) / 256;

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

fn clamp_u8(v: i32) -> u8 {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v as u8
    }
}
