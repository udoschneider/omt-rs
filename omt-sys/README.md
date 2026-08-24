# omt-sys

Low-level FFI bindings for [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt).

**Note:** This is an **unofficial, third-party** Rust wrapper. It is not affiliated with or endorsed by the Open Media Transport project.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

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
omt-sys = { git = "https://github.com/udoschneider/omt-rs" }
```

> **Not yet on crates.io.** Both crates are still marked `publish = false`
> while the API settles, so there is no `crates.io` release or `docs.rs` page
> yet. Depend on the repository directly, and build the API docs locally with
> `cargo doc --workspace --no-deps --open`.

### Prerequisites

On macOS and Windows the build script downloads the pinned prebuilt `libomt`
binaries from the [`libomtnet` releases](https://github.com/openmediatransport/libomtnet/releases)
automatically, so there is nothing to install. The download is verified against
a SHA-256 checksum and cached under `~/.cargo/omt/<version>/`.

On Linux (no prebuilt binaries are published) you must obtain `libomt` yourself:

1. **Build or download libomt**: Build from source at [openmediatransport/libomt](https://github.com/openmediatransport/libomt)
   or use a distribution package.

2. **Install the library** to `/usr/local/lib` or `/usr/lib`, **or** set
   `OMT_LIB_DIR` to the directory containing `libomt.so`.

Set `OMT_LIB_DIR` on any platform to skip the download and link against a
specific directory (e.g. for offline builds).

## Usage

```rust
use omt_sys::*;

