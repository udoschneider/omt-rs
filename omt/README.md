# omt

High-level, safe, and idiomatic Rust bindings for the [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt) library.

Part of the [Open Media Transport](https://github.com/openmediatransport) project.

[![Crates.io](https://img.shields.io/crates/v/omt.svg)](https://crates.io/crates/omt)
[![Documentation](https://docs.rs/omt/badge.svg)](https://docs.rs/omt)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

OMT is a protocol for low-latency transmission of video, audio, and metadata over IP networks. This crate provides safe, ergonomic Rust wrappers around the low-level C bindings in the `omt-sys` crate.

### Features

- 🦀 **Type-safe**: Strongly-typed enums and structs for media types, codecs, and flags
- 🔒 **Memory-safe**: RAII-based sender and receiver types with automatic resource cleanup
- ⚡ **Zero-copy**: Direct access to frame data without unnecessary copies where possible
- 🌐 **Network discovery**: Automatic discovery of available sources on the network
- 📊 **Statistics**: Built-in performance monitoring and metrics
- 🎨 **Multiple codecs**: Support for various video formats (UYVY, BGRA, VMX1, etc.) and audio (FPA1)
- 🏗️ **Frame builders**: Ergonomic builders for creating video, audio, and metadata frames

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
omt = "0.1"
```

**Note**: This crate requires the OMT C library to be installed on your system. See the [OMT repository](https://github.com/openmediatransport/libomt) for installation instructions.

## Quick Start

### Discovering Sources

```rust
use omt::Discovery;

fn main() {
    let sources = Discovery::get_addresses();
    for source in sources {
        println!("Found: {}", source);
    }
}
```

### Receiving Media

```rust
use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create receiver
    let receiver = Receiver::new(
        "omt://hostname:6400",
        FrameType::Video | FrameType::Audio,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::NONE,
    )?;

    // Receive video frames
    while let Some(frame) = receiver.receive_video(1000)? {
        println!("Video: {}x{} @ {:.2} fps", 
            frame.width(), 
            frame.height(), 
            frame.frame_rate()
        );
    }

    Ok(())
}
```

### Sending Media

```rust
use omt::{Sender, Quality, SenderInfo, VideoFrameBuilder, Codec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sender
    let sender = Sender::new("My Camera", Quality::High)?;

    // Set sender information
    let info = SenderInfo::new(
        "My Application".to_string(),
        "My Company".to_string(),
        "1.0.0".to_string(),
    );
    sender.set_sender_information(&info)?;

    // Create and send a video frame
    let data = vec![0u8; 1920 * 1080 * 2]; // UYVY: 2 bytes per pixel
    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(1920, 1080)
        .frame_rate(30, 1)
        .aspect_ratio(16.0 / 9.0)
        .data(data)
        .build()?;
    
    sender.send(&frame.as_media_frame())?;

    Ok(())
}
```

See the [Frame Builders Guide](../docs/FRAME_BUILDERS.md) for detailed information on creating video, audio, and metadata frames.

## Core Types

### Frame Types

- **`VideoFrame`**: Video frames with dimensions, frame rate, codec info
- **`AudioFrame`**: Audio frames with sample rate, channels, planar f32 data
- **`MetadataFrame`**: UTF-8 encoded XML metadata

### Codecs

**Video Codecs**:
- `Vmx1` - Fast proprietary video codec
- `Uyvy` - 16bpp YUV 4:2:2 format
- `Yuy2` - 16bpp YUV 4:2:2 format (YUYV pixel order)
- `Bgra` - 32bpp RGBA format
- `Nv12` - Planar 4:2:0 YUV format
- `Yv12` - Planar 4:2:0 YUV format
- `Uyva` - UYVY with alpha plane
- `P216` - Planar 4:2:2 16-bit YUV
- `Pa16` - P216 with 16-bit alpha plane

**Audio Codecs**:
- `Fpa1` - 32-bit floating-point planar audio

### Quality Levels

- `Quality::Default` - Allow receiver suggestions
- `Quality::Low` - Low quality encoding
- `Quality::Medium` - Medium quality encoding
- `Quality::High` - High quality encoding

### Receive Flags

- `ReceiveFlags::NONE` - Standard reception
- `ReceiveFlags::PREVIEW` - Receive 1/8th preview frames only
- `ReceiveFlags::INCLUDE_COMPRESSED` - Include compressed VMX1 data
- `ReceiveFlags::COMPRESSED_ONLY` - Compressed data only, no decoding

## Advanced Features

### Tally Control

```rust
use omt::Tally;

// Set tally state
receiver.set_tally(Tally::program_only());

// Get tally state
let (tally, changed) = sender.get_tally(1000)?;
if changed {
    println!("Tally: {}", tally);
}
```

### Statistics

```rust
let video_stats = receiver.get_video_statistics();
let audio_stats = receiver.get_audio_statistics();

println!("Frames: {}, Dropped: {}", 
    video_stats.frames, 
    video_stats.frames_dropped
);

if let Some(avg_ms) = video_stats.average_codec_time_ms() {
    println!("Avg codec time: {:.2}ms", avg_ms);
}
```

### Settings

```rust
use omt::Settings;

// Configure discovery server
Settings::set_discovery_server("omt://server:6400")?;

// Configure port range
Settings::set_network_port_start(7000);
Settings::set_network_port_end(7200);
```

### Logging

```rust
use omt::set_logging_filename;

// Enable logging to file
set_logging_filename(Some("/var/log/omt.log"));

// Disable logging
set_logging_filename(None);
```

## Examples

See the [`examples/`](examples/) directory for complete working examples:

- **`discovery.rs`**: Network discovery of OMT sources
- **`receiver.rs`**: Receiving and displaying frame information
- **`sender.rs`**: Creating a sender and monitoring connections

Run an example:

```bash
cargo run --example discovery
cargo run --example receiver
cargo run --example sender
```

## Architecture

```
┌─────────────────────────────────────┐
│         Your Application            │
│  (Rust code using omt crate)        │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│          omt crate                  │
│  (Safe, high-level Rust API)       │
│  - Sender, Receiver                 │
│  - VideoFrame, AudioFrame           │
│  - Discovery, Settings              │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│         omt-sys crate               │
│  (Low-level FFI bindings)           │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│       libomt (C library)            │
│  (Native OMT implementation)        │
└─────────────────────────────────────┘
```

## Thread Safety

Both `Sender` and `Receiver` are `Send + Sync`, allowing safe use across threads:

```rust
use std::thread;

let receiver = Receiver::new(/* ... */)?;
let receiver_ref = &receiver;

let video_thread = thread::spawn(move || {
    while let Some(frame) = receiver_ref.receive_video(1000).unwrap() {
        // Process video
    }
});

let audio_thread = thread::spawn(move || {
    while let Some(frame) = receiver_ref.receive_audio(1000).unwrap() {
        // Process audio
    }
});
```

## Color Spaces

OMT supports automatic color space detection or manual specification:

- `ColorSpace::Undefined` - Automatic (BT.601 for SD, BT.709 for HD)
- `ColorSpace::Bt601` - ITU-R BT.601 (standard definition)
- `ColorSpace::Bt709` - ITU-R BT.709 (high definition)

## Video Flags

Video frames can have various flags set:

- `VideoFlags::INTERLACED` - Interlaced video
- `VideoFlags::ALPHA` - Contains alpha channel
- `VideoFlags::PRE_MULTIPLIED` - Premultiplied alpha
- `VideoFlags::PREVIEW` - Preview frame (1/8th size)
- `VideoFlags::HIGH_BIT_DEPTH` - High bit depth (P216/PA16)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [Open Media Transport](https://github.com/openmediatransport) - The overall OMT project
- [libomt](https://github.com/openmediatransport/libomt) - The underlying C implementation
- [omt-sys](../omt-sys) - Low-level FFI bindings for Rust

## Support

For issues specific to these Rust bindings, please open an issue on this repository. For questions about the OMT protocol or C library, refer to the [openmediatransport organization](https://github.com/openmediatransport) or the [libomt repository](https://github.com/openmediatransport/libomt).