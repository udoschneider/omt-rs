# Implementation Notes for OMT High-Level Wrapper

## Overview

This document describes the implementation of the high-level, safe Rust wrapper for the OMT (Open Media Transport) C library.

## Architecture

### Layer Structure

```
Application Code
      ↓
High-Level API (omt crate) ← YOU ARE HERE
      ↓
FFI Bindings (omt-sys crate)
      ↓
C Library (libomt)
```

## Modules

### 1. Core Types (`types.rs`)

**Purpose**: Safe Rust representations of C enums and flags.

**Implementation Details**:
- All enums derive from C constants via `omt-sys`
- Bitflag types use newtype pattern for type safety
- Conversion methods (`from_ffi`, `to_ffi`) for FFI boundary
- Logical constants like `FrameType::ALL` for convenience

**Key Types**:
- `FrameType`: Enum with bitwise OR support
- `Codec`: Video/audio codec enumeration
- `Quality`, `ColorSpace`: Simple enums
- `VideoFlags`, `ReceiveFlags`: Bitflag structs
- `PreferredVideoFormat`: Receiver format preferences
- `SenderInfo`: Metadata about sources

### 2. Error Handling (`error.rs`)

**Purpose**: Idiomatic Rust error handling.

**Implementation Details**:
- Uses `thiserror` for derive macro
- All FFI errors converted to Rust Result<T, Error>
- No panics in production code
- Descriptive error messages

**Error Types**:
- FFI-related: `NullPointer`, `NulError`
- Operation-related: `Timeout`, `InvalidFrameType`
- Resource-related: `SenderCreateFailed`, `ReceiverCreateFailed`

### 3. Frame Types (`frame.rs`)

**Purpose**: Safe wrappers for media frames.

**Implementation Details**:
- `MediaFrame`: Base wrapper around FFI struct
- Type-specific wrappers: `VideoFrame`, `AudioFrame`, `MetadataFrame`
- Zero-copy data access via slices
- Lifetime tied to underlying FFI frame (receive only)
- Debug derive for easy inspection

**Safety Considerations**:
- All raw pointer access wrapped in unsafe blocks
- Null checks before dereferencing
- Length validation for slice creation
- Data remains valid until next receive() call

### 4. Receiver (`receiver.rs`)

**Purpose**: Consuming media streams from a sender.

**Implementation Details**:
- RAII: `Drop` trait ensures cleanup
- NonNull pointer for type-level guarantees
- Convenience methods for each frame type
- Statistics and tally support
- Thread-safe (Send + Sync)

**Key Methods**:
- `new()`: Creates and connects
- `receive()`: Generic receive
- `receive_video/audio/metadata()`: Type-specific receives
- `get_*_statistics()`: Performance monitoring

### 5. Sender (`sender.rs`)

**Purpose**: Broadcasting media streams to receivers.

**Implementation Details**:
- RAII: `Drop` trait ensures cleanup
- NonNull pointer for type-level guarantees
- Metadata management
- Connection monitoring
- Thread-safe (Send + Sync)

**Key Methods**:
- `new()`: Creates sender
- `send()`: Transmit frames
- `connections()`: Monitor receivers
- Metadata and redirect support

### 6. Discovery (`discovery.rs`)

**Purpose**: Network source discovery.

**Implementation Details**:
- Simple wrapper around FFI discovery function
- Returns Vec<String> of source addresses
- Handles C string array safely

### 7. Settings (`settings.rs`)

**Purpose**: Library configuration.

**Implementation Details**:
- String and integer settings support
- Convenience methods for common settings
- Process-lifetime configuration

**Common Settings**:
- `DiscoveryServer`: Discovery server URL
- `NetworkPortStart/End`: Port range for senders

### 8. Utility Types

#### Tally (`tally.rs`)
- Preview and program state
- Convenience constructors
- Display implementation

#### Statistics (`statistics.rs`)
- Performance metrics
- Computed properties (averages, rates)
- Duration conversions

#### Codec (`codec.rs`)
- Codec properties (video/audio, alpha, bit depth)
- Helper methods
- FourCC display

## Safety & Design Decisions

