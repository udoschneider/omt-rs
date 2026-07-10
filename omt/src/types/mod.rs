//! Core types and enumerations for the OMT library.

mod codec;
mod color_space;
mod frame_rate;
mod frame_type;
mod preferred_video_format;
mod quality;
mod receive_flags;
mod sender_info;
mod video_flags;

pub use codec::Codec;
pub use color_space::ColorSpace;
pub use frame_rate::{FrameRate, FrameRateError};
pub use frame_type::FrameType;
pub use preferred_video_format::PreferredVideoFormat;
pub use quality::Quality;
pub use receive_flags::ReceiveFlags;
pub use sender_info::SenderInfo;
pub use video_flags::VideoFlags;
