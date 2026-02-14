# omt

High-level, safe, and idiomatic Rust bindings for the [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt) library.

**Note:** This is an **unofficial, third-party** Rust wrapper. It is not affiliated with or endorsed by the Open Media Transport project.

[![Crates.io](https://img.shields.io/crates/v/omt.svg)](https://crates.io/crates/omt)
[![Documentation](https://docs.rs/omt/badge.svg)](https://docs.rs/omt)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Safety](https://img.shields.io/badge/safety-documented-blue.svg)](docs/unsafe-areas.md)

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
- 📖 **Safety documentation**: Comprehensive documentation of unsafe areas and safety guarantees

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
    let mut receiver = Receiver::new(
        "omt://hostname:6400",
        FrameType::VIDEO | FrameType::AUDIO,
        PreferredVideoFormat::Uyvy,
        ReceiveFlags::NONE,
    )?;

    // Receive video frames (using safe API)
    loop {
        if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
            println!("Video: {}x{} @ {:.2} fps", 
                frame.width(), 
                frame.height(), 
                frame.frame_rate()
            );
        }
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

## Frame Lifetime and Safety

### Safe API (Recommended)

The recommended way to receive frames uses `receive()`, which requires mutable access:

```rust
let mut receiver = Receiver::new(...)?;

loop {
    if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
        // Process frame immediately
        process_frame(&frame);
        // Frame automatically dropped here
    }
}
```

**Why this is safe:** The borrow checker prevents holding multiple frames simultaneously, eliminating use-after-free bugs at compile time.

### Unsafe API (Advanced)

For performance-critical scenarios requiring concurrent access to statistics:

```rust
let receiver = Arc::new(Receiver::new(...)?);

unsafe {
    if let Some(frame) = receiver.receive_unchecked(FrameType::VIDEO, 1000)? {
        process_frame(&frame);
        // CRITICAL: Frame MUST be dropped before next receive_unchecked()
    }
}
```

**Safety requirements:** You must ensure no previous frame is still alive when calling `receive_unchecked()` again. Violating this causes undefined behavior.

**Storing frames:** If you need to keep frames beyond the next receive call, you can clone them:

```rust
let receiver = Arc::new(Receiver::new(...)?);
let mut stored_frames = Vec::new();

for _ in 0..10 {
    unsafe {
        if let Some(frame) = receiver.receive_unchecked(FrameType::VIDEO, 1000)? {
            // Clone creates a deep copy of all frame data - safe to store
            stored_frames.push(frame.clone());
        }
    }
}
```

**Warning:** Cloning performs a deep copy of all frame data (potentially ~64MB for 4K 16-bit RGBA). Use sparingly and only when necessary.

### The Problem

The underlying C library reuses internal frame buffers. When you call receive again, the previous frame's data becomes invalid. Rust's type system cannot express "this method invalidates results from this specific other method," so we provide:

1. **Safe API (`receive`)**: Uses `&mut self` - the borrow checker enforces safety
2. **Unsafe API (`receive_unchecked`)**: Uses `&self` - you must manually ensure safety

**Default to the safe API.** Only use the unsafe API if you've profiled and confirmed that `Mutex` overhead is a bottleneck.

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
use omt::Settings;

// Enable logging to file
Settings::set_logging_filename(Some("/var/log/omt.log"));

// Disable logging
Settings::set_logging_filename(None);
```

## Examples

See the [`examples/`](examples/) directory for complete working examples:

### Basic Examples

#### `discovery` - Network Discovery
Continuously scans the network for available OMT sources and displays their addresses.

```bash
cargo run --example discovery
```

**Features:**
- Automatic network scanning using mDNS
- Continuous refresh every 5 seconds
- Displays source count and addresses

#### `sender` - Creating a Sender
Creates an OMT sender that broadcasts on the network and monitors for connections.

```bash
cargo run --example sender
```

**Features:**
- Creates a high-quality OMT sender
- Sets sender information metadata
- Monitors active connections
- Polls for tally state changes

**Note:** This example creates a sender but does not send frames. See `send_frames` for complete transmission.

#### `receiver` - Receiving Frames
Discovers sources and receives both video and audio frames for 10 seconds.

```bash
cargo run --example receiver
```

**Features:**
- Automatic network discovery
- Receives video and audio simultaneously
- Displays frame information (dimensions, codec, sample rate)
- Shows transmission statistics

### Advanced Examples

#### `send_frames` - Complete Frame Transmission
Loads a JPEG image and sends it as a video stream at 30fps with a 1kHz sine wave audio signal.

```bash
cargo run --example send_frames
```

**Features:**
- Loads `testcard.jpg` and converts to BGRA format
- Generates continuous 1kHz sine wave audio (48kHz)
- Sends metadata frames with stream information
- Displays connection count and statistics

**Requirements:** Image file `testcard.jpg` in the examples directory

#### `view_stream` - Terminal Video Viewer
Displays an OMT video stream directly in the terminal with true color support.

```bash
# Auto-discover first source
cargo run --example view_stream

# Or specify address
cargo run --example view_stream -- "omt://hostname:6400"
```

**Features:**
- Receives UYVY video and converts to RGB
- Renders frames in terminal with true color
- Throttles display to ~1 fps for readability

**Requirements:** Terminal with true color support (24-bit color)

#### `rebroadcast_bw` - Video Processing Pipeline
Receives an OMT stream, converts it to grayscale, and rebroadcasts as a new stream.

```bash
# Auto-discover first source
cargo run --example rebroadcast_bw

# Or specify address
cargo run --example rebroadcast_bw -- "omt://hostname:6400"
```

**Features:**
- Receives UYVY video frames
- Converts to grayscale by neutralizing chrominance
- Rebroadcasts as new stream with " (BW)" suffix
- Preserves frame rate, aspect ratio, and timing

**How it works:** Sets U and V components to 128 (neutral) while preserving Y (luma) values.

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

Both `Sender` and `Receiver` are `Send + Sync`, allowing safe use across threads.

### Option 1: Separate Instances (Recommended)

The simplest approach is to create separate receiver instances per thread:

```rust
use std::thread;
use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};

// Create separate receivers for each thread
let mut video_receiver = Receiver::new("omt://localhost:6400", 
    FrameType::VIDEO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;
let mut audio_receiver = Receiver::new("omt://localhost:6400",
    FrameType::AUDIO, PreferredVideoFormat::Uyvy, ReceiveFlags::NONE)?;

let video_thread = thread::spawn(move || {
    loop {
        if let Some(frame) = video_receiver.receive(FrameType::VIDEO, 1000).unwrap() {
            // Process video
        }
    }
});

let audio_thread = thread::spawn(move || {
    loop {
        if let Some(frame) = audio_receiver.receive(FrameType::AUDIO, 1000).unwrap() {
            // Process audio
        }
    }
});
```

### Option 2: Shared Instance with Mutex

If you need to share a single receiver instance:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let receiver = Arc::new(Mutex::new(Receiver::new(/* ... */)?));

let r1 = receiver.clone();
let video_thread = thread::spawn(move || {
    loop {
        let mut guard = r1.lock().unwrap();
        if let Some(frame) = guard.receive(FrameType::VIDEO, 1000).unwrap() {
            // Process video frame
        } // Lock released when guard is dropped
    }
});

let r2 = receiver.clone();
let stats_thread = thread::spawn(move || {
    loop {
        thread::sleep(Duration::from_secs(1));
        let guard = r2.lock().unwrap();
        let stats = guard.get_video_statistics();
        println!("Stats: {}", stats);
    }
});
```

### Option 3: Unsafe API for Lock-Free Sharing

For advanced use cases requiring concurrent statistics access without locks:

```rust
use std::sync::Arc;
use std::thread;

let receiver = Arc::new(Receiver::new(/* ... */)?);

let r1 = receiver.clone();
let video_thread = thread::spawn(move || {
    loop {
        unsafe {
            if let Some(frame) = r1.receive_unchecked(FrameType::VIDEO, 1000).unwrap() {
                // Process video - frame MUST be dropped before next receive_unchecked
            }
        }
    }
});

let r2 = receiver.clone();
let stats_thread = thread::spawn(move || {
    loop {
        thread::sleep(Duration::from_secs(1));
        // Can call concurrently with receive_unchecked (uses &self)
        let stats = r2.get_video_statistics();
        println!("Stats: {}", stats);
    }
});
```

**Note:** Option 3 requires careful adherence to safety requirements. Use Option 1 or 2 unless you've profiled and confirmed lock contention is a bottleneck.

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

## Safety and Performance

### Memory Safety

This crate uses `unsafe` code only where necessary for FFI with the C library. All unsafe operations are:
- Documented with `// SAFETY:` comments explaining the invariants
- Encapsulated behind safe APIs
- Protected by Rust's type system where possible

### Thread Safety

Both `Sender` and `Receiver` implement `Send + Sync`, allowing safe concurrent use:
- Different instances can be used concurrently without synchronization
- Sharing the same instance requires `Arc<Mutex<>>` for the safe API
- Statistics and tally methods use `&self` and can be called concurrently

### Zero-Copy Performance

Both safe and unsafe receive APIs provide zero-copy access to frame data:
- Direct pointers to C library buffers
- No memory copies when accessing frame data
- Suitable for high-bandwidth video (4K @ 60fps)

The safe API has no runtime overhead compared to the unsafe API in single-threaded scenarios.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [Open Media Transport](https://github.com/openmediatransport) - The official OMT project
- [libomt](https://github.com/openmediatransport/libomt) - The official C implementation
- [omt-sys](../omt-sys) - Low-level FFI bindings for Rust (part of this unofficial wrapper)

## Support

**For issues with these Rust bindings:** Please open an issue on this repository.

**For questions about the OMT protocol or official C library:** Refer to the [openmediatransport organization](https://github.com/openmediatransport) or the [libomt repository](https://github.com/openmediatransport/libomt).

**Disclaimer:** This is an unofficial third-party project. For official OMT implementations and support, visit the [Open Media Transport organization](https://github.com/openmediatransport).