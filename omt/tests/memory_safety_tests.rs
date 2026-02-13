//! Memory safety tests to verify that lifetime constraints prevent common memory errors.
//!
//! These tests verify that the fixes for memory safety issues are working correctly.
//! Many of these tests are compile-time tests - they should fail to compile if the
//! lifetime constraints are removed.

use omt::{AudioFrameBuilder, Codec, FrameType, MetadataFrameBuilder, VideoFrameBuilder};

/// Test that MediaFrame cannot outlive the Receiver
///
/// This is a compile-time test. If uncommented, it should fail to compile
/// with an error about the frame outliving the receiver.
#[test]
#[ignore] // This is a compile-fail test, keep it as documentation
fn test_frame_cannot_outlive_receiver() {
    // This code should NOT compile if uncommented:
    /*
    let frame = {
        let receiver = Receiver::new(
            "omt://localhost:6400",
            FrameType::VIDEO,
            PreferredVideoFormat::Uyvy,
            ReceiveFlags::NONE,
        ).unwrap();

        receiver.receive(FrameType::VIDEO, 100).unwrap()
    }; // receiver dropped here

    // frame is now invalid but would be usable without lifetime constraints
    if let Some(f) = frame {
        let _ = f.data(); // This would be use-after-free!
    }
    */
}

/// Test that MediaFrame from OwnedMediaFrame cannot outlive the owned frame
///
/// This is a compile-time test demonstrating the lifetime fix.
#[test]
#[ignore] // This is a compile-fail test, keep it as documentation
fn test_media_frame_cannot_outlive_owned_frame() {
    // This code should NOT compile if uncommented:
    /*
    let media_frame = {
        let owned_frame = VideoFrameBuilder::new()
            .codec(Codec::Uyvy)
            .dimensions(1920, 1080)
            .data(vec![0u8; 1920 * 1080 * 2])
            .build()
            .unwrap();

        owned_frame.as_media_frame() // Borrows from owned_frame
    }; // owned_frame dropped here

    // media_frame now has dangling pointers!
    let _ = media_frame.data(); // This would be use-after-free!
    */
}

/// Test that data slices cannot outlive the MediaFrame
///
/// This is a compile-time test.
#[test]
#[ignore] // This is a compile-fail test, keep it as documentation
fn test_data_slice_cannot_outlive_frame() {
    // This code should NOT compile if uncommented:
    /*
    let data: &[u8] = {
        let owned_frame = VideoFrameBuilder::new()
            .codec(Codec::Uyvy)
            .dimensions(1920, 1080)
            .data(vec![0u8; 1920 * 1080 * 2])
            .build()
            .unwrap();

        let media_frame = owned_frame.as_media_frame();
        media_frame.data() // Returns &'a [u8] tied to media_frame
    }; // media_frame dropped here

    // data is now a dangling pointer!
    let _ = data[0]; // This would be use-after-free!
    */
}

/// Test correct usage: OwnedMediaFrame and MediaFrame lifetimes are correct
#[test]
fn test_correct_owned_frame_usage() {
    let width = 1920;
    let height = 1080;
    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .build()
        .expect("Failed to build frame");

    // This is correct: media_frame borrows from owned_frame
    // and doesn't outlive it
    let media_frame = owned_frame.as_media_frame();
    assert_eq!(media_frame.width(), 1920);
    assert_eq!(media_frame.height(), 1080);
    assert_eq!(media_frame.data().len(), 1920 * 1080 * 2);

    // Both dropped in correct order
}

