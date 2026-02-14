# Unsafe Areas Summary - OMT High-Level Wrapper

## Quick Reference

### 1. FFI Boundary (Most Critical)
- **All C function calls** are unsafe blocks
- **Raw pointer dereferencing** from C API returns
- **Slice creation** from raw pointers in frame data
- **Thread safety assertions** (`unsafe impl Send/Sync`)

### 2. Memory Safety Boundaries
- **Frame data lifetime**: Valid only until next receive call
- **Discovery memory**: Potential leaks in C library
- **String conversions**: C strings to Rust `String` with UTF-8 validation
- **Handle management**: `NonNull` wrappers for C library handles

### 3. Lifetime Constraints
- `MediaFrame<'a>`: Lifetime tied to source (`Receiver`/`Sender`)
- **Cannot store frames** beyond next API call on same instance
- **Owned frames** (`OwnedMediaFrame`) for long-term storage
- **Borrow checker** prevents use-after-invalidation

### 4. Thread Safety Assumptions
- **Documented as thread-safe** but not verified
- **Different instances** can be used concurrently
- **Same instance** requires external synchronization
- **Statistics/tally**: `Copy` types, safe to share

## Safety Annotations in Code

### Receiver/Sender Creation
```rust
// SAFETY: C API returns valid pointer or null
let handle = unsafe { omt_sys::omt_receive_create(...) };
```

### Frame Data Access
```rust
// SAFETY: Data valid for lifetime 'a, C API guarantees
unsafe { slice::from_raw_parts(self.ffi.Data as *const u8, ...) }
```

### Thread Safety
```rust
// SAFETY: Underlying C library is thread-safe
unsafe impl Send for Receiver {}
unsafe impl Sync for Receiver {}
```

## Critical Safety Contracts

### 1. Frame Lifetime Contract
- **Source**: C library documentation
- **Guarantee**: Frame data valid until next `receive()`/`send()` call
- **Enforcement**: Lifetime parameters, `&self` borrows
- **Risk**: C library could violate contract

### 2. Thread Safety Contract
- **Source**: C library documentation
- **Guarantee**: Handles can be used across threads
- **Enforcement**: `Send`/`Sync` implementations
- **Risk**: Documentation may be incorrect

### 3. Memory Management Contract
- **Source**: C library implementation
- **Guarantee**: Proper cleanup in `*_destroy()` functions
- **Enforcement**: RAII (`Drop` implementations)
- **Risk**: Memory leaks, double-free issues

## Common Pitfalls to Avoid

### ❌ DON'T
```rust
// Store received frames
let saved_frame = receiver.receive(...)?.unwrap();
// Later... UNSAFE: Frame invalidated
process(&saved_frame);

// Share receiver between threads without sync
let receiver = Arc::new(receiver);
// Concurrent receives: UNSAFE
```

### ✅ DO
```rust
// Process immediately
if let Some(frame) = receiver.receive(...)? {
    process(&frame);
    // Frame dropped here
}

// Use owned frames for storage
let owned = frame_builder.build()?;
store_frame(owned.as_media_frame());

// One receiver per thread
let receiver_clone = Receiver::new(...)?;
thread::spawn(move || process_receiver(receiver_clone));
```

## Error Recovery Guidelines

### Recoverable Errors
- **Timeout**: Retry or reconnect
- **Invalid parameters**: Validate and retry
- **Network issues**: Re-establish connection

### Fatal Errors
- **Null pointer returns**: C library bug, cannot recover
- **Memory corruption**: Undefined behavior, abort
- **Thread safety violations**: Data races, unpredictable

## Testing Recommendations

### Unit Tests
- Validate type conversions
- Test error handling paths
- Verify lifetime constraints

### Integration Tests
- End-to-end sender/receiver communication
- Concurrent access patterns
- Memory usage under load

### Fuzz Testing (Recommended)
- Random frame data inputs
- Malformed C strings
- Concurrent API calls

## Version Compatibility Notes

### C Library Dependencies
- **libomt version**: Assumes ABI stability
- **Platform differences**: macOS/Linux/Windows behavior
- **Build configuration**: Feature flags affect safety

### Rust Wrapper Versioning
- **Semantic versioning**: Breaking changes in major versions
- **Unsafe contract**: Part of public API stability
- **Deprecation**: Old unsafe patterns marked deprecated

## Emergency Procedures

### Suspected Memory Corruption
1. **Immediately stop** all OMT operations
2. **Do not attempt** to free resources
3. **Restart application** to clear C library state
4. **Enable logging** to capture details

### Thread Safety Violations
1. **Isolate** offending thread
2. **Add synchronization** barriers
3. **Consider single-threaded** mode if issues persist
4. **Report bug** to library maintainers

### Frame Lifetime Issues
1. **Switch to `OwnedMediaFrame`**
2. **Implement frame pooling**
3. **Reduce frame retention time**
4. **Monitor C library memory usage**

---

## Quick Safety Checklist

Before deploying OMT in production:

- [ ] Understand frame lifetime constraints
- [ ] Implement proper error recovery
- [ ] Test concurrent access patterns
- [ ] Monitor memory usage over time
- [ ] Have rollback plan for C library issues
- [ ] Document unsafe usage in codebase
- [ ] Train team on safety boundaries
- [ ] Establish monitoring for safety violations

---

*Reference: Full documentation in `unsafe-areas.md`*  
*OMT Version: Check Cargo.toml*  
*Last Updated: $(date)*