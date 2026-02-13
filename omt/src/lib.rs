//! High-level safe Rust bindings for the Open Media Transport (OMT) library.
//!
//! Part of the [Open Media Transport](https://github.com/openmediatransport) project.
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
//! let receiver = Receiver::new(
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

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod codec;
mod discovery;
mod error;
mod frame;
mod receiver;
mod sender;
mod settings;
mod statistics;
mod tally;
mod types;

pub use codec::Codec;
pub use discovery::Discovery;
pub use error::{Error, Result};
pub use frame::{AudioFrame, MediaFrame, MetadataFrame, VideoFrame};
pub use receiver::Receiver;
pub use sender::Sender;
pub use settings::Settings;
pub use statistics::Statistics;
pub use tally::Tally;
pub use types::{
    ColorSpace, FrameType, PreferredVideoFormat, Quality, ReceiveFlags, SenderInfo, VideoFlags,
};

/// Sets the logging filename for the OMT library.
///
/// If this function is not called, a log file is created in the default location:
/// - macOS/Linux: `~/.OMT/logs/` folder for this process
/// - Windows: `C:\ProgramData\OMT\logs`
///
/// To override the default folder used for logs, set the `OMT_STORAGE_PATH`
/// environment variable prior to calling any OMT functions.
///
/// # Arguments
///
/// * `filename` - Full path to the log file, or `None` to disable logging
///
/// # Examples
///
/// ```no_run
/// # use omt::set_logging_filename;
/// // Enable logging to a specific file
/// set_logging_filename(Some("/tmp/omt.log"));
///
/// // Disable logging
/// set_logging_filename(None);
/// ```
pub fn set_logging_filename(filename: Option<&str>) {
    use std::ffi::CString;
    use std::ptr;

    unsafe {
        if let Some(name) = filename {
            if let Ok(c_name) = CString::new(name) {
                omt_sys::omt_setloggingfilename(c_name.as_ptr());
            }
        } else {
            omt_sys::omt_setloggingfilename(ptr::null());
        }
    }
}

/// Maximum length for string fields in OMT structures.
pub const MAX_STRING_LENGTH: usize = omt_sys::OMT_MAX_STRING_LENGTH as usize;
