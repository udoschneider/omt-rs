use log::{error, info};
use omt::{
    helpers::{discover_first_sender, discover_matching_sender, parse_cli},
    Codec, ColorSpace, FrameType, OutgoingFrame, PreferredVideoFormat, Quality, ReceiveFlags,
    Receiver, Sender, Source, Timeout, VideoDataFormat, VideoFlags,
};

use std::time::Duration;

fn main() {
    env_logger::init();

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

    let rebroadcast_name = match stream.as_deref() {
        Some(name) => format!("{} (BW)", name),
        None => extract_stream_name(address.as_str())
            .map(|name| format!("{} (BW)", name))
            .unwrap_or_else(|| "OMT Stream (BW)".to_string()),
    };

    info!("Connecting to: {}", address);
    info!("Rebroadcast name: {}", rebroadcast_name);

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

    let source = Source::from(rebroadcast_name.clone());
    let sender = match Sender::create(&source, Quality::Default) {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to create sender: {}", err);
            std::process::exit(1);
        }
    };

    loop {
        match receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
            Ok(Some(frame_ref)) => {
                if let Some(video) = frame_ref.video() {
                    let timestamp = frame_ref.timestamp();
                    let (fr_n, fr_d) = video.frame_rate();
                    let aspect_ratio = video.aspect_ratio();
                    let color_space = video.color_space();
                    let flags = video.flags();

                    let width = video.width();
                    let height = video.height();

                    let data = match video.data(VideoDataFormat::RGB) {
                        Some(d) => d,
                        None => {
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                    };

                    let outgoing = bw_from_rgb(
                        &data,
                        width,
                        height,
                        flags,
                        fr_n,
                        fr_d,
                        aspect_ratio,
                        color_space,
                        timestamp,
                    );

                    if let Some(mut out) = outgoing {
                        let _ = sender.send(&mut out);
                    }
                }
            }
            Ok(None) => {
                // Timeout: no frame this cycle.
            }
            Err(err) => {
                error!("Receive error: {}", err);
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn extract_stream_name(address: &str) -> Option<String> {
    let start = address.find('(')?;
    let end = address.rfind(')')?;
    if start + 1 >= end {
        return None;
    }
    Some(address[start + 1..end].trim().to_string())
}

fn bw_from_rgb(
    data: &[u8],
    width: i32,
    height: i32,
    flags: VideoFlags,
    frame_rate_n: i32,
    frame_rate_d: i32,
    aspect_ratio: f32,
    color_space: ColorSpace,
    timestamp: i64,
) -> Option<OutgoingFrame> {
    let width = width as usize;
    let height = height as usize;

    if width == 0 || height == 0 || data.len() < width * height * 3 {
        return None;
    }

    let stride = width * 4;
    let mut out = vec![0u8; height * stride];

    let mut out_flags = flags;
    out_flags.remove(VideoFlags::ALPHA | VideoFlags::PREMULTIPLIED | VideoFlags::HIGH_BIT_DEPTH);

    for y in 0..height {
        let row = &data[y * width * 3..y * width * 3 + width * 3];
        let out_row = &mut out[y * stride..y * stride + width * 4];

        for x in 0..width {
            let i = x * 3;
            let r = row[i];
            let g = row[i + 1];
            let b = row[i + 2];

            let y_luma = luma_from_rgb(r, g, b);

            let o = x * 4;
            out_row[o] = y_luma;
            out_row[o + 1] = y_luma;
            out_row[o + 2] = y_luma;
            out_row[o + 3] = 255;
        }
    }

    Some(OutgoingFrame::video(
        Codec::BGRA,
        width as i32,
        height as i32,
        stride as i32,
        out_flags,
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
