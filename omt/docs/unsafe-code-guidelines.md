# Unsafe Code Guidelines for OMT Rust Wrapper

## Purpose

This document outlines the standards, conventions, and safety requirements for unsafe code in the OMT Rust wrapper. These guidelines ensure that unsafe code is used judiciously, documented thoroughly, and maintains the safety guarantees expected from Rust code.

## 1. Principles of Unsafe Code Usage

### 1.1 Minimal Unsafe
- Use unsafe code only when strictly necessary (FFI, performance optimizations backed by benchmarks)
- Prefer safe abstractions over direct unsafe operations
- Encapsulate unsafe code in safe interfaces

### 1.2 Documentation First
- Every unsafe block MUST include a `// SAFETY:` comment
- Document all invariants, preconditions, and postconditions
- Explain why the operation is safe and what the caller must uphold

### 1.3 Defense in Depth
- Validate inputs before unsafe operations
- Use Rust's type system to enforce invariants where possible
- Implement runtime checks for conditions that can't be checked at compile time

## 2. FFI Safety Requirements

### 2.1 C Function Calls
```rust
// GOOD - Properly documented
// SAFETY: omt_receive_create returns a valid pointer or null.
// The C library guarantees the pointer remains valid until omt_receive_destroy is called.
let handle = unsafe {
    omt_sys::omt_receive_create(
        c_address.as_ptr(),
        frame_types.to_ffi(),
        format.to_ffi(),
        flags.to_ffi(),
    )
};

// BAD - Missing safety comment
let handle = unsafe {
    omt_sys::omt_receive_create(...)
};
```

### 2.2 Pointer Validation
- Check for null pointers before dereferencing
- Validate pointer alignment requirements
- Ensure lifetime constraints are properly encoded in types

### 2.3 Memory Management
- Use RAII (`Drop` implementations) for C resources
- Prefer `NonNull` over raw pointers for non-null handles
- Document ownership transfer across FFI boundaries

## 3. Lifetime Safety

### 3.1 Frame Data Lifetime
```rust
// GOOD - Lifetime properly documented
// SAFETY: The C API guarantees the frame data is valid until the next call to omt_receive.
// The lifetime bound to &self ensures the frame cannot outlive this receiver instance
// and cannot be used after the next receive() call (due to &self borrow).
Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })

// BAD - Missing lifetime documentation
Ok(unsafe { MediaFrame::from_ffi_ptr(ptr) })
```

### 3.2 Borrowed Data
- Use lifetime parameters to encode C API guarantees
- Document the source of lifetime constraints (C library documentation)
- Ensure borrowed data doesn't outlive its source

## 4. Thread Safety

### 4.1 Send/Sync Implementations
```rust
// GOOD - Documented thread safety assumption
// SAFETY: The underlying C library is thread-safe as documented.
// Different handles can be used concurrently, and the library provides
// internal synchronization for shared resources.
unsafe impl Send for Receiver {}
unsafe impl Sync for Receiver {}

// BAD - Missing justification
unsafe impl Send for Receiver {}
```

### 4.2 Concurrent Access
- Document which operations are thread-safe
- Specify synchronization requirements for shared instances
- Use Rust's type system to prevent unsafe concurrent access

## 5. Memory Safety

### 5.1 Slice Creation
```rust
// GOOD - Comprehensive safety documentation
// SAFETY: The lifetime 'a ensures this slice cannot outlive the source data.
// The C API guarantees Data is valid for the frame's lifetime.
// DataLength accurately reflects the allocated memory size.
unsafe {
    slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
}

// BAD - Insufficient documentation
unsafe {
    slice::from_raw_parts(self.ffi.Data as *const u8, self.ffi.DataLength as usize)
}
```

### 5.2 Buffer Management
- Validate buffer sizes before unsafe operations
- Use Rust's bounds checking where possible
- Document buffer ownership and mutation rights

## 6. Error Handling in Unsafe Code

### 6.1 Error Recovery
- Unsafe operations should handle errors gracefully
- Convert C error codes to Rust Result types
- Ensure resources are properly cleaned up on error

### 6.2 Panic Safety
- Avoid panics in unsafe code when possible
- Document panic conditions if they exist
- Ensure no resource leaks on panic

## 7. Testing Unsafe Code

### 7.1 Unit Tests
- Test safe wrappers around unsafe code
- Validate error handling paths
- Test edge cases and boundary conditions

### 7.2 Integration Tests
- Test end-to-end scenarios involving unsafe code
- Verify memory safety through valgrind or similar tools
- Test concurrent access patterns

### 7.3 Compile-Time Tests
- Use compile_fail tests to verify lifetime constraints
- Test that unsafe patterns can't be misused
- Verify type system constraints

## 8. Code Review Requirements

### 8.1 Safety Review Checklist
For every unsafe block, reviewers must verify:
- [ ] Safety comment exists and is comprehensive
- [ ] All invariants are documented
- [ ] Input validation is performed
- [ ] Lifetime constraints are properly encoded
- [ ] Error handling is appropriate
- [ ] Thread safety is considered
- [ ] No undefined behavior is possible

### 8.2 Documentation Review
- Safety comments must explain WHY the code is safe
- Document assumptions about C library behavior
- Include references to C library documentation when available

## 9. Common Patterns and Anti-Patterns

### 9.1 Good Patterns
```rust
// Pattern: Safe wrapper around unsafe FFI
impl Receiver {
    pub fn new(...) -> Result<Self> {
        let handle = unsafe { /* FFI call with validation */ };
        // Validate and wrap in safe type
    }
}

// Pattern: Lifetime-bound borrowed data
pub struct MediaFrame<'a> {
    ffi: omt_sys::OMTMediaFrame,
    _marker: PhantomData<&'a ()>, // Enforces lifetime
}
```

### 9.2 Anti-Patterns to Avoid
```rust
// ANTI-PATTERN: Unsafe without validation
unsafe {
    let data = *ptr; // No null check!
}

// ANTI-PATTERN: Missing lifetime constraints
fn get_data(&self) -> &[u8] {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
    // No lifetime connection to self!
}

// ANTI-PATTERN: Undocumented thread safety
unsafe impl Send for MyType {} // Why is this safe?
```

## 10. Maintenance and Evolution

### 10.1 Version Compatibility
- Document unsafe API stability guarantees
- Consider unsafe code part of the public API
- Use deprecation warnings for unsafe pattern changes

### 10.2 Security Updates
- Monitor C library security advisories
- Update unsafe code when C library behavior changes
- Document security implications of unsafe operations

### 10.3 Performance Considerations
- Benchmark unsafe optimizations before implementation
- Document performance/security tradeoffs
- Consider safe alternatives with acceptable performance

## 11. Emergency Procedures

### 11.1 Suspected Safety Violations
1. Immediately document the suspected issue
2. Add additional validation if possible
3. Consider disabling the unsafe code path
4. Report to maintainers with full context

### 11.2 C Library Changes
1. Review all unsafe code that interacts with changed C functions
2. Update safety documentation to reflect new behavior
3. Add tests for new edge cases
4. Consider API changes if safety guarantees are affected

## 12. References

- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [The Rust Reference - Unsafe Operations](https://doc.rust-lang.org/reference/unsafety.html)
- [Rust API Guidelines - FFI](https://rust-lang.github.io/api-guidelines/ffi.html)
- [libomt Documentation](https://github.com/openmediatransport/libomt)

---

*These guidelines are based on the AGENTS.md requirements and Rust best practices.*  
*Last Updated: $(date)*  
*OMT Version: $(crate_version)*