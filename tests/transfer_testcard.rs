use libomt::{
    Address, Codec, ColorSpace, FrameRef, FrameType, OutgoingFrame, PreferredVideoFormat, Quality,
    ReceiveFlags, Receiver, Sender, Source, Timeout, VideoFlags,
};
use std::collections::HashSet;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[test]
fn sender_receiver_transfers_testcard() {
    let source = Source::from(format!("omt-rs-testcard-{}", std::process::id()));
    let sender = Sender::create(&source, Quality::Default).expect("create sender");

    let address: Address = wait_for_sender_address(&sender)
        .unwrap_or_else(|| panic!("sender address not available for '{}'", source));

    let mut receiver = Receiver::create(
        &address,
        FrameType::Video,
        PreferredVideoFormat::UYVYorBGRA,
        ReceiveFlags::NONE,
    )
    .expect("create receiver");

    let (width, height, data, samples) = load_burosch_bgra();

    let mut frame = OutgoingFrame::video(
        Codec::BGRA,
        width,
        height,
        width * 4,
        VideoFlags::NONE,
        30,
        1,
        1.0,
        ColorSpace::BT601,
        -1,
        data,
    );

    let warmup_deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < warmup_deadline {
        sender.send(&mut frame);
        let _ = receiver.receive(FrameType::Video, Timeout::from_millis(100));
        sleep(Duration::from_millis(20));
    }

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut matched = false;

    while Instant::now() < deadline {
        sender.send(&mut frame);

        match receiver.receive(FrameType::Video, Timeout::from_millis(200)) {
            Ok(Some(received)) => {
                if verify_burosch(&received, width, height, &samples) {
                    matched = true;
                    break;
                } else {
                    log_frame_summary(&received, width, height, &samples);
                }
            }
            Ok(None) => {
                println!("No frame received yet");
            }
            Err(err) => {
                println!("Receive error: {}", err);
            }
        }

        sleep(Duration::from_millis(50));
    }

    assert!(
        matched,
        "did not receive the expected testcard image from sender '{}'",
        source
    );
}

fn wait_for_sender_address(sender: &Sender) -> Option<Address> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Some(addr) = sender.get_address() {
            if !addr.as_str().trim().is_empty() {
                return Some(addr);
            }
        }

        sleep(Duration::from_millis(100));
    }
    None
}

struct SamplePoint {
    x: i32,
    y: i32,
    expected: (u8, u8, u8),
}

fn load_burosch_bgra() -> (i32, i32, Vec<u8>, Vec<SamplePoint>) {
    let path = Path::new("examples/Burosch_Blue-only_Test_pattern_mit_erklaerung.jpg");
    let image = image::open(path).expect("load Burosch test image");
    let rgb = image.to_rgb8();
    let (width, height) = rgb.dimensions();

    let samples = select_distinct_samples(&rgb, 16);
    if samples.len() < 16 {
        panic!(
            "need at least 16 distinct sample colors, found {}",
            samples.len()
        );
    }

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

    (width as i32, height as i32, data, samples)
}

fn select_distinct_samples(image: &image::RgbImage, count: usize) -> Vec<SamplePoint> {
    let (width, height) = image.dimensions();
    let step = (width.min(height) / 64).max(1);
    let mut samples = Vec::new();
    let mut seen = HashSet::new();

    let start_x = step;
    let end_x = width.saturating_sub(step);
    let start_y = step;
    let end_y = height.saturating_sub(step);

    let mut y = start_y;
    while y < end_y {
        let mut x = start_x;
        while x < end_x {
            let pixel = image.get_pixel(x, y);
            let color = (pixel[0], pixel[1], pixel[2]);
            if seen.insert(color) {
                samples.push(SamplePoint {
                    x: x as i32,
                    y: y as i32,
                    expected: color,
                });
                if samples.len() >= count {
                    return samples;
                }
            }
            x += step;
        }
        y += step;
    }

    samples
}

