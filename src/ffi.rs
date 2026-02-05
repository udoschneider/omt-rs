#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::{c_char, c_int, c_longlong, c_void};

pub const OMT_MAX_STRING_LENGTH: usize = 1024;

#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_snake_case)]
pub enum OMTFrameType {
    None = 0,
    Metadata = 1,
    Video = 2,
    Audio = 4,
    INT32 = 0x7fffffff,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum OMTCodec {
    VMX1 = 0x31584D56,
    FPA1 = 0x31415046,
    UYVY = 0x59565955,
    YUY2 = 0x32595559,
    BGRA = 0x41524742,
    NV12 = 0x3231564E,
    YV12 = 0x32315659,
    UYVA = 0x41565955,
    P216 = 0x36313250,
    PA16 = 0x36314150,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OMTQuality {
    Default = 0,
    Low = 1,
    Medium = 50,
    High = 100,
    INT32 = 0x7fffffff,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OMTColorSpace {
    Undefined = 0,
    BT601 = 601,
    BT709 = 709,
    INT32 = 0x7fffffff,
}

pub type OMTVideoFlags = i32;

pub const OMT_VIDEO_FLAGS_NONE: OMTVideoFlags = 0;
pub const OMT_VIDEO_FLAGS_INTERLACED: OMTVideoFlags = 1;
pub const OMT_VIDEO_FLAGS_ALPHA: OMTVideoFlags = 2;
pub const OMT_VIDEO_FLAGS_PREMULTIPLIED: OMTVideoFlags = 4;
pub const OMT_VIDEO_FLAGS_PREVIEW: OMTVideoFlags = 8;
pub const OMT_VIDEO_FLAGS_HIGH_BIT_DEPTH: OMTVideoFlags = 16;

#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum OMTPreferredVideoFormat {
    UYVY = 0,
    UYVYorBGRA = 1,
    BGRA = 2,
    UYVYorUYVA = 3,
    UYVYorUYVAorP216orPA16 = 4,
    P216 = 5,
    INT32 = 0x7fffffff,
}

pub type OMTReceiveFlags = i32;

pub const OMT_RECEIVE_FLAGS_NONE: OMTReceiveFlags = 0;
pub const OMT_RECEIVE_FLAGS_PREVIEW: OMTReceiveFlags = 1;
pub const OMT_RECEIVE_FLAGS_INCLUDE_COMPRESSED: OMTReceiveFlags = 2;
pub const OMT_RECEIVE_FLAGS_COMPRESSED_ONLY: OMTReceiveFlags = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OMTTally {
    pub preview: c_int,
    pub program: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OMTSenderInfo {
    pub ProductName: [c_char; OMT_MAX_STRING_LENGTH],
    pub Manufacturer: [c_char; OMT_MAX_STRING_LENGTH],
    pub Version: [c_char; OMT_MAX_STRING_LENGTH],
    pub Reserved1: [c_char; OMT_MAX_STRING_LENGTH],
    pub Reserved2: [c_char; OMT_MAX_STRING_LENGTH],
    pub Reserved3: [c_char; OMT_MAX_STRING_LENGTH],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OMTStatistics {
    pub BytesSent: c_longlong,
    pub BytesReceived: c_longlong,
    pub BytesSentSinceLast: c_longlong,
    pub BytesReceivedSinceLast: c_longlong,
    pub Frames: c_longlong,
    pub FramesSinceLast: c_longlong,
    pub FramesDropped: c_longlong,
    pub CodecTime: c_longlong,
    pub CodecTimeSinceLast: c_longlong,
    pub Reserved1: c_longlong,
    pub Reserved2: c_longlong,
    pub Reserved3: c_longlong,
    pub Reserved4: c_longlong,
    pub Reserved5: c_longlong,
    pub Reserved6: c_longlong,
    pub Reserved7: c_longlong,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OMTMediaFrame {
    pub Type: OMTFrameType,
    pub Timestamp: c_longlong,
    pub Codec: OMTCodec,
    pub Width: c_int,
    pub Height: c_int,
    pub Stride: c_int,
    pub Flags: OMTVideoFlags,
    pub FrameRateN: c_int,
    pub FrameRateD: c_int,
    pub AspectRatio: f32,
    pub ColorSpace: OMTColorSpace,
    pub SampleRate: c_int,
    pub Channels: c_int,
    pub SamplesPerChannel: c_int,
    pub Data: *mut c_void,
    pub DataLength: c_int,
    pub CompressedData: *mut c_void,
    pub CompressedLength: c_int,
    pub FrameMetadata: *mut c_void,
    pub FrameMetadataLength: c_int,
}

pub type omt_receive_t = c_longlong;
pub type omt_send_t = c_longlong;

#[link(name = "omt")]
extern "C" {
    pub fn omt_discovery_getaddresses(count: *mut c_int) -> *mut *mut c_char;

    pub fn omt_receive_create(
        address: *const c_char,
        frameTypes: OMTFrameType,
        format: OMTPreferredVideoFormat,
        flags: OMTReceiveFlags,
    ) -> *mut omt_receive_t;
    pub fn omt_receive_destroy(instance: *mut omt_receive_t);
    pub fn omt_receive(
        instance: *mut omt_receive_t,
        frameTypes: OMTFrameType,
        timeoutMilliseconds: c_int,
    ) -> *mut OMTMediaFrame;
    pub fn omt_receive_send(instance: *mut omt_receive_t, frame: *mut OMTMediaFrame) -> c_int;
    pub fn omt_receive_settally(instance: *mut omt_receive_t, tally: *mut OMTTally);
    pub fn omt_receive_gettally(
        instance: *mut omt_send_t,
        timeoutMilliseconds: c_int,
        tally: *mut OMTTally,
    ) -> c_int;
    pub fn omt_receive_setflags(instance: *mut omt_receive_t, flags: OMTReceiveFlags);
    pub fn omt_receive_setsuggestedquality(instance: *mut omt_receive_t, quality: OMTQuality);
    pub fn omt_receive_getsenderinformation(instance: *mut omt_receive_t, info: *mut OMTSenderInfo);
    pub fn omt_receive_getvideostatistics(instance: *mut omt_receive_t, stats: *mut OMTStatistics);
    pub fn omt_receive_getaudiostatistics(instance: *mut omt_receive_t, stats: *mut OMTStatistics);

    pub fn omt_send_create(name: *const c_char, quality: OMTQuality) -> *mut omt_send_t;
    pub fn omt_send_setsenderinformation(instance: *mut omt_send_t, info: *mut OMTSenderInfo);
    pub fn omt_send_addconnectionmetadata(instance: *mut omt_send_t, metadata: *const c_char);
    pub fn omt_send_clearconnectionmetadata(instance: *mut omt_send_t);
    pub fn omt_send_setredirect(instance: *mut omt_send_t, newAddress: *const c_char);
    pub fn omt_send_getaddress(
        instance: *mut omt_send_t,
        address: *mut c_char,
        maxLength: c_int,
    ) -> c_int;
    pub fn omt_send_destroy(instance: *mut omt_send_t);
    pub fn omt_send(instance: *mut omt_send_t, frame: *mut OMTMediaFrame) -> c_int;
    pub fn omt_send_connections(instance: *mut omt_send_t) -> c_int;
    pub fn omt_send_receive(
        instance: *mut omt_send_t,
        timeoutMilliseconds: c_int,
    ) -> *mut OMTMediaFrame;
    pub fn omt_send_gettally(
        instance: *mut omt_send_t,
        timeoutMilliseconds: c_int,
        tally: *mut OMTTally,
    ) -> c_int;
    pub fn omt_send_getvideostatistics(instance: *mut omt_send_t, stats: *mut OMTStatistics);
    pub fn omt_send_getaudiostatistics(instance: *mut omt_send_t, stats: *mut OMTStatistics);

    pub fn omt_setloggingfilename(filename: *const c_char);

    pub fn omt_settings_get_string(
        name: *const c_char,
        value: *mut c_char,
        maxLength: c_int,
    ) -> c_int;
    pub fn omt_settings_set_string(name: *const c_char, value: *const c_char);
    pub fn omt_settings_get_integer(name: *const c_char) -> c_int;
    pub fn omt_settings_set_integer(name: *const c_char, value: c_int);
}
