# omt — High‑level Rust wrapper for libomt

This crate (`omt`) provides a high‑level Rust API for the **libomt** C library from the Open Media Transport (OMT) project. It includes:

- Ergonomic wrappers around OMT discovery, sending, and receiving.
- Strongly‑typed enums for codecs, flags, and formats that map to `libomt.h`.
- Small example binaries that exercise discovery, receive, and send flows.

Open Media Transport (OMT) is an open-source network protocol for high‑performance, low‑latency video over a LAN. It supports multiple HD A/V feeds on standard gigabit networks, transports streams over TCP, and publishes sources via DNS‑SD or a discovery server. See: https://github.com/openmediatransport

---

## Table of contents

- [Requirements](#requirements)
- [Build](#build)
- [Quick start](#quick-start)
  - [Discover sources](#discover-sources)
  - [Receive video](#receive-video)
  - [Send video](#send-video)
  - [Metadata](#metadata)
- [API overview](#api-overview)
  - [Discovery](#discovery)
  - [Receiver](#receiver)
  - [Sender](#sender)
  - [Frames and data access](#frames-and-data-access)
  - [Settings and logging](#settings-and-logging)
- [Video formats and flags](#video-formats-and-flags)
- [Quality selection](#quality-selection)
- [Networking and discovery](#networking-and-discovery)
- [Safety and lifetime notes](#safety-and-lifetime-notes)
- [Examples](#examples)
- [License](#license)

---

## Requirements

- **libomt** installed on your system and discoverable by the linker.
- Rust toolchain (edition 2021).

On macOS/Linux you may need to set:

- `LIBRARY_PATH` or `RUSTFLAGS="-L /path/to/libomt"`
- `DYLD_LIBRARY_PATH` (macOS) or `LD_LIBRARY_PATH` (Linux) at runtime

The `omt` crate links with:

```
#[link(name = "omt")]
```

So the linker expects `libomt` to be available as a shared library.

---

## Build

From the project root:

```
cargo build
```

---

## Quick start

### Discover sources

```rust
use omt::Discovery;
use std::time::Duration;

// Single attempt - may miss sources on first call
let addresses = Discovery::get_addresses();
for addr in addresses {
    println!("{}", addr);
}

// Multiple attempts with delay - recommended for complete results
let addresses = Discovery::get_addresses_with_options(3, Duration::from_millis(100));
for addr in addresses {
    println!("{}", addr);
}
```

**Note:** The first call to `get_addresses()` starts discovery in a background thread, so initial results may be incomplete. Use `get_addresses_with_options()` or `get_addresses_with_backoff()` for reliable discovery.

### Receive video

```rust
use omt::{Address, Receiver, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};

let address = Address::new("HOST (Sender Name)");
let mut receiver = Receiver::create(
    &address,
    FrameType::Video,
    PreferredVideoFormat::UYVYorBGRA,
    ReceiveFlags::NONE,
).expect("create receiver");

if let Ok(Some(frame)) = receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
    println!("{}x{} @ {}/{}", frame.width(), frame.height(), frame.frame_rate().0, frame.frame_rate().1);
}
```



### Send video

```rust
use omt::{Sender, Name, MediaFrame, Codec, ColorSpace, Quality, VideoFlags};

let name = Name::new("My Sender");
let sender = Sender::create(&name, Quality::Default).expect("create sender");

// Fill `data` with raw uncompressed frame bytes for the chosen format.
let data = vec![0u8; 1920 * 1080 * 2]; // UYVY example (2 bytes per pixel)

let mut frame = MediaFrame::video(
    Codec::UYVY,
    1920,
    1080,
    1920 * 2,
    VideoFlags::NONE,
    60,
    1,
    16.0 / 9.0,
    ColorSpace::BT709,
    -1, // timestamp; -1 = let OMT pace by frame rate
    data,
);

sender.send(&mut frame);
```

### Metadata

OMT supports bi-directional XML metadata transmission. The protocol uses UTF-8 encoded XML strings. The Rust wrapper automatically handles null termination.

#### Creating and sending metadata frames

```rust
use omt::MediaFrame;

// Simple metadata
let mut metadata = MediaFrame::metadata(
    "<example><value>42</value></example>",
    123456789,
).expect("metadata frame");

// Send metadata with Sender::send(...)
```

#### Web management interface

```rust
let mut web_metadata = MediaFrame::metadata(
    r#"<OMTWeb URL="http://192.168.1.100/" />"#,
    -1,
).expect("web management metadata");
```

#### PTZ control metadata

```rust
// VISCA over IP
let mut ptz_visca_ip = MediaFrame::metadata(
    r#"<OMTPTZ Protocol="VISCAoverIP" URL="visca://192.168.1.100:52381" />"#,
    -1,
).expect("PTZ VISCA over IP");

// VISCA inband command
let mut ptz_command = MediaFrame::metadata(
    r#"<OMTPTZ Protocol="VISCA" Sequence="22" Command="8101040700FF" />"#,
    -1,
).expect("PTZ command");

// VISCA inband reply
let mut ptz_reply = MediaFrame::metadata(
    r#"<OMTPTZ Protocol="VISCA" Sequence="22" Reply="0011AABBCC" />"#,
    -1,
).expect("PTZ reply");
```

#### Grouped metadata

To send multiple metadata elements in a single frame:

```rust
let grouped = r#"<OMTGroup>
<OMTPTZ Protocol="VISCA" Sequence="22" Reply="0011AABBCC" />
<AncillaryData xmlns="urn:anc:1.0">
<Packet did="45" sdid="01" field="1" line="21" horizOffset="0" st2110Channel="0" pts90k="32109876" link="A" stream="VANC">
<Payload>81010A011E0000</Payload>
</Packet>
</AncillaryData>
</OMTGroup>"#;

let mut metadata = MediaFrame::metadata(grouped, -1).expect("grouped metadata");
```

#### Per-frame metadata

You can attach metadata to individual video or audio frames:

```rust
use omt::{MediaFrame, Codec, VideoFlags, ColorSpace};

let mut frame = MediaFrame::video(
    Codec::BGRA,
    1920,
    1080,
    1920 * 4,
    VideoFlags::NONE,
    30,
    1,
    1.77778,
    ColorSpace::BT709,
    -1,
    vec![0u8; 1920 * 1080 * 4],
);

// Attach per-frame metadata
frame.set_frame_metadata("<custom>frame-specific metadata</custom>").unwrap();
```



### Sender information

Query sender device/software metadata:

```rust
use omt::{Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver, Timeout};

let addresses = Discovery::addresses_with_backoff(
    3,
    Timeout::from_millis(200).as_duration(),
    Timeout::from_millis(500).as_duration(),
    2.0,
);

for address in addresses {
    let mut receiver = Receiver::create(
        &address,
        FrameType::Video,
        PreferredVideoFormat::UYVYorBGRA,
        ReceiveFlags::NONE,
    )
    .expect("create receiver");

    if let Some(info) = receiver.get_sender_info() {
        println!("{} -> {} {} v{}", 
            address, 
            info.manufacturer, 
            info.product_name,
            info.version
        );
    }
}
```

### Connection-level metadata

Senders can attach metadata that is sent to all new connections:

```rust
use omt::{Sender, Name, Quality};

let name = Name::new("My Camera");
let sender = Sender::create(&name, Quality::Default).expect("create sender");

// Add web management interface
sender.add_connection_metadata(r#"<OMTWeb URL="http://192.168.1.100/" />"#).unwrap();

// Add PTZ control information
sender.add_connection_metadata(
    r#"<OMTPTZ Protocol="VISCAoverIP" URL="visca://192.168.1.100:52381" />"#
).unwrap();

// Clear all connection metadata
sender.clear_connection_metadata();
```

---

## API overview

The high-level API is re-exported at the crate root (`omt::*`). Newtypes follow `libomt.h` naming: use `Address` for receiver addresses and `Name` for sender names.

### Discovery

**Type:** `Discovery`  
**Purpose:** Find advertised OMT sources on the LAN.

Key APIs:

- `Discovery::get_addresses() -> Vec<Address>` — Single attempt
- `Discovery::get_addresses_with_options(attempts, delay)` — Multiple attempts with fixed delay
- `Discovery::get_addresses_with_backoff(attempts, initial_delay, max_delay, backoff_factor)` — Multiple attempts with exponential backoff
- `Discovery::addresses()` — Iterator over single attempt results
- `Discovery::addresses_with_backoff(...)` — Iterator with backoff

Discovery uses DNS‑SD (Bonjour/Avahi) or a discovery server depending on your network setup. The first discovery call starts a background thread, so multiple attempts with delays are recommended for complete results. Debug output is controlled via the `RUST_LOG` environment variable (see [Logging](#logging-with-the-log-crate)).

### Receiver

**Type:** `Receiver`  
**Purpose:** Connect to a sender and receive video/audio/metadata.

Key APIs:

- `Receiver::create(address, frame_types, preferred_format, flags) -> Result<Receiver, OmtError>`
- `Receiver::receive(frame_types, timeout: Timeout) -> Result<Option<MediaFrame>, OmtError>`
- `Receiver::send_metadata(&mut metadata_frame) -> Result<i32, OmtError>`
- `Receiver::set_tally(tally)`
- `Receiver::get_tally(timeout: Timeout, &mut tally) -> i32`
- `Receiver::set_flags(flags)`
- `Receiver::set_suggested_quality(quality)`
- `Receiver::get_sender_info() -> Option<SenderInfo>`
- `Receiver::get_video_statistics() -> Statistics`
- `Receiver::get_audio_statistics() -> Statistics`

#### Frame types

Use `FrameType` to select what to receive:

- `FrameType::Video`
- `FrameType::Audio`
- `FrameType::Metadata`

### Sender

**Type:** `Sender`  
**Purpose:** Publish a source and send video/metadata to connected receivers.

Key APIs:

- `Sender::create(name: &Name, quality) -> Result<Sender, OmtError>`
- `Sender::send(&mut MediaFrame) -> i32`
- `Sender::connections() -> i32`
- `Sender::receive_metadata(timeout: Timeout) -> Result<Option<MediaFrame>, OmtError>`
- `Sender::set_sender_info(&SenderInfo)`
- `Sender::add_connection_metadata(metadata)`
- `Sender::clear_connection_metadata()`
- `Sender::set_redirect(Some(address: &Address) | None)`
- `Sender::get_address() -> Option<Address>`
- `Sender::get_video_statistics() -> Statistics`
- `Sender::get_audio_statistics() -> Statistics`

### Frames and data access

Received frames are exposed through `MediaFrame`.

- `MediaFrame::frame_type()` → `FrameType`
- `MediaFrame::timestamp()` → `i64` (OMT timebase; 10,000,000 ticks per second)
- `MediaFrame::codec()` → `Codec`
- `MediaFrame::xml_data()` → `Option<&str>` (UTF‑8 XML metadata without null terminator, for Metadata frame types)

`MediaFrame` provides:

- `width()`, `height()`, `stride()`
- `frame_rate() -> (i32, i32)`
- `aspect_ratio()`
- `color_space()`
- `flags() -> VideoFlags`
- `raw_data() -> Option<&[u8]>` (uncompressed pixel data)
- `rgb8_data() -> Option<Vec<u8>>` (8-bit RGB conversion, 3 bytes per pixel)
- `rgba8_data() -> Option<Vec<u8>>` (8-bit RGBA conversion, 4 bytes per pixel)
- `rgb16_data() -> Option<Vec<u8>>` (16-bit RGB conversion, 6 bytes per pixel)
- `rgba16_data() -> Option<Vec<u8>>` (16-bit RGBA conversion, 8 bytes per pixel)
- `compressed_data() -> Option<&[u8]>` (VMX1 if `ReceiveFlags::INCLUDE_COMPRESSED` or `COMPRESSED_ONLY`)
- `frame_metadata() -> Option<&str>` (per‑frame metadata payload without null terminator)

For audio frames, `MediaFrame` also provides:

- `sample_rate()`, `channels()`, `samples_per_channel()`
- `audio_data() -> Option<Vec<Vec<f32>>>` (planar 32‑bit float audio)

#### Timestamps and metadata

- Timestamps use the OMT timebase (10,000,000 ticks per second) and should represent the original capture time for proper synchronization.
- For outbound video frames, a timestamp of `-1` asks the sender to generate timestamps and pace delivery according to the frame rate.
- **Metadata strings and null terminators:**
  - The low-level C API (`libomt.h`) requires metadata strings to *include* the null terminator in both the data and length.
  - The high-level Rust wrapper handles this automatically: functions accepting metadata strings expect strings *without* null terminators and will add the null terminator behind the scenes.
  - Passing a string containing a null character (`'\0'`) to Rust wrapper functions will result in an error.
- Received frame buffers are valid only until the next receive call on the same sender/receiver.

### Settings and logging

- `set_logging_filename(Some(path) | None)`
- `settings_get_string(name) -> Option<String>`
- `settings_set_string(name, value)`
- `settings_get_integer(name) -> Option<i32>`
- `settings_set_integer(name, value)`

#### Convenience methods for documented settings

The following convenience methods are available for settings documented in `libomt.h`:

- `get_discovery_server() -> Option<String>` - Get DiscoveryServer URL
- `set_discovery_server(server: &str)` - Set DiscoveryServer URL (empty string for DNS-SD)
- `get_network_port_start() -> i32` - Get first port for Send instances (default: 6400)
- `set_network_port_start(port: i32)` - Set first port for Send instances
- `get_network_port_end() -> i32` - Get last port for Send instances (default: 6600)
- `set_network_port_end(port: i32)` - Set last port for Send instances
- `get_network_port_range() -> (i32, i32)` - Get port range as tuple
- `set_network_port_range(start: i32, end: i32)` - Set port range

#### Logging with the `log` crate

The library uses the Rust `log` crate for debug and diagnostic output. To enable logging:

1. Add `env_logger` to your dependencies:
   ```toml
   [dependencies]
   env_logger = "0.11"
   ```

2. Initialize the logger in your application:
   ```rust
   fn main() {
       env_logger::init();
       // Your code...
   }
   ```

3. Control log level via the `RUST_LOG` environment variable:
   ```bash
   RUST_LOG=debug cargo run --example list_senders
   RUST_LOG=info cargo run
   RUST_LOG=error cargo run
   ```

The library uses these log levels:
- `error!` for failures that prevent normal operation
- `warn!` for recoverable issues or unexpected conditions  
- `info!` for general operational information
- `debug!` for detailed debugging information

Discovery debug output is now controlled via the `RUST_LOG` environment variable instead of explicit debug flags.

### Timeout helpers

The API uses `Timeout` for receive timeouts, with convenience constructors:

```rust
use omt::Timeout;

let t1 = Timeout::from_millis(1000);
let t2 = Timeout::from_secs(2);
```

---

## Video formats and flags

OMT supports multiple pixel formats and alpha channel options. In this wrapper:

### Codecs (`Codec`)

All codecs listed below are supported by the library:

- `VMX1` — fast compressed video codec.
- `UYVY` — 16‑bit YUV packed format (4:2:2).
- `YUY2` — 16‑bit YUV packed format with YUYV pixel order (4:2:2).
- `UYVA` — UYVY followed by a full‑resolution alpha plane.
- `NV12` — planar 4:2:0 YUV format (Y plane followed by interleaved UV plane).
- `YV12` — planar 4:2:0 YUV format (Y plane followed by separate U and V planes).
- `BGRA` — 32‑bit RGBA format (Win32 ARGB32 layout).
- `P216` — planar 4:2:2 YUV with 16‑bit components (Y plane + interleaved UV plane).
- `PA16` — `P216` plus a 16‑bit alpha plane.
- `FPA1` — planar 32‑bit float audio.
- `Unknown(i32)` for non‑standard values.

When receiving uncompressed video, OMT delivers only `UYVY`, `UYVA`, `BGRA`, or `BGRX` (alpha omitted). Other formats may arrive as `VMX1` and can be decoded using the conversion methods (`rgb8_data()`, `rgba8_data()`, etc.).

### Format Conversion Support

The `MediaFrame` provides format conversion methods: `rgb8_data()`, `rgba8_data()`, `rgb16_data()`, and `rgba16_data()`. **However, not all codecs support all output formats.** The following table shows which conversions are currently implemented:

| Input Codec | `rgb8_data()` | `rgba8_data()` | `rgb16_data()` | `rgba16_data()` |
|-------------|---------------|----------------|--------------|-----------------|
| **UYVY**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **YUY2**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **NV12**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **YV12**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **BGRA**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **UYVA**    | ✅ Yes      | ✅ Yes       | ❌ No        | ❌ No         |
| **P216**    | ❌ No       | ❌ No        | ✅ Yes       | ✅ Yes        |
| **PA16**    | ❌ No       | ❌ No        | ✅ Yes       | ✅ Yes        |
| **VMX1**    | ❌ No       | ❌ No        | ❌ No        | ❌ No         |
| **FPA1**    | ❌ No       | ❌ No        | ❌ No        | ❌ No         |

**Key points:**
- **8-bit codecs** (UYVY, YUY2, NV12, YV12, BGRA, UYVA) support conversion via `rgb8_data()` and `rgba8_data()` only.
- **16-bit codecs** (P216, PA16) support conversion via `rgb16_data()` and `rgba16_data()` only.
- **Compressed codecs** (VMX1) and **audio codecs** (FPA1) do not support these conversion functions. VMX1 frames must be decoded by OMT first (they will arrive as one of the uncompressed formats above).
- For unsupported conversions, the conversion methods return `None`.
- The `UYVA` format supports alpha channel output when using `rgba8_data()`.
- The `PA16` format supports alpha channel output when using `rgba16_data()`.

### Preferred receive formats (`PreferredVideoFormat`)

- `UYVY`
- `UYVYorBGRA`
- `BGRA`
- `UYVYorUYVA`
- `UYVYorUYVAorP216orPA16`
- `P216`

### Video flags (`VideoFlags`)

- `NONE` — no special flags.
- `INTERLACED` — frame is interlaced.
- `ALPHA` — frame contains an alpha channel (if unset, `BGRA` is treated as `BGRX` and `UYVA` as `UYVY`).
- `PREMULTIPLIED` — alpha channel is premultiplied (only meaningful when `ALPHA` is set). Note: The current video conversion implementation does not handle premultiplied alpha differently from straight alpha.
- `PREVIEW` — sender emitted a 1/8th preview frame.
- `HIGH_BIT_DEPTH` — set for `P216`/`PA16` sources and for `VMX1` that originated from those formats, so decoders can select the right output format.

### Video conversion output formats

The conversion methods return pixel data in the following formats:

- `rgb8_data()` — 8-bit RGB (24-bit per pixel, 3 bytes: R, G, B)
- `rgba8_data()` — 8-bit RGBA (32-bit per pixel, 4 bytes: R, G, B, A with straight alpha)
- `rgb16_data()` — 16-bit RGB (48-bit per pixel, 6 bytes: R, G, B at 16-bit per component)
- `rgba16_data()` — 16-bit RGBA (64-bit per pixel, 8 bytes: R, G, B, A at 16-bit per component with straight alpha)

**Important notes:**
- See the conversion support table above for which codecs support which output formats.
- The `PREMULTIPLIED` flag is not currently handled during conversion; all alpha is treated as straight alpha.
- For unsupported codec/format combinations, the conversion methods return `None`.

---

## Quality selection

Senders select a quality level with `Quality`:

- `Low`, `Medium`, `High`, `Default`

When a sender uses `Default`, it starts at **Medium** and allows receivers to suggest a preferred quality. The sender then chooses the **highest suggested quality** across connected receivers. If a receiver is set to `Default`, it defers to other receivers’ suggestions.

```rust
use omt::{Receiver, Quality};

receiver.set_suggested_quality(Quality::High);
```

---

## Networking and discovery

- OMT streams audio and video over **TCP**.
- Each sender listens on a single port; a receiver may open up to **two TCP connections** (separate audio/video streams).
- Discovery uses **DNS‑SD** (Bonjour/Avahi, multicast UDP) or an optional **Discovery Server** (TCP) when multicast is unavailable.
- Default sender port range: **6400–6600**
- DNS‑SD uses UDP port **5353**
- Discovery server default port: **6399**

---

## Safety and lifetime notes

The `omt` crate is safe to use as long as you follow these rules:

1. **Frame lifetimes**  
   Frames returned by `Receiver::receive` and `Sender::receive_metadata` are valid only until the next call on the same receiver/sender. Do not store references beyond that.

2. **Data ownership**  
   - Received frame buffers are owned by libomt. Do **not** free them.
   - Outgoing frames own their `Vec<u8>` payload. Keep the `MediaFrame` alive until `Sender::send` returns.

3. **FFI zero‑init**  
   When using raw FFI types, zero‑initialize `OMTMediaFrame` before use. The high-level API does this for you.

---

## Examples

The `omt` crate includes runnable examples under `examples/` that use discovery and direct receive calls:

- `list_senders` discovers sources and prints their video format.
- `view_stream` renders frames to the terminal.
- `rebroadcast_bw` rebroadcasts a grayscale view of a stream.

Run them from the project root:

```
cargo run --example list_senders
cargo run --example view_stream
cargo run --example rebroadcast_bw
```



Example output:

```
Discovered 2 sender(s):
- HOST1 (Camera 1)
  -> Video: 1920x1080 @ 60/1 fps, codec UYVY, flags [None], colorspace BT709
- HOST2 (Camera 2)
  -> Video: 3840x2160 @ 30/1 fps, codec BGRA, flags [HighBitDepth], colorspace BT709
```

---

## License

MIT (matches libomt).
