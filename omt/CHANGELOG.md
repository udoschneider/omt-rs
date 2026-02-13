# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Initial Release)
- High-level safe Rust wrapper for OMT library
- `Receiver` type for consuming media streams
- `Sender` type for broadcasting media streams
- `VideoFrame`, `AudioFrame`, `MetadataFrame` types
- `Discovery` for network source discovery
- `Settings` for library configuration
- `Statistics` for performance monitoring
- `Tally` for tally state management
- Comprehensive type system:
  - `FrameType`, `Codec`, `Quality`, `ColorSpace`
  - `VideoFlags`, `ReceiveFlags`, `PreferredVideoFormat`
  - `SenderInfo` for source metadata
- Error handling with `Error` enum and `Result` type
- Complete documentation with examples
- Three working examples (discovery, receiver, sender)

### Documentation
- README.md with quick start guide and API overview
- API_SUMMARY.md with complete API reference
- IMPLEMENTATION_NOTES.md with technical details
- LINKS.md with project links
- SUMMARY.md with project overview
- Inline documentation for all public APIs

### Fixed
- Corrected repository URLs to point to openmediatransport organization
  - Main organization: https://github.com/openmediatransport
  - C library: https://github.com/openmediatransport/libomt
- Fixed all documentation references

### Project Links
- Organization: https://github.com/openmediatransport
- libomt (C library): https://github.com/openmediatransport/libomt
- Rust bindings: omt-rs (this repository)

## [0.1.0] - TBD

Initial release of the high-level Rust wrapper for OMT.

[Unreleased]: https://github.com/openmediatransport/omt-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/openmediatransport/omt-rs/releases/tag/v0.1.0