fn main() {
    unsafe {
        // Get available sources on the network. The returned `char**` array is
        // owned by the C library and remains valid only until the next call —
        // there is no free function, so copy out anything you need immediately.
        let mut count: i32 = 0;
        let addresses = omt_discovery_getaddresses(&mut count as *mut i32);

        println!("Found {} OMT sources", count);
        let _ = addresses;
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

- **`omt_send_t`**: Opaque sender handle
- **`omt_receive_t`**: Opaque receiver handle
- **`OMTMediaFrame`**: Frame structure containing pixel/audio data and metadata
- **`OMTSenderInfo`**: Sender description (product name, manufacturer, version)
- **`OMTStatistics`**: Frame/byte/codec-time counters
- **`OMTTally`**: Program/preview tally state

### Enumerations

- **`OMTFrameType`**: Frame types (None, Metadata, Video, Audio)
- **`OMTCodec`**: Supported codecs (VMX1, UYVY, YUY2, BGRA, NV12, YV12, UYVA, P216, PA16, FPA1)
- **`OMTQuality`**: Encoding quality levels (Default, Low, Medium, High)
- **`OMTColorSpace`**: Color space definitions (Undefined, BT601, BT709)
- **`OMTVideoFlags`**: Video frame flags (Interlaced, Alpha, PreMultiplied, Preview, HighBitDepth)
- **`OMTPreferredVideoFormat`**: Preferred output video format for receivers
- **`OMTReceiveFlags`**: Receiver configuration flags

### Functions

The bindings expose all OMT C API functions. Function names are lower
snake_case, matching the C header exactly. The main ones are:

#### Discovery
- `omt_discovery_getaddresses()` - Get available sources (returns a `char**`
  owned by the library; there is no corresponding free function)

#### Sender
- `omt_send_create()` - Create sender
- `omt_send_destroy()` - Destroy sender
- `omt_send()` - Send a media frame (video, audio, or metadata)
- `omt_send_receive()` - Receive metadata from receivers
- `omt_send_gettally()` - Get tally state
- `omt_send_setsenderinformation()` - Set sender metadata
- `omt_send_connections()` - Number of active connections
- `omt_send_getaddress()` - Get the discovery address

#### Receiver
- `omt_receive_create()` - Create receiver
- `omt_receive_destroy()` - Destroy receiver
- `omt_receive()` - Receive a frame; the requested frame type(s) are passed as
  an argument, so a single function covers video, audio, and metadata
- `omt_receive_send()` - Send metadata back to the sender
- `omt_receive_settally()` - Set tally state
- `omt_receive_setflags()` - Change receive flags
- `omt_receive_setsuggestedquality()` - Suggest an encoding quality

#### Frame data

The C API does not create or destroy frames; `OMTMediaFrame` is a plain struct
you populate and pass to `omt_send`. The frames returned by `omt_receive` point
into buffers owned by the library and must not be freed by the caller (they are
valid only until the next receive of the same type on that instance).

#### Statistics
- `omt_send_getvideostatistics()` / `omt_send_getaudiostatistics()`
- `omt_receive_getvideostatistics()` / `omt_receive_getaudiostatistics()`

#### Settings
- `omt_settings_set_string()` / `omt_settings_get_string()` - String settings
  (e.g. `"DiscoveryServer"`)
- `omt_settings_set_integer()` / `omt_settings_get_integer()` - Integer settings
  (e.g. `"NetworkPortStart"`, `"NetworkPortEnd"`)
- `omt_setloggingfilename()` - Configure the log file
- `omt_setloggingcallback()` - Register a logging callback

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

1. Resolves the `libomt` shared library, in order of preference:
   - `OMT_LIB_DIR` (explicit override — never touches the network)
   - a previously downloaded cache (offline after the first build)
   - the pinned prebuilt release, downloaded and SHA-256-verified (macOS/Windows)
   - the conventional system locations `/usr/local/lib`, `/usr/lib`, `/opt/homebrew/lib`

2. Generates Rust bindings from the vendored `libomt.h` using `bindgen`. The
   downloaded release's header is verified to match the vendored one, so the
   bindings and the binary always describe the same ABI.

3. Links against the `omt` shared library.

The resolved library location is exported to dependent crates via the
`links = "omt"` metadata (`DEP_OMT_LIBDIR`) and to this crate's code via the
[`OMT_LIB_DIR`] / [`OMT_LIB_FILE`] constants.

### Custom Library Path

If your OMT library is installed in a non-standard location, set `OMT_LIB_DIR`
(or `LIBRARY_PATH`):

```bash
# Linux/macOS
export OMT_LIB_DIR=/path/to/omt/lib

# Build
cargo build
```

`OMT_CACHE_DIR` relocates the download cache (default `~/.cargo/omt/<version>`).

## Examples

### Creating a Sender

```rust
use omt_sys::*;
use std::ffi::CString;

unsafe {
    let name = CString::new("My Sender").unwrap();
    let sender = omt_send_create(name.as_ptr(), OMTQuality_High);

    if !sender.is_null() {
        // Use sender...

        // Cleanup
        omt_send_destroy(sender);
    }
}
```

### Creating a Receiver

```rust
use omt_sys::*;
use std::ffi::CString;

unsafe {
    let address = CString::new("omt://hostname:6400").unwrap();
    let receiver = omt_receive_create(
        address.as_ptr(),
        OMTFrameType_Video | OMTFrameType_Audio,
        OMTPreferredVideoFormat_UYVY,
        OMTReceiveFlags_None,
    );

    if !receiver.is_null() {
        // Receive a video frame (frame type is an argument, not a separate call)
        let frame = omt_receive(receiver, OMTFrameType_Video, 1000);
        let _ = frame;

        // Cleanup
        omt_receive_destroy(receiver);
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
- **Frame Lifecycle**: Frames returned by `omt_receive` are owned by the library
  and must **not** be freed; they stay valid only until the next receive of the
  same type on that instance. Frames you build for `omt_send` are plain structs
  you own — there is no create/destroy call in the C API.

## Safety Considerations

When using `omt-sys` directly:

1. **Null checks**: Always check that pointers returned from OMT are non-null
2. **Memory management**: Destroy senders/receivers (`omt_send_destroy` /
   `omt_receive_destroy`) when done. Received frame buffers are library-owned and
   must not be freed.
3. **String handling**: Use `CString` for passing Rust strings to C
4. **Lifetimes**: Don't hold a received frame past the next receive call on the
   same instance and frame type — its buffer is reused
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

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Related Projects

- **[omt](../omt)**: High-level, safe Rust bindings (recommended for most users)
- **[libomt](https://github.com/openmediatransport/libomt)**: The underlying C implementation
- **[Open Media Transport](https://github.com/openmediatransport)**: The overall OMT project

## Support

For issues specific to these FFI bindings, please open an issue on this repository. For questions about the OMT protocol or C library, refer to the [openmediatransport organization](https://github.com/openmediatransport) or the [libomt repository](https://github.com/openmediatransport/libomt).