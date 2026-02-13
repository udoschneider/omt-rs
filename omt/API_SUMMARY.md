# OMT High-Level Wrapper API Summary

This document summarizes the high-level, safe Rust API created for the OMT library.

## Module Structure

```
omt/
├── lib.rs           - Main crate with re-exports and top-level functions
├── codec.rs         - Codec type with helper methods
├── discovery.rs     - Network discovery
├── error.rs         - Error types and Result alias
├── frame.rs         - Media frame types (VideoFrame, AudioFrame, MetadataFrame)
├── receiver.rs      - Receiver for consuming media streams
├── sender.rs        - Sender for broadcasting media streams
├── settings.rs      - Configuration management
├── statistics.rs    - Performance statistics
├── tally.rs         - Tally state management
└── types.rs         - Core types and enumerations
```

## Public API

### Core Types

#### `Receiver`
- `new()` - Create and connect to a source
- `receive()` - Receive any frame type
- `receive_video()` - Receive video frame
- `receive_audio()` - Receive audio frame
- `receive_metadata()` - Receive metadata frame
- `set_tally()` - Set tally state
- `get_tally()` - Get current tally state
- `set_flags()` - Change receive flags
- `set_suggested_quality()` - Suggest encoding quality
- `get_sender_information()` - Get sender info
- `get_video_statistics()` - Get video stats
- `get_audio_statistics()` - Get audio stats

#### `Sender`
- `new()` - Create a sender
- `set_sender_information()` - Set sender metadata
- `add_connection_metadata()` - Add connection metadata
- `clear_connection_metadata()` - Clear connection metadata
- `set_redirect()` - Set redirect address
- `get_address()` - Get discovery address
- `send()` - Send a frame
- `connections()` - Get connection count
- `receive_metadata()` - Receive metadata from receivers
- `get_tally()` - Get current tally state
- `get_video_statistics()` - Get video stats
- `get_audio_statistics()` - Get audio stats

#### `VideoFrame`
- `width()`, `height()` - Frame dimensions
- `stride()` - Row pitch in bytes
- `flags()` - Video flags
- `frame_rate_numerator()`, `frame_rate_denominator()` - Frame rate components
- `frame_rate()` - Frame rate as float
- `aspect_ratio()` - Display aspect ratio
- `color_space()` - Color space
- `codec()` - Video codec
- `timestamp()` - Presentation timestamp
- `data()` - Pixel data
- `compressed_data()` - Compressed VMX1 data (if available)
- `frame_metadata()` - Per-frame metadata

#### `AudioFrame`
- `sample_rate()` - Sample rate (e.g., 48000 Hz)
- `channels()` - Number of channels
- `samples_per_channel()` - Samples per channel
- `codec()` - Audio codec (FPA1)
- `timestamp()` - Presentation timestamp
- `data()` - Raw audio data
- `as_f32_planar()` - Audio as f32 slices per channel

#### `MetadataFrame`
- `timestamp()` - Timestamp
- `data()` - Raw UTF-8 data
- `as_str()` - Metadata as string slice

### Enumerations

#### `FrameType`
- `None`, `Metadata`, `Video`, `Audio`
- Supports bitwise OR for combining types
- Constants: `ALL`, `VIDEO_AUDIO`

#### `Codec`
Video codecs:
- `Vmx1` - Fast video codec
- `Uyvy` - 16bpp YUV 4:2:2
- `Yuy2` - 16bpp YUV 4:2:2 (YUYV order)
- `Bgra` - 32bpp RGBA
- `Nv12` - Planar 4:2:0 YUV
- `Yv12` - Planar 4:2:0 YUV
- `Uyva` - UYVY with alpha
- `P216` - Planar 4:2:2 16-bit YUV
- `Pa16` - P216 with alpha

Audio codecs:
- `Fpa1` - 32-bit float planar audio

Helper methods:
- `is_video()`, `is_audio()`
- `supports_alpha()`
- `is_high_bit_depth()`
- `is_compressed()`
- `bits_per_pixel()`
- `fourcc()`

#### `Quality`
- `Default` - Allow receiver suggestions
- `Low`, `Medium`, `High`

