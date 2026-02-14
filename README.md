# omt-rs

Unofficial Rust bindings for [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt) - a protocol for low-latency transmission of video, audio, and metadata over IP networks.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Important:** This is an **unofficial, third-party** project. It is not affiliated with or endorsed by the Open Media Transport project or its maintainers.

## Overview

This repository provides unofficial Rust bindings for the Open Media Transport library, organized as a Cargo workspace with two crates:

- **[`omt`](omt/)**: High-level, safe, and idiomatic Rust API (recommended for most users)
- **[`omt-sys`](omt-sys/)**: Low-level FFI bindings to the C library

OMT is designed for professional broadcast and media production workflows where low latency, high quality, and precise timing are critical.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
omt = "0.1"
```

### Simple Example

```rust
use omt::{Discovery, Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover sources on the network
    let sources = Discovery::get_addresses();
    println!("Found {} sources", sources.len());
    
    if let Some(address) = sources.first() {
        // Create a receiver
        let mut receiver = Receiver::new(
            address,
            FrameType::VIDEO | FrameType::AUDIO,
            PreferredVideoFormat::Uyvy,
            ReceiveFlags::NONE,
        )?;
        
        // Receive frames (using safe API)
        loop {
            if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
                println!("Video: {}x{} @ {:.2} fps", 
                    frame.width(), 
                    frame.height(), 
                    frame.frame_rate()
                );
            }
        }
    }
    
    Ok(())
}
```

## Crates

### omt

High-level, safe, and ergonomic Rust bindings for OMT.

**Features:**
- 🦀 **Type-safe**: Strong typing for media types, codecs, and flags
- 🔒 **Memory-safe**: RAII-based resource management with automatic cleanup
- ⚡ **Zero-copy**: Direct access to frame data where possible
- 🌐 **Discovery**: Automatic network discovery of sources
- 📊 **Statistics**: Built-in performance monitoring
- 🎨 **Multiple codecs**: Support for various video and audio formats
- 🏗️ **Frame builders**: Ergonomic API for creating frames

**Documentation:** [omt README](omt/README.md) | [docs.rs](https://docs.rs/omt)

### omt-sys

Low-level FFI bindings generated from the OMT C library headers.

**Features:**
- Raw C API access
- Zero-cost abstraction
- Automatic binding generation via `bindgen`
- Cross-platform support

**⚠️ Note:** Most users should use the high-level `omt` crate instead.

**Documentation:** [omt-sys README](omt-sys/README.md) | [docs.rs](https://docs.rs/omt-sys)

## Prerequisites

The OMT C library must be installed on your system before building:

### Installation

1. **Download libomt**: Get the latest release from [openmediatransport/libomt](https://github.com/openmediatransport/libomt)

2. **Install the library**:
   - **macOS**: Install to `/usr/local/lib` or `/opt/homebrew/lib`
   - **Linux**: Install to `/usr/local/lib` or `/usr/lib`
   - **Windows**: Ensure library is in system PATH

3. **Verify**: Ensure the shared library is accessible:
   ```bash
   # macOS
   ls /usr/local/lib/libomt.dylib
   # or
   ls /opt/homebrew/lib/libomt.dylib
   
   # Linux
   ls /usr/local/lib/libomt.so
   
   # Windows
   where omt.dll
   ```

## Building

```bash
# Clone the repository
git clone https://github.com/yourusername/omt-rs.git
cd omt-rs

# Build all crates
cargo build

# Build with release optimizations
cargo build --release

# Run tests
cargo test

# Build examples
cargo build --examples
```

## Examples

The workspace includes several examples demonstrating different use cases:

### Basic Examples

#### Discovery
Continuously scan the network for available OMT sources:

```bash
cargo run --example discovery
```

#### Receiver
Discover and receive video/audio frames:

```bash
cargo run --example receiver
```

#### Sender
Create an OMT sender and monitor connections:

```bash
cargo run --example sender
```

### Advanced Examples

#### Send Frames
Load an image and transmit as a video stream with audio:

```bash
cargo run --example send_frames
```

**Requirements:** `testcard.jpg` in the examples directory

#### View Stream
Display OMT video stream in terminal with true color:

```bash
# Auto-discover first source
cargo run --example view_stream

# Or specify address
cargo run --example view_stream -- "omt://hostname:6400"
```

**Requirements:** Terminal with 24-bit true color support

#### Rebroadcast (Black & White)
Receive stream, convert to grayscale, and rebroadcast:

```bash
# Auto-discover first source
cargo run --example rebroadcast_bw

