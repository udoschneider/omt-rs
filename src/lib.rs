mod ffi;
mod ffi_utils;

mod video_conversion;
mod video_frame;

pub mod discovery;
pub mod helpers;
pub mod receiver;
pub mod sender;
pub mod settings;
pub mod types;

use std::fmt;

pub use discovery::Discovery;
pub use receiver::{AudioFrame, Receiver, SenderInfo, Statistics, Tally};
pub use sender::{OutgoingFrame, Sender};
pub use settings::{
    get_discovery_server, get_network_port_end, get_network_port_range, get_network_port_start,
    set_discovery_server, set_logging_filename, set_network_port_end, set_network_port_range,
    set_network_port_start, settings_get_integer, settings_get_string, settings_set_integer,
    settings_set_string,
};
pub use types::{
    Address, Codec, ColorSpace, FrameRef, FrameType, Name, PreferredVideoFormat, Quality,
    ReceiveFlags, Timeout, VideoFlags, VideoFrame,
};

#[derive(Debug)]
pub enum OmtError {
    NullHandle,
    InvalidCString,
}

impl fmt::Display for OmtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OmtError::NullHandle => write!(f, "libomt returned a null handle"),
            OmtError::InvalidCString => write!(f, "string contained an interior null byte"),
        }
    }
}

impl std::error::Error for OmtError {}

impl Codec {
    pub fn fourcc_string(self) -> String {
        fourcc_to_string(self.fourcc())
    }
}

pub fn fourcc_to_string(fourcc: u32) -> String {
    let bytes = [
        (fourcc & 0xff) as u8,
        ((fourcc >> 8) & 0xff) as u8,
        ((fourcc >> 16) & 0xff) as u8,
        ((fourcc >> 24) & 0xff) as u8,
    ];
    bytes
        .iter()
        .map(|&b| {
            if (32..=126).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}
