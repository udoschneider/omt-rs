mod ffi;
mod ffi_utils;

mod media_frame;
mod video_conversion;

pub mod discovery;
pub mod error;
pub mod helpers;
pub mod receiver;
pub mod sender;
pub mod settings;
pub mod types;

pub use discovery::Discovery;
pub use error::Error;
pub use receiver::Receiver;
pub use sender::Sender;
pub use settings::{
    get_discovery_server, get_network_port_end, get_network_port_range, get_network_port_start,
    set_discovery_server, set_logging_filename, set_network_port_end, set_network_port_range,
    set_network_port_start, settings_get_integer, settings_get_string, settings_set_integer,
    settings_set_string,
};
pub use types::{
    Address, Codec, ColorSpace, FrameType, MediaFrame, Name, PreferredVideoFormat, Quality,
    ReceiveFlags, SenderInfo, Statistics, Tally, Timeout, VideoFlags,
};
