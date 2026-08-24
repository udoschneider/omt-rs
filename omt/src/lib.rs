//! High-level safe Rust bindings for the Open Media Transport (OMT) library.
//!
//! **Note:** This is an **unofficial, third-party** Rust wrapper. It is not affiliated
//! with or endorsed by the Open Media Transport project.
//!
//! This crate provides idiomatic Rust wrappers around the low-level C bindings
//! in the `omt-sys` crate. It offers safe, ergonomic APIs for sending and receiving
//! media frames over the network using the OMT protocol.
//!
//! The underlying C library is [libomt](https://github.com/openmediatransport/libomt).
//!
//! # Overview
//!
//! OMT is a protocol for low-latency transmission of video, audio, and metadata
//! over IP networks. This crate provides:
//!
//! - Type-safe enums and structs for media types, codecs, and flags
//! - RAII-based sender and receiver types with automatic resource cleanup
//! - Zero-copy access to frame data where possible
//! - Discovery of available sources on the network
//!
//! # Examples
//!
//! ## Creating a Sender
//!
//! ```no_run
//! use omt::{Sender, Quality};
//!
//! let sender = Sender::new("My Source", Quality::High)?;
//! // Send frames...
//! # Ok::<(), omt::Error>(())
//! ```
//!
//! ## Creating a Receiver
//!
//! ```no_run
//! use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
//!
//! let mut receiver = Receiver::new(
//!     "omt://hostname:6400",
//!     FrameType::VIDEO | FrameType::AUDIO,
//!     PreferredVideoFormat::Uyvy,
//!     ReceiveFlags::NONE,
//! )?;
//!
//! // Receive frames...
//! if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
//!     // Process frame...
//! }
//! # Ok::<(), omt::Error>(())
//! ```
//!
//! ## Converting Video Frames
//!
//! ```no_run
//! use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};
//!
//! let mut receiver = Receiver::new(
//!     "omt://hostname:6400",
//!     FrameType::VIDEO,
//!     PreferredVideoFormat::Uyvy,
//!     ReceiveFlags::NONE,
//! )?;
//!
//! if let Some(frame) = receiver.receive(FrameType::VIDEO, 1000)? {
//!     // Convert to RGB8
//!     if let Some(rgb_pixels) = frame.to_rgb8() {
//!         // Process RGB8 pixels...
//!     }
//!
//!     // Convert to RGBA8
//!     if let Some(rgba_pixels) = frame.to_rgba8() {
//!         // Process RGBA8 pixels...
//!     }
//! }
//! # Ok::<(), omt::Error>(())
//! ```
//!
//! ## Configuring Settings
//!
//! ```no_run
//! use omt::Settings;
//!
//! // Configure discovery server
//! Settings::set_discovery_server("omt://server:6400")?;
//!
//! // Configure network port range
//! Settings::set_network_port_start(7000);
//! Settings::set_network_port_end(7200);
//!
//! // Configure logging
//! Settings::set_logging_filename(Some("/var/log/omt.log"));
//! # Ok::<(), omt::Error>(())
//! ```
//!
//! # Runtime requirements
//!
//! This crate is a thin wrapper, but the native stack behind it has a larger
//! footprint than a typical C library. At runtime you need:
//!
//! - **`libomt`** (`.dylib` / `.dll` / `.so`) — the OMT C library this crate
//!   links against. It embeds a .NET (Native-AOT) implementation, so it is
//!   self-contained but sizeable. On macOS and Windows the build script
//!   downloads and links a pinned prebuilt copy (see the workspace README); on
//!   Linux it must be built from source. It cannot be statically linked into
//!   your binary.
//! - **Avahi** (`libavahi-client`, `avahi-daemon`) — required for network
//!   discovery ([`Discovery`]) via mDNS. Sending/receiving to a known address
//!   works without it; enumerating sources does not.
//! - **`libvmx`** — the VMX video codec, loaded lazily via `dlopen` only when
//!   encoding or decoding compressed (`Vmx1`) video. It must sit on the loader
//!   path next to your application. It is *not* needed for uncompressed formats
//!   or for the CPU-side RGB conversions in this crate (those use the pure-Rust
//!   `yuv` crate).
//!
//! On macOS and Windows the `libomt` (and `libvmx`) binaries are co-located in
//! the download cache, and the crate embeds an rpath (macOS) so tests and
//! examples resolve them at runtime. To ship an application, copy
//! `omt_sys::OMT_LIB_FILE` (and the `libvmx` binary) next to your executable,
//! or into your application bundle.
//!
//! # Safety assumptions about the `libomt` C library
//!
//! Every `unsafe` block in this crate ultimately rests on a small set of
//! behavioral guarantees from the underlying `libomt` C library. Some are
//! documented in the C header (`omt-sys/libomt.h`); others are *inferred* from
//! the upstream implementation (`libomtnet`) and are **not** promised by a
//! stable ABI. They are collected here as the single source of truth — the
//! individual `// SAFETY:` comments throughout the crate refer back to this
//! list by number.
//!
//! These were validated against **libomt v1.0.0.16** (the version of the header
//! vendored at `omt-sys/libomt.h`). Treat the list as a checklist to re-verify
//! whenever that header is bumped: a silent change to any of these can turn
//! currently-sound code into undefined behavior with no compiler error. The
//! `debug_assert!` canaries in the receive path (see `frame::MediaFrame`) exist
//! to catch the most dangerous of these regressions during testing rather than
//! in production.
//!
//! 1. **Received frame buffers stay valid until the next receive of the same
//!    type on the same instance.** *Documented* in `libomt.h`: the returned
//!    `OMTMediaFrame` data "is valid until the next call to `omt_receive` for
//!    this instance and frameType" and "pointers do not need to be freed by the
//!    caller". This underpins the borrowed [`MediaFrame`] lifetime and the
//!    `&mut self` receive API.
//! 2. **That invalidation is synchronous and single-slot.** *Inferred* from
//!    `libomtnet` (`OMTReceive.cs`): the library holds one `lastVideoFrame` /
//!    `lastAudioFrame` slot per instance and recycles it *inside* the next
//!    receive call, never asynchronously from a background thread. This is what
//!    makes moving a borrowed frame across threads ([`MediaFrame`] is `Send`)
//!    sound.
//! 3. **Concurrent receive calls on one instance are NOT internally locked.**
//!    *Inferred*: the marshalling layer frees and reallocates the frame buffer
//!    without synchronization, so two overlapping receive calls on the same
//!    instance race and can double-free. Hence the safe API takes `&mut self`,
//!    and [`Receiver::receive_unchecked`] makes serialization a caller
//!    obligation.
//! 4. **`omt_send` / `omt_receive_send` only read the frame.** *Confirmed* in
//!    `libomt` (`libomt.cs`): both copy the frame in via
//!    `OMTMediaFrame.FromIntPtr` and operate on that managed copy, never writing
//!    back through the pointer — so [`Sender::send`] casting a shared
//!    `&MediaFrame` to `*mut` is not an aliasing violation. Not guaranteed by
//!    the C ABI, so re-check on upgrade.
//! 5. **The C API catches its own internal errors and reports "no frame" as a
//!    null return.** *Documented behavior*: there is no separate error channel,
//!    so `Ok(None)` from a receive cannot distinguish a timeout from an internal
//!    failure.
//! 6. **Other `&self` receiver/sender methods are internally synchronized.**
//!    *Partially inferred* — the weakest assumption here, and the one most worth
//!    auditing on an upgrade; it underpins the `Sync` impls on [`Receiver`] and
//!    [`Sender`].
//! 7. **The discovery address list is process-global, not per-instance.**
//!    *Documented* in `libomt.h`: the `char**` from
//!    `omt_discovery_getaddresses` "is valid until the next call to
//!    getaddresses". There is no handle to scope that to, so unlike the receive
//!    path it cannot be serialized by a `&mut self` borrow;
//!    [`Discovery::get_addresses`] holds an internal mutex across both the call
//!    and the copying of the strings instead. This assumption is only about
//!    *this* crate's calls — another library calling the same C function
//!    directly in the same process would still race.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod c_string;
mod discovery;
mod error;
mod frame;

