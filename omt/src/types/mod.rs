//! Core types and enumerations for the OMT library.

mod codec;
mod color_space;
mod flags;
mod format;
mod frame_type;
mod quality;
mod sender_info;

pub use codec::Codec;
pub use color_space::ColorSpace;
pub use flags::{ReceiveFlags, VideoFlags};
pub use format::PreferredVideoFormat;
pub use frame_type::FrameType;
pub use quality::Quality;
pub use sender_info::SenderInfo;
