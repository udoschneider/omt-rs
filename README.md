# libomt (Rust) — High‑level wrapper

This crate provides a high‑level Rust API for the **libomt** C library from the Open Media Transport (OMT) project. It includes:

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

The crate links with:

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
use libomt::Discovery;

let addresses = Discovery::get_addresses();
for addr in addresses {
    println!("{}", addr);
}
```

### Receive video

```rust
use libomt::{Address, Receiver, FrameType, PreferredVideoFormat, ReceiveFlags, Timeout};

let address = Address::new("HOST (Sender Name)");
let mut receiver = Receiver::create(
    &address,
    FrameType::Video,
    PreferredVideoFormat::UYVYorBGRA,
    ReceiveFlags::NONE,
).expect("create receiver");

if let Ok(Some(frame)) = receiver.receive(FrameType::Video, Timeout::from_millis(1000)) {
    let video = frame.video().expect("video frame");
    println!("{}x{} @ {}/{}", video.width(), video.height(), video.frame_rate().0, video.frame_rate().1);
}
```



### Send video

```rust
use libomt::{Sender, Source, OutgoingFrame, Codec, ColorSpace, Quality, VideoFlags};

let source = Source::new("My Sender");
let sender = Sender::create(&source, Quality::Default).expect("create sender");

// Fill `data` with raw uncompressed frame bytes for the chosen format.
let data = vec![0u8; 1920 * 1080 * 2]; // UYVY example (2 bytes per pixel)

