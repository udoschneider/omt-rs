# Frame Builder Feature Summary

## Overview

This document summarizes the new frame builder feature that enables creating and sending owned media frames in the OMT Rust library.

## Problem Statement

Previously, the codebase only supported receiving (borrowed) frames. There was no way to:
- Create new frames from scratch
- Set frame properties programmatically
- Send frames through the sender

## Solution

Implemented a comprehensive frame building system with three builder types:

### 1. VideoFrameBuilder
Creates video frames with full control over properties:
- Codec selection (UYVY, BGRA, VMX1, etc.)
- Dimensions and stride
- Frame rate and aspect ratio
- Video flags (interlaced, alpha, premultiplied, etc.)
- Color space (BT.601, BT.709)
- Timestamps
- Per-frame metadata

### 2. AudioFrameBuilder
Creates audio frames in FPA1 format:
- Sample rate configuration
- Channel count (1-32 channels)
- Planar 32-bit float audio data
- Samples per channel
- Timestamps
- Per-frame metadata

### 3. MetadataFrameBuilder
Creates metadata frames:
- UTF-8 encoded XML metadata
- Timestamps

## Key Components

### OwnedMediaFrame
A new type that owns all frame data and manages memory safely:
- Stores all frame properties
- Owns the data buffer and metadata
- Can be converted to `MediaFrame` for sending
- Implements `Send` and `Sync` for thread safety

### MediaFrame Enhancement
Added `from_owned_ffi()` method to support creating borrowed frames from owned data.

### Error Handling
Added `InvalidParameter` error variant for builder validation errors.

## Usage Example

```rust
use omt::{Sender, Quality, VideoFrameBuilder, Codec};

// Create sender
let sender = Sender::new("My Source", Quality::High)?;

// Create video frame
let data = vec![0u8; 1920 * 1080 * 2]; // UYVY data
let frame = VideoFrameBuilder::new()
    .codec(Codec::Uyvy)
    .dimensions(1920, 1080)
    .stride(1920 * 2)
    .frame_rate(30, 1)
    .aspect_ratio(16.0 / 9.0)
    .data(data)
    .build()?;

// Send the frame
sender.send(&frame.as_media_frame())?;
```

## Files Added

1. **omt/src/frame_builder.rs** (577 lines)
   - `VideoFrameBuilder` - Video frame builder
   - `AudioFrameBuilder` - Audio frame builder
   - `MetadataFrameBuilder` - Metadata frame builder
   - `OwnedMediaFrame` - Owned frame type

2. **omt/examples/send_frames.rs** (207 lines)
   - Complete example demonstrating frame creation and sending
   - Shows video, audio, and metadata frame generation
   - Includes test pattern generation

3. **omt/tests/frame_builder_tests.rs** (411 lines)
   - 22 comprehensive integration tests
   - Tests all builder types and error cases
   - Validates frame properties and data access

4. **docs/FRAME_BUILDERS.md** (462 lines)
   - Complete documentation for frame builders
   - Usage examples for all frame types
   - Best practices and performance tips
   - Thread safety guidelines

## Files Modified

1. **omt/src/frame.rs**
   - Added `from_owned_ffi()` method for frame builders

2. **omt/src/error.rs**
   - Added `InvalidParameter` error variant

3. **omt/src/lib.rs**
   - Added frame_builder module
   - Exported builder types: `VideoFrameBuilder`, `AudioFrameBuilder`, `MetadataFrameBuilder`, `OwnedMediaFrame`

## Testing

All tests pass successfully:
- 22 integration tests for frame builders
- Tests cover all builder types
- Error cases validated
- Frame property access tested
- Media frame conversion tested

Build status:
- ✅ `cargo build` - Success
- ✅ `cargo build --release` - Success
- ✅ `cargo build --examples` - Success
- ✅ `cargo test --test frame_builder_tests` - 22 tests passed

## Features

### Video Frames
- Support for all major codecs (UYVY, BGRA, VMX1, NV12, YV12, P216, etc.)
- Automatic stride calculation based on codec
- Full control over video flags
- Color space selection
- Frame rate as numerator/denominator
- Aspect ratio support

### Audio Frames
- 32-bit floating-point planar audio (FPA1)
- 1-32 channels supported
- Configurable sample rate
- Proper planar data layout validation

### Metadata Frames
- UTF-8 encoded strings
- Automatic null termination
- Suitable for XML metadata

### Common Features
- Timestamp support (auto-generation or manual)
- Per-frame metadata (up to 65536 bytes)
- Builder pattern for ergonomic API
- Comprehensive error validation
- Memory safety guarantees

## Memory Management

- All frame data is owned by `OwnedMediaFrame`
- Borrowed `MediaFrame` references the owned data safely
- No manual memory management required
- Automatic cleanup on drop
- Thread-safe with `Send` and `Sync` implementations

## Performance Considerations

- Data buffers should be pre-allocated and reused
- Builder validation happens at build time
- Zero-copy conversion to `MediaFrame` for sending
- Efficient planar audio layout

## API Design

Following Rust best practices:
- Builder pattern for optional parameters
- Type safety for all parameters
- Compile-time validation where possible
- Clear error messages
- Idiomatic Rust API

## Compliance

The implementation follows all project rules:
- ✅ Formatted with `rustfmt`
- ✅ No `clippy` warnings
- ✅ All public APIs documented
- ✅ Examples provided
- ✅ Integration tests included
- ✅ No `unwrap()` in production code
- ✅ Proper error handling
- ✅ Thread-safe implementations

## Future Enhancements

Potential improvements:
1. Support for frame pooling/reuse
2. Async frame generation support
3. Frame validation helpers
4. Codec-specific builders with type safety
5. Frame conversion utilities

## Documentation

Complete documentation available in:
- **docs/FRAME_BUILDERS.md** - Comprehensive guide with examples
- API docs in source code
- Example program: `examples/send_frames.rs`
- Integration tests: `tests/frame_builder_tests.rs`

## Conclusion

The frame builder feature provides a complete, safe, and ergonomic API for creating and sending media frames. It enables full sender functionality with proper memory management, comprehensive error handling, and excellent developer experience.