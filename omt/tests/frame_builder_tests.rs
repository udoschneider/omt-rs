//! Integration tests for frame builders.

use omt::{
    AudioFrameBuilder, Codec, ColorSpace, MetadataFrameBuilder, VideoFlags, VideoFrameBuilder,
};

#[test]
fn test_video_frame_builder_basic() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2]; // UYVY: 2 bytes per pixel

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .stride((width * 2) as i32)
        .frame_rate(30, 1)
        .aspect_ratio(16.0 / 9.0)
        .data(data)
        .build()
        .expect("Failed to build video frame");

    assert_eq!(frame.codec(), Codec::Uyvy);
    assert_eq!(frame.data().len(), width * height * 2);
}

#[test]
fn test_video_frame_builder_auto_stride() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    // Test UYVY auto-stride (width * 2)
    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .data(data.clone())
        .build()
        .expect("Failed to build video frame");

    assert!(frame.data().len() == width * height * 2);

    // Test BGRA auto-stride (width * 4)
    let data_bgra = vec![0u8; width * height * 4];
    let frame = VideoFrameBuilder::new()
        .codec(Codec::Bgra)
        .dimensions(width as i32, height as i32)
        .data(data_bgra)
        .build()
        .expect("Failed to build BGRA frame");

    assert!(frame.data().len() == width * height * 4);
}

#[test]
fn test_video_frame_builder_with_flags() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 4];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Bgra)
        .dimensions(width as i32, height as i32)
        .flags(VideoFlags::ALPHA | VideoFlags::PRE_MULTIPLIED)
        .data(data)
        .build()
        .expect("Failed to build video frame with flags");

    assert_eq!(frame.codec(), Codec::Bgra);
}

#[test]
fn test_video_frame_builder_with_color_space() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .color_space(ColorSpace::Bt709)
        .data(data)
        .build()
        .expect("Failed to build video frame with color space");

    assert_eq!(frame.codec(), Codec::Uyvy);
}

#[test]
fn test_video_frame_builder_with_timestamp() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .timestamp(10_000_000) // 1 second
        .data(data)
        .build()
        .expect("Failed to build video frame with timestamp");

    assert_eq!(frame.timestamp(), 10_000_000);
}

#[test]
fn test_video_frame_builder_with_frame_metadata() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .frame_metadata("<test>metadata</test>".to_string())
        .data(data)
        .build()
        .expect("Failed to build video frame with metadata");

    assert_eq!(frame.codec(), Codec::Uyvy);
}

#[test]
fn test_video_frame_builder_missing_codec() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let result = VideoFrameBuilder::new()
        .dimensions(width as i32, height as i32)
        .data(data)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_video_frame_builder_invalid_dimensions() {
    let data = vec![0u8; 1920 * 1080 * 2];

    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(0, 0)
        .data(data)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_video_frame_builder_empty_data() {
    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(1920, 1080)
        .data(vec![])
        .build();

    assert!(result.is_err());
}

#[test]
fn test_audio_frame_builder_basic() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;

    // Create planar audio data
    let audio_samples = vec![0.0f32; (samples_per_channel * channels) as usize];
    let data = audio_samples
        .iter()
        .flat_map(|&f| f.to_ne_bytes())
        .collect::<Vec<u8>>();

    let frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(data)
        .build()
        .expect("Failed to build audio frame");

    assert_eq!(frame.codec(), Codec::Fpa1);
    assert_eq!(
        frame.data().len(),
        (samples_per_channel * channels * 4) as usize
    );
}

#[test]
fn test_audio_frame_builder_stereo() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1600i32;

    // Generate test tone
    let mut audio_samples = vec![0.0f32; (samples_per_channel * channels) as usize];
    let frequency = 440.0;

    for i in 0..samples_per_channel as usize {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1;
        audio_samples[i] = sample; // Left channel
        audio_samples[samples_per_channel as usize + i] = sample; // Right channel
    }

    let data = audio_samples
        .iter()
        .flat_map(|&f| f.to_ne_bytes())
        .collect::<Vec<u8>>();

    let frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(data)
        .build()
        .expect("Failed to build stereo audio frame");

    assert_eq!(frame.codec(), Codec::Fpa1);
}