/// Fuzzing hooks for this crate's untrusted-input handling.
///
/// Hidden, semver-exempt, and gated behind the off-by-default
/// `unstable-fuzzing` feature. See the module docs; do not depend on it.
#[cfg(any(test, feature = "unstable-fuzzing"))]
#[doc(hidden)]
pub mod fuzzing;

mod frame_builder;
mod receiver;
mod sender;
mod settings;
mod statistics;
mod tally;
mod types;
mod video_conversion;

pub use discovery::{Discovery, MAX_PLAUSIBLE_SOURCES};
pub use error::{Error, Result};
pub use frame::MediaFrame;
pub use frame_builder::{
    AudioFrameBuilder, MetadataFrameBuilder, OwnedMediaFrame, VideoFrameBuilder,
};
pub use receiver::Receiver;
pub use sender::Sender;
pub use settings::Settings;
pub use statistics::Statistics;
pub use tally::Tally;
pub use types::{
    Codec, ColorSpace, FrameRate, FrameRateError, FrameType, PreferredVideoFormat, Quality,
    ReceiveFlags, SenderInfo, VideoFlags,
};

/// Maximum length for string fields in OMT structures.
pub const MAX_STRING_LENGTH: usize = omt_sys::OMT_MAX_STRING_LENGTH as usize;
