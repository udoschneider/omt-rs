//! End-to-end loopback test over the real native libomt.
//!
//! Creates an OMT sender, connects receivers to it on `127.0.0.1`, and verifies
//! that video and metadata frames survive a full send/receive round trip with
//! their payload intact. Exercises the safe `receive(&mut self)` API exactly as
//! applications use it.
//!
//! The sender binds a port chosen by libomt from the configured
//! `NetworkPortStart..NetworkPortEnd` range, and there is no API to query the
//! bound port directly. The test therefore reserves a narrow, unlikely-to-
//! collide range, creates one candidate receiver per port, and polls them all;
//! the candidate that delivers frames is the sender's actual port. No mDNS /
//! discovery infrastructure is involved, so this runs anywhere.

use omt::{
    Codec, FrameType, MetadataFrameBuilder, PreferredVideoFormat, Quality, ReceiveFlags, Receiver,
    Sender, Settings, VideoFrameBuilder,
};

const WIDTH: i32 = 64;
const HEIGHT: i32 = 48;

/// Narrow dedicated port range so other local OMT traffic cannot collide.
const PORT_START: i32 = 27400;
const PORT_END: i32 = 27404;

/// Polling budget: `ROUNDS` rounds x `PORTS` candidates x `RECV_TIMEOUT_MS`.
const ROUNDS: usize = 30;
const RECV_TIMEOUT_MS: i32 = 200;

/// Dark/light gray levels inside studio-swing range.
const DARK: u8 = 16;
const LIGHT: u8 = 235;
/// Slack for the sender's lossy RGB -> YUV -> RGB round trip (grays survive
/// nearly unchanged, but clamping and rounding are observable).
const TOLERANCE: i32 = 20;

/// Builds a BGRA test frame: dark left half, light right half. Gray pixels
/// survive the protocol's YUV transcoding, while the step edge still detects
/// misrouted or truncated payloads.
fn test_video_frame() -> omt::OwnedMediaFrame {
    let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    for (i, px) in data.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let level = if i % (WIDTH as usize) < (WIDTH as usize) / 2 {
            DARK
        } else {
            LIGHT
        };
        px[0] = level;
        px[1] = level;
        px[2] = level;
        px[3] = 255;
    }

    VideoFrameBuilder::new()
        .codec(Codec::Bgra)
        .dimensions(WIDTH, HEIGHT)
        .data(data)
        .build()
        .expect("valid test video frame")
}

/// Asserts the received buffer matches the two-tone pattern within [`TOLERANCE`].
fn assert_two_tone(payload: &[u8], context: &str) {
    assert_eq!(
        payload.len(),
        (WIDTH * HEIGHT * 4) as usize,
        "{context}: unexpected payload size"
    );
    for (i, px) in payload.as_chunks::<4>().0.iter().enumerate() {
        let level = if i % (WIDTH as usize) < (WIDTH as usize) / 2 {
            DARK
        } else {
            LIGHT
        };
        // BGRA may arrive as BGRX ("BGRA treated as BGRX" where alpha flags are
        // unset), so compare the three color channels order-insensitively.
        for (channel_index, &channel) in px[..3].iter().enumerate() {
            assert!(
                (channel as i32 - level as i32).abs() <= TOLERANCE,
                "{context}: pixel {i} channel {channel_index} deviates from {level}: got {px:?}"
            );
        }
    }
}

#[test]
fn loopback_video_and_metadata_roundtrip() {
    Settings::set_network_port_start(PORT_START);
    Settings::set_network_port_end(PORT_END);

    let sender =
        Sender::new("omt-rs loopback test", Quality::Default).expect("failed to create sender");

    // Regression guard for the string-getter NUL trimming (the C API reports
    // lengths including the terminator): a live address must contain none.
    let address = sender.get_address().expect("sender address");
    assert!(!address.contains('\0'), "address contains NUL: {address:?}");

    let frame_types = FrameType::VIDEO | FrameType::METADATA;
    let mut candidates: Vec<Receiver> = (PORT_START..=PORT_END)
        .map(|port| {
            Receiver::new(
                &format!("omt://127.0.0.1:{port}"),
                frame_types,
                PreferredVideoFormat::Bgra,
                ReceiveFlags::NONE,
            )
            .expect("failed to create candidate receiver")
        })
        .collect();

    // -- Video round trip ---------------------------------------------------
    {
        let sent = test_video_frame();
        for round in 0..ROUNDS {
            sender
                .send(&sent.as_media_frame())
                .expect("failed to send video frame");

            let mut received = None;
            for receiver in candidates.iter_mut() {
                match receiver.receive(FrameType::VIDEO, RECV_TIMEOUT_MS) {
                    Ok(Some(frame)) => {
                        received = Some(frame);
                        break;
                    }
                    Ok(None) => {} // not this port / not yet connected
                    Err(err) => panic!("receive failed: {err}"),
                }
            }

            if let Some(frame) = received {
                assert_eq!(frame.frame_type(), FrameType::VIDEO);
                assert_eq!(frame.codec(), Some(Codec::Bgra));
                assert_eq!(frame.width(), WIDTH);
                assert_eq!(frame.height(), HEIGHT);
                assert_two_tone(frame.data(), "video payload");
                break;
            }

            assert!(
                round + 1 < ROUNDS,
                "no video frame received after {ROUNDS} polling rounds"
            );
        }
    }

    // -- Metadata round trip ------------------------------------------------
    {
        let metadata = "<omt-rs-loopback>roundtrip</omt-rs-loopback>".to_string();
        let sent = MetadataFrameBuilder::new()
            .metadata(metadata.clone())
            .build()
            .expect("valid test metadata frame");

        for round in 0..ROUNDS {
            sender
                .send(&sent.as_media_frame())
                .expect("failed to send metadata frame");

            let mut received = None;
            for receiver in candidates.iter_mut() {
                match receiver.receive(FrameType::METADATA, RECV_TIMEOUT_MS) {
                    Ok(Some(frame)) => {
                        received = Some(frame);
                        break;
                    }
                    Ok(None) => {}
                    Err(err) => panic!("receive failed: {err}"),
                }
            }

            if let Some(frame) = received {
                assert_eq!(frame.frame_type(), FrameType::METADATA);
                // libomt re-terminates metadata payloads, so compare the
                // content as a prefix and require the tail to be pure padding.
                assert!(
                    frame.data().starts_with(metadata.as_bytes()),
                    "metadata payload corrupted in transit"
                );
                assert!(
                    frame.data()[metadata.len()..].iter().all(|&b| b == 0),
                    "metadata padding contains non-NUL bytes"
                );
                break;
            }

            assert!(
                round + 1 < ROUNDS,
                "no metadata frame received after {ROUNDS} polling rounds"
            );
        }
    }
}