#[test]
fn test_audio_frame_builder_with_timestamp() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;

    let audio_samples = vec![0.0f32; (samples_per_channel * channels) as usize];
    let data = audio_samples
        .iter()
        .flat_map(|&f| f.to_ne_bytes())
        .collect::<Vec<u8>>();

    let frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .timestamp(5_000_000) // 0.5 seconds
        .data(data)
        .build()
        .expect("Failed to build audio frame with timestamp");

    assert_eq!(frame.timestamp(), 5_000_000);
}

#[test]
fn test_audio_frame_builder_invalid_sample_rate() {
    let channels = 2i32;
    let samples_per_channel = 1024i32;
    let data = vec![0u8; (samples_per_channel * channels * 4) as usize];

    let result = AudioFrameBuilder::new()
        .sample_rate(0)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(data)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_audio_frame_builder_invalid_channels() {
    let sample_rate = 48000i32;
    let samples_per_channel = 1024i32;
    let data = vec![0u8; (samples_per_channel * 33 * 4) as usize]; // 33 channels (invalid)

    let result = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(33) // Max is 32
        .samples_per_channel(samples_per_channel)
        .data(data)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_audio_frame_builder_wrong_data_size() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;

    // Wrong size: should be samples_per_channel * channels * 4
    let data = vec![0u8; 100];

    let result = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(data)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_metadata_frame_builder_basic() {
    let metadata = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
    <source>Test</source>
</metadata>"#;

    let frame = MetadataFrameBuilder::new()
        .metadata(metadata)
        .build()
        .expect("Failed to build metadata frame");

    assert!(frame.data().len() > 0);
}

#[test]
fn test_metadata_frame_builder_with_timestamp() {
    let metadata = "<test>data</test>";

    let frame = MetadataFrameBuilder::new()
        .metadata(metadata)
        .timestamp(20_000_000) // 2 seconds
        .build()
        .expect("Failed to build metadata frame with timestamp");

    assert_eq!(frame.timestamp(), 20_000_000);
}

#[test]
fn test_metadata_frame_builder_empty() {
    let result = MetadataFrameBuilder::new().metadata("").build();

    assert!(result.is_err());
}

#[test]
fn test_owned_frame_set_timestamp() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let mut frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .timestamp(0)
        .data(data)
        .build()
        .expect("Failed to build frame");

    assert_eq!(frame.timestamp(), 0);

    frame.set_timestamp(10_000_000);
    assert_eq!(frame.timestamp(), 10_000_000);
}

#[test]
fn test_owned_frame_data_access() {
    let width = 1920;
    let height = 1080;
    let data = vec![128u8; width * height * 2];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .data(data)
        .build()
        .expect("Failed to build frame");

    let frame_data = frame.data();
    assert_eq!(frame_data.len(), width * height * 2);
    assert_eq!(frame_data[0], 128);
}

#[test]
fn test_owned_frame_data_mutation() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let mut frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .data(data)
        .build()
        .expect("Failed to build frame");

    {
        let frame_data = frame.data_mut();
        frame_data[0] = 255;
    }

    assert_eq!(frame.data()[0], 255);
}

#[test]
fn test_as_media_frame_conversion() {
    let width = 1920;
    let height = 1080;
    let data = vec![0u8; width * height * 2];

    let frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width as i32, height as i32)
        .data(data)
        .build()
        .expect("Failed to build frame");

    let media_frame = frame.as_media_frame();

    // Verify the media frame has the correct properties
    assert_eq!(media_frame.codec(), Some(Codec::Uyvy));
    assert_eq!(media_frame.width(), width as i32);
    assert_eq!(media_frame.height(), height as i32);
    assert_eq!(media_frame.data().len(), width * height * 2);
}