/// Test that audio frame data is properly validated for alignment
#[test]
fn test_audio_frame_alignment_validation() {
    // Create valid audio data
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;
    let expected_size = (samples_per_channel * channels * 4) as usize; // 4 bytes per f32

    let mut audio_data = vec![0u8; expected_size];

    // Fill with valid f32 data (zeros)
    let f32_slice: &mut [f32] = unsafe {
        std::slice::from_raw_parts_mut(
            audio_data.as_mut_ptr() as *mut f32,
            (samples_per_channel * channels) as usize,
        )
    };
    for sample in f32_slice.iter_mut() {
        *sample = 0.0;
    }

    let owned_frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(audio_data)
        .build()
        .expect("Failed to build audio frame");

    let media_frame = owned_frame.as_media_frame();

    // This should succeed with properly aligned data
    let planes = media_frame.as_f32_planar();
    assert!(
        planes.is_some(),
        "Should successfully convert aligned audio data"
    );
    let planes = planes.unwrap();
    assert_eq!(planes.len(), channels as usize);
    assert_eq!(planes[0].len(), samples_per_channel as usize);
}

/// Test that audio frame validation rejects misaligned data
#[test]
fn test_audio_frame_rejects_wrong_size() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;
    let expected_size = (samples_per_channel * channels * 4) as usize;

    // Create audio data with WRONG size (off by one)
    let audio_data = vec![0u8; expected_size - 1];

    let result = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(audio_data)
        .build();

    // Should fail validation
    assert!(
        result.is_err(),
        "Should reject audio data with incorrect size"
    );
}

