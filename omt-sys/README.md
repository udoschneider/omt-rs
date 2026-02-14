# omt-sys

Low-level FFI bindings for [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt).

**Note:** This is an **unofficial, third-party** Rust wrapper. It is not affiliated with or endorsed by the Open Media Transport project.

[![Crates.io](https://img.shields.io/crates/v/omt-sys.svg)](https://crates.io/crates/omt-sys)
[![Documentation](https://docs.rs/omt-sys/badge.svg)](https://docs.rs/omt-sys)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

This crate provides raw, automatically-generated Rust bindings to the OMT C library using `bindgen`. It exposes the low-level C API directly to Rust code.

**⚠️ Important:** Most users should use the high-level [`omt`](../omt) crate instead, which provides safe, idiomatic Rust wrappers around these bindings. Only use `omt-sys` directly if you need low-level control or are building your own abstractions.

## What is OMT?

Open Media Transport (OMT) is a protocol for low-latency transmission of video, audio, and metadata over IP networks. It's designed for professional broadcast and media production workflows where timing and quality are critical.

## Features

- **Direct C API access**: Raw bindings to all OMT functions
- **Zero-cost abstraction**: No runtime overhead beyond the C library itself
- **Automatic generation**: Bindings are generated from `libomt.h` using `bindgen`
- **Cross-platform**: Supports macOS, Linux, and Windows

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
omt-sys = "0.1"
```

### Prerequisites

The OMT C library must be installed on your system:

1. **Download libomt**: Get the latest release from [openmediatransport/libomt](https://github.com/openmediatransport/libomt)

2. **Install the library**:
   - **macOS**: Library should be installed to `/usr/local/lib` or `/opt/homebrew/lib`
   - **Linux**: Library should be installed to `/usr/local/lib` or `/usr/lib`
   - **Windows**: Library should be available in the system library path

3. **Verify installation**: Ensure `libomt.h` is available and the shared library (`libomt.so`, `libomt.dylib`, or `omt.dll`) is in your system's library path.

## Usage

```rust
use omt_sys::*;

fn main() {
    unsafe {
        // Get available sources on the network
        let mut count: i32 = 0;
        let addresses = OMTDiscovery_GetAddresses(&mut count as *mut i32);
        
        println!("Found {} OMT sources", count);
        
        // Clean up
        OMTDiscovery_FreeAddresses(addresses);
    }
}
```

**Safety**: All functions in this crate are `unsafe` because they interact with C code. You are responsible for:
- Ensuring proper initialization and cleanup
- Managing memory lifetimes
- Validating pointer arguments
- Handling thread safety

## API Structure

The bindings include the following main components:

### Types

- **`OMTSender`**: Opaque sender handle
- **`OMTReceiver`**: Opaque receiver handle
- **`OMTMediaFrame`**: Media frame handle
- **`OMTFrame`**: Frame structure containing pixel/audio data
- **`OMTString`**: OMT string type (fixed-size array)

### Enumerations

- **`OMTFrameType`**: Frame types (None, Metadata, Video, Audio)
- **`OMTCodec`**: Supported codecs (VMX1, UYVY, YUY2, BGRA, NV12, YV12, UYVA, P216, PA16, FPA1)
- **`OMTQuality`**: Encoding quality levels (Default, Low, Medium, High)
- **`OMTColorSpace`**: Color space definitions (Undefined, BT601, BT709)
- **`OMTVideoFlags`**: Video frame flags (Interlaced, Alpha, PreMultiplied, Preview, HighBitDepth)
- **`OMTPreferredVideoFormat`**: Preferred output video format for receivers
- **`OMTReceiveFlags`**: Receiver configuration flags

### Functions

The bindings expose all OMT C API functions, including:

#### Discovery
- `OMTDiscovery_GetAddresses()` - Get available sources
- `OMTDiscovery_FreeAddresses()` - Free address list

#### Sender
- `OMTSender_Create()` - Create sender
- `OMTSender_Destroy()` - Destroy sender
- `OMTSender_Send()` - Send media frame
- `OMTSender_GetTally()` - Get tally state
- `OMTSender_SetSenderInformation()` - Set metadata

#### Receiver
- `OMTReceiver_Create()` - Create receiver
- `OMTReceiver_Destroy()` - Destroy receiver
- `OMTReceiver_ReceiveVideo()` - Receive video frame
- `OMTReceiver_ReceiveAudio()` - Receive audio frame
- `OMTReceiver_ReceiveMetadata()` - Receive metadata frame
- `OMTReceiver_SetTally()` - Set tally state

#### Frame Management
- `OMTMediaFrame_CreateVideo()` - Create video frame
- `OMTMediaFrame_CreateAudio()` - Create audio frame
- `OMTMediaFrame_CreateMetadata()` - Create metadata frame
- `OMTMediaFrame_Destroy()` - Destroy frame
- `OMTMediaFrame_GetType()` - Get frame type

#### Settings
- `OMTSettings_SetDiscoveryServer()` - Configure discovery
- `OMTSettings_SetNetworkPortStart()` - Set port range start
- `OMTSettings_SetNetworkPortEnd()` - Set port range end
- `OMTSettings_SetLoggingFilename()` - Configure logging

## Codec Support

### Video Codecs

- **VMX1**: Fast proprietary video codec
- **UYVY**: 16bpp YUV 4:2:2 format
- **YUY2**: 16bpp YUV 4:2:2 format (YUYV pixel order)
- **BGRA**: 32bpp RGBA format
- **NV12**: Planar 4:2:0 YUV format
- **YV12**: Planar 4:2:0 YUV format
- **UYVA**: UYVY with alpha plane
- **P216**: Planar 4:2:2 16-bit YUV
- **PA16**: P216 with 16-bit alpha plane

### Audio Codecs

- **FPA1**: 32-bit floating-point planar audio

See `libomt.h` for detailed codec specifications.

## Build Process

This crate uses a `build.rs` script that:

1. Searches for the OMT library in standard locations:
   - `/usr/local/lib`
   - `/usr/lib`
   - `/opt/homebrew/lib` (macOS)

2. Generates Rust bindings from `libomt.h` using `bindgen`

3. Links against the OMT shared library

### Custom Library Path

If your OMT library is installed in a non-standard location, set the library search path:

```bash
# Linux/macOS
export LIBRARY_PATH=/path/to/omt/lib:$LIBRARY_PATH
export LD_LIBRARY_PATH=/path/to/omt/lib:$LD_LIBRARY_PATH

# Build
cargo build
```

## Examples

### Creating a Sender

```rust
use omt_sys::*;
use std::ffi::CString;

unsafe {
    let name = CString::new("My Sender").unwrap();
    let sender = OMTSender_Create(name.as_ptr(), OMTQuality_High);
    
    if !sender.is_null() {
        // Use sender...
        
        // Cleanup
        OMTSender_Destroy(sender);
    }
}
```

### Creating a Receiver

```rust
use omt_sys::*;
use std::ffi::CString;

unsafe {
    let address = CString::new("omt://hostname:6400").unwrap();
    let receiver = OMTReceiver_Create(
        address.as_ptr(),
        OMTFrameType_Video | OMTFrameType_Audio,
        OMTPreferredVideoFormat_UYVY,
        OMTReceiveFlags_None,
    );
    
    if !receiver.is_null() {
        // Receive frames...
        let frame = OMTReceiver_ReceiveVideo(receiver, 1000);
        
        // Cleanup
        OMTReceiver_Destroy(receiver);
    }
}
```

## Thread Safety

The OMT C library handles its own thread safety. However, you must ensure that Rust's borrowing rules and thread safety guarantees are maintained when using these bindings from multiple threads.

## Metadata Specification

OMT supports XML-based metadata for various use cases. See [`docs/METADATA.md`](docs/METADATA.md) for specifications including:

- Web management interfaces
- PTZ camera control (VISCA over IP and inband)
- Ancillary data (SDI ANC packets)
- Metadata grouping

## Documentation

Full C API documentation is available in `libomt.h`. Key concepts:

- **Color Spaces**: Automatic detection (BT.601 for SD, BT.709 for HD+) or manual specification
- **Quality Levels**: Sender can accept receiver quality suggestions or override
- **Video Flags**: Support for interlaced, alpha, high bit depth, and preview frames
- **Frame Lifecycle**: Frames must be explicitly destroyed after use

## Safety Considerations

When using `omt-sys` directly:

1. **Null checks**: Always check that pointers returned from OMT are non-null
2. **Memory management**: Destroy/free all resources when done
3. **String handling**: Use `CString` for passing Rust strings to C
4. **Lifetimes**: Ensure frames are not destroyed while still in use
5. **Error handling**: Check return values and error conditions

## Comparison: omt-sys vs omt

| Feature | omt-sys | omt |
|---------|---------|-----|
| API Style | Raw C FFI | Safe Rust API |
| Memory Safety | Manual (`unsafe`) | Automatic (RAII) |
| Error Handling | Return codes | `Result<T, E>` |
| Type Safety | C types | Rust types |
| Resource Cleanup | Manual | Automatic (Drop) |
| Documentation | C headers | Rust docs |
| Learning Curve | C API knowledge | Rust idioms |

**Recommendation**: Use the [`omt`](../omt) crate unless you specifically need low-level control.

## Contributing

Contributions are welcome! Please note:

- This crate is mostly auto-generated from C headers
- Changes should be made to the build process or header files
- Test any changes with both the C library and high-level `omt` crate

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- **[omt](../omt)**: High-level, safe Rust bindings (recommended for most users)
- **[libomt](https://github.com/openmediatransport/libomt)**: The underlying C implementation
- **[Open Media Transport](https://github.com/openmediatransport)**: The overall OMT project

## Support

For issues specific to these FFI bindings, please open an issue on this repository. For questions about the OMT protocol or C library, refer to the [openmediatransport organization](https://github.com/openmediatransport) or the [libomt repository](https://github.com/openmediatransport/libomt).