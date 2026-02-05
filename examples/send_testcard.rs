use log::{error, info};
use omt::{
    Codec, ColorSpace, Name, OutgoingFrame, Quality, Sender, SenderInfo, Timeout, VideoFlags,
};
use std::env;
use std::f32::consts::TAU;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    let name = Name::new(format!("omt-rs-testcard-{}", std::process::id()));
    let mut sender = Sender::create(&name, Quality::Default).expect("create sender");
    let info = SenderInfo {
        product_name: "omt-rs".to_string(),
        manufacturer: "omt-rs examples".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        reserved1: String::new(),
        reserved2: String::new(),
        reserved3: String::new(),
    };
    sender.set_sender_info(&info);
    let _ = sender.add_connection_metadata("<omt><source>burosch-testcard</source></omt>");

    let (width, height, data) = load_burosch_bgra();

    let mut frame = OutgoingFrame::video(
        Codec::BGRA,
        width,
        height,
        width * 4,
        VideoFlags::NONE,
        30,
        1,
        1.0,
        ColorSpace::Undefined,
        -1,
        data,
    );

    let audio_sample_rate = 48_000;
    let audio_channels = 2;
    let audio_frequency_hz = 1_000.0f32;
    let audio_samples_per_frame = audio_sample_rate / 30;
    let mut audio_phase = 0.0f32;

    if let Some(address) = sender.get_address() {
        info!("Sender address: {}", address);
    }

    let mut last_report = Instant::now();
    let mut sent_frames: u64 = 0;
    let mut last_connections = sender.connections();

    loop {
        sender.send(&mut frame);

        let audio_data = generate_sine_f32_planar(
            audio_sample_rate,
            audio_channels,
            audio_samples_per_frame,
            audio_frequency_hz,
            &mut audio_phase,
        );
        let mut audio_frame = OutgoingFrame::audio(
            Codec::FPA1,
            audio_sample_rate,
            audio_channels,
            audio_samples_per_frame,
            -1,
            audio_data,
        );
        sender.send(&mut audio_frame);

        sent_frames += 1;

        loop {
            match sender.receive_metadata(Timeout::from_millis(0)) {
                Ok(Some(frame_ref)) => {
                    if let Some(payload) = frame_ref.metadata() {
                        let text = String::from_utf8_lossy(payload);
                        let text = text.trim_matches('\0');
                        if text.is_empty() {
                            info!("Metadata: <empty>");
                        } else {
                            info!("Metadata: {}", text);
                        }
                    } else {
                        info!("Metadata frame with no payload");
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    error!("Metadata receive error: {}", err);
                    break;
                }
            }
        }

        let elapsed = last_report.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = sent_frames as f64 / elapsed.as_secs_f64();
            let connections = sender.connections();
            if connections != last_connections {
                info!("Connections: {} (was {})", connections, last_connections);
                last_connections = connections;
            } else {
                info!("Connections: {}", connections);
            }

            let stats = sender.get_video_statistics();
            info!(
                "FPS: {:.2} | bytes_sent={} (+{}) frames={} (+{}) dropped={}",
                fps,
                stats.bytes_sent,
                stats.bytes_sent_since_last,
                stats.frames,
                stats.frames_since_last,
                stats.frames_dropped,
            );

            sent_frames = 0;
            last_report = Instant::now();
        }

        sleep(Duration::from_millis(33));
    }
}

fn load_burosch_bgra() -> (i32, i32, Vec<u8>) {
    let path = Path::new("examples/Burosch_Blue-only_Test_pattern_mit_erklaerung.jpg");
    let image = image::open(path).expect("load Burosch test image");
    let rgb = image.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = b;
            data[idx + 1] = g;
            data[idx + 2] = r;
            data[idx + 3] = 255;
        }
    }

    (width as i32, height as i32, data)
}

fn generate_sine_f32_planar(
    sample_rate: i32,
    channels: i32,
    samples_per_channel: i32,
    frequency_hz: f32,
    phase: &mut f32,
) -> Vec<u8> {
    let channels_usize = channels.max(1) as usize;
    let samples_per_channel_usize = samples_per_channel.max(0) as usize;
    let total_samples = channels_usize * samples_per_channel_usize;

    let phase_inc = TAU * frequency_hz / sample_rate as f32;
    let mut planar = vec![0f32; total_samples];

    for sample_idx in 0..samples_per_channel_usize {
        let value = (*phase).sin();
        *phase += phase_inc;
        if *phase >= TAU {
            *phase -= TAU;
        }

        for ch in 0..channels_usize {
            let idx = ch * samples_per_channel_usize + sample_idx;
            planar[idx] = value;
        }
    }

    let mut bytes = Vec::with_capacity(planar.len() * 4);
    for sample in planar {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}