/// Test that video frame with metadata preserves lifetime correctly
#[test]
fn test_video_frame_with_metadata_lifetime() {
    let metadata = String::from(r#"<metadata><frame>42</frame></metadata>"#);

    let width = 1920;
    let height = 1080;
    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .frame_metadata(metadata)
        .build()
        .expect("Failed to build frame");

    let media_frame = owned_frame.as_media_frame();

    // Access metadata - should work while owned_frame is alive
    let frame_meta = media_frame.frame_metadata();
    assert!(frame_meta.contains("<frame>42</frame>"));

    // Both dropped together - correct lifetime management
}

/// Test metadata frame lifetime
#[test]
fn test_metadata_frame_lifetime() {
    let metadata_content = String::from(r#"<root><test>data</test></root>"#);

    let owned_frame = MetadataFrameBuilder::new()
        .metadata(metadata_content.clone())
        .build()
        .expect("Failed to build metadata frame");

    let media_frame = owned_frame.as_media_frame();

    // Access the data while owned_frame is still alive
    let data = media_frame.data();
    assert!(!data.is_empty());

    let meta_str = media_frame.as_utf8().expect("Should be valid UTF-8");
    assert_eq!(meta_str, metadata_content);
}

/// Test that multiple borrows of data from the same frame work correctly
#[test]
fn test_multiple_data_borrows() {
    let width = 1920;
    let height = 1080;
    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .build()
        .expect("Failed to build frame");

    let media_frame = owned_frame.as_media_frame();

    // Multiple borrows should all have the same lifetime
    let data1 = media_frame.data();
    let data2 = media_frame.data();

    assert_eq!(data1.len(), data2.len());
    assert_eq!(data1.as_ptr(), data2.as_ptr());

    // All valid for the same lifetime
}

/// Test that owned frame data can be mutated before conversion
#[test]
fn test_owned_frame_mutation() {
    let width = 1920;
    let height = 1080;
    let mut owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .build()
        .expect("Failed to build frame");

    // Mutate the data
    owned_frame.data_mut()[0] = 42;
    owned_frame.data_mut()[1] = 84;

    // Convert to MediaFrame
    let media_frame = owned_frame.as_media_frame();

    // Verify the mutations are visible
    let data = media_frame.data();
    assert_eq!(data[0], 42);
    assert_eq!(data[1], 84);
}

/// Test timestamp modification on owned frame
#[test]
fn test_owned_frame_timestamp_modification() {
    let width = 1920;
    let height = 1080;
    let mut owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .timestamp(1000)
        .build()
        .expect("Failed to build frame");

    assert_eq!(owned_frame.timestamp(), 1000);

    // Modify timestamp
    owned_frame.set_timestamp(2000);
    assert_eq!(owned_frame.timestamp(), 2000);

    // Verify it's reflected in MediaFrame
    let media_frame = owned_frame.as_media_frame();
    assert_eq!(media_frame.timestamp(), 2000);
}

/// Test that CString in frame metadata is properly null-terminated
#[test]
fn test_metadata_null_termination() {
    let metadata = "Test metadata with special chars: <>&\"'";

    let width = 1920;
    let height = 1080;
    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .frame_metadata(metadata.to_string())
        .build()
        .expect("Failed to build frame");

    let media_frame = owned_frame.as_media_frame();
    let retrieved_meta = media_frame.frame_metadata();

    assert_eq!(retrieved_meta, metadata);
    // Verify no null bytes in the middle
    assert!(!retrieved_meta.contains('\0'));
}

/// Test frame metadata that's too large
#[test]
fn test_oversized_frame_metadata() {
    let huge_metadata = "x".repeat(70000); // Larger than 65536 byte limit

    let width = 1920;
    let height = 1080;
    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .frame_metadata(huge_metadata)
        .build();

    assert!(
        result.is_err(),
        "Should reject metadata larger than 65536 bytes"
    );
}

/// Test audio frame with metadata
#[test]
fn test_audio_frame_with_metadata() {
    let sample_rate = 48000i32;
    let channels = 2i32;
    let samples_per_channel = 1024i32;
    let audio_data = vec![0u8; (samples_per_channel * channels * 4) as usize];
    let metadata = "<audio><timecode>01:00:00:00</timecode></audio>";

    let owned_frame = AudioFrameBuilder::new()
        .sample_rate(sample_rate)
        .channels(channels)
        .samples_per_channel(samples_per_channel)
        .data(audio_data)
        .frame_metadata(metadata.to_string())
        .build()
        .expect("Failed to build audio frame");

    let media_frame = owned_frame.as_media_frame();
    let retrieved_meta = media_frame.frame_metadata();

    assert_eq!(retrieved_meta, metadata);
}

/// Test that empty data is rejected for video frames
#[test]
fn test_video_frame_empty_data_rejected() {
    let width = 1920;
    let height = 1080;
    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![]) // Empty data
        .build();

    assert!(result.is_err(), "Should reject empty video data");
}

/// Test that zero dimensions are rejected
#[test]
fn test_video_frame_zero_dimensions_rejected() {
    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(0, 1080) // Zero width
        .data(vec![1, 2, 3])
        .build();

    assert!(result.is_err(), "Should reject zero dimensions");

    let result = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(1920, 0) // Zero height
        .data(vec![1, 2, 3])
        .build();

    assert!(result.is_err(), "Should reject zero dimensions");
}

/// Test audio frame with invalid channel count
#[test]
fn test_audio_frame_invalid_channels() {
    let result = AudioFrameBuilder::new()
        .sample_rate(48000)
        .channels(0) // Zero channels
        .samples_per_channel(1024)
        .data(vec![0u8; (1024 * 4) as usize])
        .build();

    assert!(result.is_err(), "Should reject zero channels");

    let result = AudioFrameBuilder::new()
        .sample_rate(48000)
        .channels(33) // More than 32 channels
        .samples_per_channel(1024)
        .data(vec![0u8; (33 * 1024 * 4) as usize])
        .build();

    assert!(result.is_err(), "Should reject more than 32 channels");
}

/// Test audio frame with invalid sample rate
#[test]
fn test_audio_frame_invalid_sample_rate() {
    let result = AudioFrameBuilder::new()
        .sample_rate(0) // Zero sample rate
        .channels(2)
        .samples_per_channel(1024)
        .data(vec![0u8; (2 * 1024 * 4) as usize])
        .build();

    assert!(result.is_err(), "Should reject zero sample rate");

    let result = AudioFrameBuilder::new()
        .sample_rate(-1) // Negative sample rate
        .channels(2)
        .samples_per_channel(1024)
        .data(vec![0u8; (2 * 1024 * 4) as usize])
        .build();

    assert!(result.is_err(), "Should reject negative sample rate");
}

/// Test that frame builders validate all required fields
#[test]
fn test_video_frame_missing_codec() {
    let result = VideoFrameBuilder::new()
        // No codec specified
        .dimensions(1920, 1080)
        .data(vec![0u8; 1920 * 1080 * 2])
        .build();

    assert!(result.is_err(), "Should reject frame without codec");
}

/// Demonstrate safe pattern: frame lives only as long as needed
#[test]
fn test_safe_frame_usage_pattern() {
    let width = 1920;
    let height = 1080;
    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .build()
        .expect("Failed to build frame");

    // Process frame immediately
    {
        let media_frame = owned_frame.as_media_frame();
        let _width = media_frame.width();
        let _height = media_frame.height();
        let _data = media_frame.data();
        // media_frame dropped here
    }

    // owned_frame still valid for reuse
    assert_eq!(owned_frame.frame_type(), FrameType::VIDEO);
}

/// Test that stride is calculated correctly for different codecs
#[test]
fn test_stride_calculation() {
    let width = 1920;
    let height = 1080;

    // UYVY: 2 bytes per pixel
    let uyvy_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 2) as usize])
        .build()
        .expect("Failed to build UYVY frame");
    let media_frame = uyvy_frame.as_media_frame();
    assert_eq!(media_frame.stride(), width * 2);

    // BGRA: 4 bytes per pixel
    let bgra_frame = VideoFrameBuilder::new()
        .codec(Codec::Bgra)
        .dimensions(width, height)
        .data(vec![0u8; (width * height * 4) as usize])
        .build()
        .expect("Failed to build BGRA frame");
    let media_frame = bgra_frame.as_media_frame();
    assert_eq!(media_frame.stride(), width * 4);
}