fn verify_burosch(frame: &FrameRef, width: i32, height: i32, samples: &[SamplePoint]) -> bool {
    let video = match frame.video() {
        Some(v) => v,
        None => return false,
    };

    if video.width() != width || video.height() != height {
        return false;
    }

    if samples.len() < 16 {
        return false;
    }

    let data = match video.raw_data() {
        Some(d) => d,
        None => return false,
    };

    let tolerance = 40u8;

    match frame.codec() {
        Codec::BGRA => {
            let stride = video.stride() as usize;
            for sample in samples {
                let expected = sample.expected;
                let actual = match bgra_pixel(data, stride, sample.x as usize, sample.y as usize) {
                    Some(v) => v,
                    None => return false,
                };
                if !color_close(actual, expected, tolerance) {
                    return false;
                }
            }
            true
        }
        Codec::UYVY => {
            let stride = video.stride() as usize;
            for sample in samples {
                let expected = sample.expected;
                let actual =
                    match uyvy_pixel_to_rgb(data, stride, sample.x as usize, sample.y as usize) {
                        Some(v) => v,
                        None => return false,
                    };
                if !color_close(actual, expected, tolerance) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn bgra_pixel(data: &[u8], stride: usize, x: usize, y: usize) -> Option<(u8, u8, u8)> {
    let row_start = y.checked_mul(stride)?;
    let idx = row_start.checked_add(x.checked_mul(4)?)?;
    let b = *data.get(idx)?;
    let g = *data.get(idx + 1)?;
    let r = *data.get(idx + 2)?;
    Some((r, g, b))
}

fn uyvy_pixel_to_rgb(data: &[u8], stride: usize, x: usize, y: usize) -> Option<(u8, u8, u8)> {
    let row_start = y.checked_mul(stride)?;
    let pair = x / 2;
    let idx = row_start.checked_add(pair.checked_mul(4)?)?;
    let u = *data.get(idx)? as i32;
    let y0 = *data.get(idx + 1)? as i32;
    let v = *data.get(idx + 2)? as i32;
    let y1 = *data.get(idx + 3)? as i32;

    let y_val = if x % 2 == 0 { y0 } else { y1 };
    let (r, g, b) = yuv_to_rgb_bt601(y_val, u, v);
    Some((r, g, b))
}

fn yuv_to_rgb_bt601(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
    let c = y - 16;
    let d = u - 128;
    let e = v - 128;

    let r = (298 * c + 409 * e + 128) >> 8;
    let g = (298 * c - 100 * d - 208 * e + 128) >> 8;
    let b = (298 * c + 516 * d + 128) >> 8;

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

fn clamp_u8(val: i32) -> u8 {
    if val < 0 {
        0
    } else if val > 255 {
        255
    } else {
        val as u8
    }
}

fn log_frame_summary(
    frame: &FrameRef,
    expected_width: i32,
    expected_height: i32,
    samples: &[SamplePoint],
) {
    if let Some(video) = frame.video() {
        println!(
            "Received frame: codec={:?} size={}x{} stride={} expected={}x{}",
            frame.codec(),
            video.width(),
            video.height(),
            video.stride(),
            expected_width,
            expected_height
        );

        match video.raw_data() {
            Some(data) => {
                for sample in samples {
                    let expected = sample.expected;
                    let x = sample.x;
                    let y = sample.y;
                    match frame.codec() {
                        Codec::BGRA => {
                            if let Some(actual) =
                                bgra_pixel(data, video.stride() as usize, x as usize, y as usize)
                            {
                                println!(
                                    "Point ({},{}): actual={:?} expected={:?}",
                                    x, y, actual, expected
                                );
                            } else {
                                println!("Point ({},{}): unavailable BGRA data", x, y);
                            }
                        }
                        Codec::UYVY => {
                            if let Some(actual) = uyvy_pixel_to_rgb(
                                data,
                                video.stride() as usize,
                                x as usize,
                                y as usize,
                            ) {
                                println!(
                                    "Point ({},{}): actual={:?} expected={:?}",
                                    x, y, actual, expected
                                );
                            } else {
                                println!("Point ({},{}): unavailable UYVY data", x, y);
                            }
                        }
                        other => {
                            println!("Unsupported codec for debug output: {:?}", other);
                        }
                    }
                }
            }
            None => {
                println!("Received video frame with no data payload");
            }
        }
    } else {
        println!("Received non-video frame: codec={:?}", frame.codec());
    }
}

fn color_close(a: (u8, u8, u8), b: (u8, u8, u8), tol: u8) -> bool {
    let dr = a.0.abs_diff(b.0);
    let dg = a.1.abs_diff(b.1);
    let db = a.2.abs_diff(b.2);
    dr <= tol && dg <= tol && db <= tol
}