let mut frame = OutgoingFrame::video(
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

```rust
use libomt::OutgoingFrame;

let mut metadata = OutgoingFrame::metadata_xml(
    "<example><value>42</value></example>",
    123456789,
).expect("metadata frame");

// Send metadata with Sender::send(...)
```



### Sender metadata

```rust
use libomt::{Discovery, FrameType, PreferredVideoFormat, ReceiveFlags, Receiver, Timeout};

let addresses = Discovery::addresses_with_backoff(
    3,
    Timeout::from_millis(200).as_duration(),
    Timeout::from_millis(500).as_duration(),
    2.0,
    false,
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
        println!("{} -> {} {}", address, info.manufacturer, info.product_name);
    }
}
```

---

## API overview

The high-level API is re-exported at the crate root. Newtypes follow `libomt.h` naming: use `Address` for receiver addresses and `Source` for sender names.

### Discovery

**Type:** `Discovery`  
**Purpose:** Find advertised OMT sources on the LAN.

Key APIs:

- `Discovery::get_addresses() -> Vec<Address>`
- `Discovery::get_addresses_with_options(attempts, delay: Duration, debug)`
- `Discovery::get_addresses_with_backoff(attempts, initial_delay: Duration, max_delay: Duration, backoff_factor, debug)`


Discovery uses DNS‑SD (Bonjour/Avahi) or a discovery server depending on your network setup.

### Receiver

**Type:** `Receiver`  
**Purpose:** Connect to a sender and receive video/audio/metadata.

Key APIs:

- `Receiver::create(address, frame_types, preferred_format, flags) -> Result<Receiver, OmtError>`
- `Receiver::receive(frame_types, timeout: Timeout) -> Result<Option<FrameRef>, OmtError>`
- `Receiver::send_metadata_xml(xml, timestamp) -> Result<i32, OmtError>`
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

- `Sender::create(source: &Source, quality) -> Result<Sender, OmtError>`
- `Sender::send(&mut OutgoingFrame) -> i32`
- `Sender::connections() -> i32`
- `Sender::receive_metadata(timeout: Timeout) -> Result<Option<FrameRef>, OmtError>`
- `Sender::set_sender_info(&SenderInfo)`
- `Sender::add_connection_metadata(metadata)`
- `Sender::clear_connection_metadata()`
- `Sender::set_redirect(Some(address: &Address) | None)`
- `Sender::get_address() -> Option<Address>`
- `Sender::get_video_statistics() -> Statistics`
- `Sender::get_audio_statistics() -> Statistics`

### Frames and data access

Received frames are exposed through `FrameRef`, `VideoFrame`, and `AudioFrame`.

- `FrameRef::frame_type()` → `FrameType`
- `FrameRef::timestamp()` → `i64` (OMT timebase; 10,000,000 ticks per second)
- `FrameRef::codec()` → `Codec`
- `FrameRef::video()` → `Option<VideoFrame>`
- `FrameRef::audio()` → `Option<AudioFrame>`
- `FrameRef::metadata()` → `Option<&[u8]>` (UTF‑8 XML with terminating null)

`VideoFrame` provides:

- `width()`, `height()`, `stride()`
- `frame_rate() -> (i32, i32)`
- `aspect_ratio()`
- `color_space()`
- `flags() -> VideoFlags`
- `raw_data() -> Option<&[u8]>` (uncompressed pixel data)
- `data(format: VideoDataFormat) -> Option<Vec<u8>>` (converted RGB/RGBA output)
- `compressed_data() -> Option<&[u8]>` (VMX1 if `ReceiveFlags::INCLUDE_COMPRESSED` or `COMPRESSED_ONLY`)
- `metadata() -> Option<&[u8]>` (per‑frame metadata payload)

`AudioFrame` provides:

- `sample_rate()`, `channels()`, `samples_per_channel()`
- `raw_data() -> Option<&[u8]>` (planar 32‑bit float audio, FPA1)
- `data() -> Option<Vec<Vec<f32>>>`
- `compressed_data()` and `metadata()` (if present)

#### Timestamps and metadata

- Timestamps use the OMT timebase (10,000,000 ticks per second) and should represent the original capture time for proper synchronization.
- For outbound video frames, a timestamp of `-1` asks the sender to generate timestamps and pace delivery according to the frame rate.
- Metadata frames and per‑frame metadata payloads are UTF‑8 XML with a terminating null; lengths include the null byte.
- Received frame buffers are valid only until the next receive call on the same sender/receiver.

### Settings and logging

- `set_logging_filename(Some(path) | None)`
- `settings_get_string(name) -> Option<String>`
- `settings_set_string(name, value)`
- `settings_get_integer(name) -> Option<i32>`
- `settings_set_integer(name, value)`

### Timeout helpers

The API uses `Timeout` for receive timeouts, with convenience constructors:

```rust
use libomt::Timeout;

let t1 = Timeout::from_millis(1000);
let t2 = Timeout::from_secs(2);
```

---

## Video formats and flags

OMT supports multiple pixel formats and alpha channel options. In this wrapper:

### Codecs (`Codec`)

- `VMX1` — fast compressed video codec.
- `UYVY`, `YUY2` — 16‑bit YUV packed formats (YUY2 = YUYV order).
- `UYVA` — UYVY followed by a full‑resolution alpha plane.
- `NV12`, `YV12` — planar 4:2:0 YUV formats (NV12 = interleaved UV plane).
- `BGRA` — 32‑bit RGBA (Win32 ARGB32 layout).
- `P216` — planar 4:2:2 YUV with 16‑bit components (Y plane + interleaved UV plane).
- `PA16` — `P216` plus a 16‑bit alpha plane.
- `FPA1` — planar 32‑bit float audio.
- `Unknown(i32)` for non‑standard values.

When receiving uncompressed video, OMT delivers only `UYVY`, `UYVA`, `BGRA`, or `BGRX` (alpha omitted). Other formats may arrive as `VMX1` and can be decoded using `VideoFrame::data(...)`.

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
- `PREMULTIPLIED` — alpha channel is premultiplied (only meaningful when `ALPHA` is set).
- `PREVIEW` — sender emitted a 1/8th preview frame.
- `HIGH_BIT_DEPTH` — set for `P216`/`PA16` sources and for `VMX1` that originated from those formats, so decoders can select the right output format.

### Video data formats (`VideoDataFormat`)

- `RGB` (8-bit/component)
- `RGBAs` (8-bit/component, straight alpha) — alias: `RGBA`
- `RGBAp` (8-bit/component, premultiplied alpha)
- `RGB16` (16-bit/component)
- `RGBAs16` (16-bit/component, straight alpha) — alias: `RGBA16`
- `RGBAp16` (16-bit/component, premultiplied alpha)

---

## Quality selection

Senders select a quality level with `Quality`:

- `Low`, `Medium`, `High`, `Default`

When a sender uses `Default`, it starts at **Medium** and allows receivers to suggest a preferred quality. The sender then chooses the **highest suggested quality** across connected receivers. If a receiver is set to `Default`, it defers to other receivers’ suggestions.

```rust
use libomt::{Receiver, Quality};

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

This wrapper is safe to use as long as you follow these rules:

1. **Frame lifetimes**  
   Frames returned by `Receiver::receive` and `Sender::receive_metadata` are valid only until the next call on the same receiver/sender. Do not store references beyond that.

2. **Data ownership**  
   - Received frame buffers are owned by libomt. Do **not** free them.
   - Outgoing frames own their `Vec<u8>` payload. Keep the `OutgoingFrame` alive until `Sender::send` returns.

3. **FFI zero‑init**  
   When using raw FFI types, zero‑initialize `OMTMediaFrame` before use. The high-level API does this for you.

---

## Examples

This project includes runnable examples under `examples/` that use discovery and direct receive calls:

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