# Or specify address
cargo run --example rebroadcast_bw -- "omt://hostname:6400"
```

See the [examples directory](omt/examples/) for complete source code.

## Features

### Supported Codecs

**Video:**
- VMX1 - Fast proprietary video codec
- UYVY - 16bpp YUV 4:2:2
- YUY2 - 16bpp YUV 4:2:2 (YUYV order)
- BGRA - 32bpp RGBA
- NV12 - Planar 4:2:0 YUV
- YV12 - Planar 4:2:0 YUV
- UYVA - UYVY with alpha plane
- P216 - Planar 4:2:2 16-bit YUV
- PA16 - P216 with 16-bit alpha

**Audio:**
- FPA1 - 32-bit floating-point planar audio

### Core Capabilities

- **Network Discovery**: Automatic mDNS-based source discovery
- **Tally Support**: Program/preview tally state management
- **Statistics**: Comprehensive frame and codec performance metrics
- **Color Spaces**: BT.601 and BT.709 support with automatic detection
- **Quality Control**: Configurable encoding quality levels
- **Metadata**: XML-based metadata frames for custom data
- **Thread Safety**: `Send + Sync` implementations for multi-threaded use

### Metadata Support

OMT supports XML-based metadata for:
- Web management interfaces
- PTZ camera control (VISCA over IP and inband)
- Ancillary data (SDI ANC packets)
- Custom application data

See [METADATA.md](omt-sys/docs/METADATA.md) for specifications.

## Architecture

```
┌─────────────────────────────────────┐
│         Your Application            │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│          omt crate                  │
│  (High-level safe Rust API)         │
│  - Sender, Receiver                 │
│  - VideoFrame, AudioFrame           │
│  - Discovery, Settings              │
│  - Frame builders                   │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│         omt-sys crate               │
│  (Low-level FFI bindings)           │
│  - Auto-generated via bindgen       │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│       libomt (C library)            │
│  (Native OMT implementation)        │
└─────────────────────────────────────┘
```

## Documentation

- **[omt crate](omt/README.md)**: High-level API documentation
- **[omt-sys crate](omt-sys/README.md)**: Low-level FFI documentation
- **[Metadata Specification](omt-sys/docs/METADATA.md)**: XML metadata formats
- **[Examples](omt/examples/)**: Working code examples
- **[API Documentation](https://docs.rs/omt)**: Generated API docs

## Project Structure

```
omt-rs/
├── omt/                    # High-level safe Rust bindings
│   ├── src/               # Source code
│   ├── examples/          # Example applications
│   ├── tests/             # Integration tests
│   └── README.md
├── omt-sys/               # Low-level FFI bindings
│   ├── src/
│   │   └── lib.rs         # Generated bindings
│   ├── build.rs           # Bindgen build script
│   ├── libomt.h           # C header file
│   ├── docs/
│   │   └── METADATA.md    # Metadata specification
│   └── README.md
├── Cargo.toml             # Workspace configuration
└── README.md              # This file
```

## Development

### Running Tests

```bash
# Test all crates
cargo test

# Test specific crate
cargo test -p omt
cargo test -p omt-sys

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Check without building
cargo check --all
```

### Building Documentation

```bash
# Build documentation
cargo doc --all --no-deps

# Build and open in browser
cargo doc --all --no-deps --open
```

## Contributing

Contributions are welcome! Please ensure:

- [ ] Code is formatted with `cargo fmt`
- [ ] Code passes `cargo clippy` with no warnings
- [ ] All tests pass with `cargo test`
- [ ] New features include tests
- [ ] Public API is documented
- [ ] No `unwrap()` in production code
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

See [AGENTS.md](AGENTS.md) for detailed development guidelines.

## Version Compatibility

| omt-rs | libomt | Rust |
|--------|--------|------|
| 0.1.x  | latest | 1.70+ |

## Platform Support

- ✅ Linux (x86_64, aarch64)
- ✅ macOS (x86_64, Apple Silicon)
- ✅ Windows (x86_64)

## Performance

OMT is designed for low-latency professional media workflows:

- **Latency**: Typically < 1 frame at 60fps
- **Throughput**: Supports 4K 60fps with VMX1 codec
- **CPU Usage**: Optimized for real-time encoding/decoding
- **Network**: Efficient bandwidth usage with quality control

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- **[Open Media Transport](https://github.com/openmediatransport)**: The official OMT project
- **[libomt](https://github.com/openmediatransport/libomt)**: The official C implementation

## Support

**For issues with these unofficial Rust bindings:** Open an issue on this repository.

**For questions about the official OMT protocol or C library:** See [openmediatransport](https://github.com/openmediatransport) or [libomt](https://github.com/openmediatransport/libomt).

**Disclaimer:** This is an unofficial third-party wrapper. For official OMT implementations and support, visit the [Open Media Transport organization](https://github.com/openmediatransport).

## Acknowledgments

Open Media Transport is developed and maintained by the Open Media Transport Contributors.