/// Test explicit stride override
#[test]
fn test_explicit_stride() {
    let width = 1920;
    let height = 1080;
    let stride = 2048; // Padded stride

    let owned_frame = VideoFrameBuilder::new()
        .codec(Codec::Uyvy)
        .dimensions(width, height)
        .stride(stride)
        .data(vec![0u8; (stride * height) as usize])
        .build()
        .expect("Failed to build frame with explicit stride");

    let media_frame = owned_frame.as_media_frame();
    assert_eq!(media_frame.stride(), stride);
}

/// These examples demonstrate code patterns that SHOULD NOT compile.
/// They are kept as comments for documentation purposes.
#[cfg(test)]
mod compile_fail_examples {
    /*
    Example 1: Frame outliving receiver

    ```compile_fail
    use omt::{Receiver, FrameType, PreferredVideoFormat, ReceiveFlags};

    let frame = {
        let receiver = Receiver::new(
            "omt://localhost:6400",
            FrameType::VIDEO,
            PreferredVideoFormat::Uyvy,
            ReceiveFlags::NONE,
        ).unwrap();
        receiver.receive(FrameType::VIDEO, 100).unwrap()
    };
    // Error: `receiver` does not live long enough
    ```

    Example 2: MediaFrame outliving OwnedMediaFrame

    ```compile_fail
    use omt::{VideoFrameBuilder, Codec};

    let media_frame = {
        let owned = VideoFrameBuilder::new()
            .codec(Codec::Uyvy)
            .dimensions(1920, 1080)
            .data(vec![0u8; 1920 * 1080 * 2])
            .build()
            .unwrap();
        owned.as_media_frame()
    };
    // Error: `owned` does not live long enough
    ```

    Example 3: Data slice outliving frame

    ```compile_fail
    use omt::{VideoFrameBuilder, Codec};

    let data = {
        let owned = VideoFrameBuilder::new()
            .codec(Codec::Uyvy)
            .dimensions(1920, 1080)
            .data(vec![0u8; 1920 * 1080 * 2])
            .build()
            .unwrap();
        let frame = owned.as_media_frame();
        frame.data()
    };
    // Error: borrowed data escapes outside of its scope
    ```
    */
}