#### `ColorSpace`
- `Undefined` - Automatic selection
- `Bt601` - ITU-R BT.601 (SD)
- `Bt709` - ITU-R BT.709 (HD)

#### `VideoFlags`
Bitflags:
- `NONE`
- `INTERLACED`
- `ALPHA`
- `PRE_MULTIPLIED`
- `PREVIEW`
- `HIGH_BIT_DEPTH`

#### `PreferredVideoFormat`
- `Uyvy` - Always UYVY
- `UyvyOrBgra` - UYVY or BGRA (when alpha)
- `Bgra` - Always BGRA
- `UyvyOrUyva` - UYVY or UYVA (when alpha)
- `UyvyOrUyvaOrP216OrPa16` - Auto based on sender
- `P216` - Always P216

#### `ReceiveFlags`
Bitflags:
- `NONE`
- `PREVIEW` - 1/8th preview only
- `INCLUDE_COMPRESSED` - Include VMX1 data
- `COMPRESSED_ONLY` - No decoding

### Utility Types

#### `Tally`
- `new()` - Create tally state
- `off()`, `preview_only()`, `program_only()`
- `is_active()`, `is_off()`
- Fields: `preview`, `program` (bool)

#### `SenderInfo`
- `new()` - Create sender info
- Fields: `product_name`, `manufacturer`, `version` (String)

#### `Statistics`
- Fields: 
  - `bytes_sent`, `bytes_received`
  - `bytes_sent_since_last`, `bytes_received_since_last`
  - `frames`, `frames_since_last`, `frames_dropped`
  - `codec_time`, `codec_time_since_last`
- Methods:
  - `codec_duration()` - As Duration
  - `average_codec_time_ms()` - Average per frame
  - `total_bytes()` - Total transferred
  - `drop_rate()` - Frame drop percentage

#### `Discovery`
- `get_addresses()` - List available sources

#### `Settings`
- `get_string()`, `set_string()`
- `get_integer()`, `set_integer()`
- Convenience methods:
  - `discovery_server()`, `set_discovery_server()`
  - `network_port_start()`, `set_network_port_start()`
  - `network_port_end()`, `set_network_port_end()`
  - `set_logging_filename()` - Configure or disable logging to file

### Top-Level Functions



### Error Handling

#### `Error` enum
- `NullPointer`
- `NulError` - String contains null byte
- `InvalidUtf8`
- `Timeout`
- `SenderCreateFailed`
- `ReceiverCreateFailed`
- `InvalidFrameType`
- `InvalidCodec`
- `BufferTooSmall`
- `Other` - Custom message

#### `Result<T>` type alias
Convenience alias for `std::result::Result<T, Error>`

## Safety Guarantees

1. **Memory Safety**: All FFI calls are wrapped safely, no undefined behavior
2. **Resource Management**: RAII ensures proper cleanup (Drop implementations)
3. **Thread Safety**: `Sender` and `Receiver` are `Send + Sync`
4. **Type Safety**: Strong typing prevents misuse of APIs
5. **No Panics**: No unwrap/expect in production code (Result-based error handling)

## Example Usage Patterns

### Basic Receiving
```rust
let receiver = Receiver::new(addr, FrameType::Video, format, flags)?;
while let Some(frame) = receiver.receive_video(1000)? {
    // Process frame
}
```

### Multi-threaded Receiving
```rust
let receiver = Arc::new(Receiver::new(/* ... */)?);
let r1 = receiver.clone();
let r2 = receiver.clone();

thread::spawn(move || { /* video */ });
thread::spawn(move || { /* audio */ });
```

### Sending with Monitoring
```rust
let sender = Sender::new("Source", Quality::High)?;
sender.set_sender_information(&info)?;

loop {
    sender.send(&frame)?;
    let stats = sender.get_video_statistics();
    // Monitor stats
}
```

## Design Principles

1. **Zero-cost abstractions**: Thin wrappers over FFI with minimal overhead
2. **Rust idioms**: Builder patterns, Result types, iterators where appropriate
3. **Documentation**: Comprehensive doc comments with examples
4. **Safety first**: All unsafe code is isolated and documented
5. **Ergonomics**: Convenient methods and sensible defaults
6. **Compatibility**: Direct mapping to C API for predictability