### Memory Safety

1. **No manual memory management**: All allocations handled by C library
2. **RAII everywhere**: Drop implementations ensure cleanup
3. **Validated pointers**: NonNull types prevent null dereference
4. **Bounded slices**: All data access through validated slices

### Thread Safety

Both `Sender` and `Receiver` are marked `Send + Sync` because:
- The underlying C library is thread-safe
- All mutable state is behind FFI boundary
- Rust's borrow checker prevents data races

### Error Handling

Philosophy: **No panics in library code**
- All fallible operations return `Result<T, Error>`
- Timeouts return `Option<T>` within Result
- FFI errors converted to typed Rust errors

### Zero-Copy Design

Where possible, data is accessed via slices:
```rust
// Video frame data - no copy
let pixels: &[u8] = frame.data();

// Audio planes - no copy
let channels: Vec<&[f32]> = audio_frame.as_f32_planar();
```

### API Design Principles

1. **Type-driven**: Impossible states are unrepresentable
2. **Ergonomic**: Common operations are concise
3. **Discoverable**: Good documentation and examples
4. **Predictable**: Mirrors C API where appropriate
5. **Safe by default**: Unsafe only where necessary

## FFI Boundary

### C to Rust

**Null pointer handling**:
```rust
NonNull::new(ptr).ok_or(Error::NullPointer)?
```

**C strings to Rust**:
```rust
CStr::from_ptr(ptr).to_str()?
```

**C arrays to Vec**:
```rust
let bytes: Vec<u8> = c_array[..len].iter().map(|&c| c as u8).collect();
```

### Rust to C

**Rust strings to C**:
```rust
let c_str = CString::new(rust_str)?;
omt_sys::function(c_str.as_ptr());
```

**Enums to integers**:
```rust
enum_value.to_ffi() // -> u32
```

**Structs with fixed arrays**:
```rust
// Careful handling of OMT_MAX_STRING_LENGTH arrays
fn string_to_c_array(s: &str, arr: &mut [i8; 1024]) -> Result<()>
```

## Future Enhancements

Potential improvements for future versions:

1. **Frame Builders**: Ergonomic frame construction for sending
2. **Async Support**: Tokio-based async API
3. **Iterator API**: Streaming frame iterators
4. **Batch Operations**: Send multiple frames efficiently
5. **Codec Validation**: Type-level codec guarantees
6. **Connection Events**: Callback/channel for connection changes
7. **Compressed Frame Support**: Full VMX1 handling
8. **Profile Support**: Common use-case profiles (HD, 4K, etc.)

## Testing Strategy

Current test coverage:
- Unit tests for type conversions
- Property tests for flags
- Basic integration tests

Recommended additions:
- Integration tests with actual OMT library
- Fuzz testing for FFI boundary
- Benchmark suite
- Example-based testing

## Documentation

All public items have:
- Doc comments with descriptions
- Parameter documentation
- Return value documentation
- Example code blocks
- Links to related items

## Compliance with Rust Guidelines

This implementation follows:
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Idiomatic Rust patterns
- Zero-cost abstraction principles
- Safety-first approach

## Known Limitations

1. **Frame Creation**: Currently receive-only, sending requires raw FFI access
2. **Blocking API**: All operations are synchronous
3. **Library Dependency**: Requires libomt to be installed
4. **Platform Support**: Depends on underlying C library support

## Maintenance Notes

When updating for new libomt versions:

1. Regenerate FFI bindings in omt-sys
2. Check for new constants/functions
3. Update type mappings if enums change
4. Add tests for new functionality
5. Update documentation
6. Bump version appropriately

## Performance Characteristics

- **Zero-copy**: Frame data accessed directly
- **Minimal allocations**: Most operations stack-allocated
- **Thin wrappers**: ~0 overhead vs C API
- **Thread-safe**: No locks in Rust layer

## Contributing Guidelines

When adding features:

1. Maintain safety guarantees
2. Add comprehensive documentation
3. Include examples
4. Write tests
5. Follow existing patterns
6. No unwrap/expect in library code
7. Use thiserror for errors
8. Keep FFI boundary isolated
