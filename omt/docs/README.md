# OMT Documentation

This directory contains comprehensive documentation for the OMT Rust wrapper.

## Documentation Files

### Primary Documentation

- **[unsafe-areas.md](unsafe-areas.md)** - Comprehensive documentation about unsafe areas in the high-level wrapper not covered by lifetimes and compile-time guarantees
- **[unsafe-summary.md](unsafe-summary.md)** - Quick reference summary of unsafe areas and safety guidelines

## Documentation Purpose

The documentation in this directory focuses on:

1. **Safety Boundaries** - Areas where Rust's safety guarantees don't apply
2. **FFI Integration** - Interaction with the underlying C library (`libomt`)
3. **Memory Management** - How memory is managed across the FFI boundary
4. **Thread Safety** - Concurrency considerations and guarantees
5. **Error Handling** - Recovery from C library errors
6. **Best Practices** - Safe usage patterns and common pitfalls

## Target Audience

- **Library Users** - Understanding safety constraints when using the OMT wrapper
- **Contributors** - Knowing where unsafe code exists and why
- **Security Auditors** - Identifying potential safety issues
- **Integrators** - Building safe systems on top of OMT

## Related Documentation

- **API Documentation**: `cargo doc --open` for auto-generated API docs
- **Examples**: See `examples/` directory for usage patterns
- **C Library Docs**: Refer to `omt-sys/libomt.h` for C API details

## Contributing to Documentation

When updating documentation:

1. Keep safety annotations in sync with code changes
2. Update both detailed and summary documents
3. Add examples for new unsafe patterns
4. Document any changes to safety contracts

## Safety Philosophy

The OMT wrapper follows these safety principles:

1. **Minimize Unsafe** - Use Rust's type system where possible
2. **Document Assumptions** - Clearly state safety requirements
3. **Validate Inputs** - Check parameters before FFI calls
4. **Provide Safe Abstractions** - Hide unsafe details from users
5. **Enable Testing** - Make unsafe boundaries testable

## Getting Help

For questions about safety or unsafe usage:

1. Review the documentation in this directory
2. Check the examples for safe patterns
3. Examine the source code safety annotations
4. Contact the maintainers for clarification

---
*Documentation Version: 1.0*  
*Last Updated: $(date)*