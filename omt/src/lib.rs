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

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod discovery;
mod error;
mod frame;
mod frame_builder;
mod receiver;
mod sender;
mod settings;
mod statistics;
mod tally;
mod types;
mod video_conversion;

pub use discovery::Discovery;
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
    Codec, ColorSpace, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, SenderInfo,
    VideoFlags,
};

/// Maximum length for string fields in OMT structures.
pub const MAX_STRING_LENGTH: usize = omt_sys::OMT_MAX_STRING_LENGTH as usize;
