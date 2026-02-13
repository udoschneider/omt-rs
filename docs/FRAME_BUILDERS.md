# Frame Builders

This document describes how to create and send owned media frames using the OMT Rust library.

## Overview

The OMT library provides three builder types for creating owned frames that can be sent over the network:

- **`VideoFrameBuilder`** - For creating video frames
- **`AudioFrameBuilder`** - For creating audio frames  
- **`MetadataFrameBuilder`** - For creating metadata frames

All builders follow the builder pattern and create `OwnedMediaFrame` instances that own their data and manage memory safely.

## Video Frames

### Basic Usage

```rust
use omt::{VideoFrameBuilder, Codec, VideoFlags};

// Create video data (UYVY format: 2 bytes per pixel)
let width = 1920;
let height = 1080;
let data = vec![0u8; width * height * 2];

// Build the frame
let frame = VideoFrameBuilder::new()
    .codec(Codec::Uyvy)
    .dimensions(width as i32, height as i32)
    .stride((width * 2) as i32)
    .frame_rate(30, 1)  // 30 fps
    .aspect_ratio(16.0 / 9.0)
    .data(data)
    .build()?;

// Send the frame
let media_frame = frame.as_media_frame();
sender.send(&media_frame)?;
```

### Supported Video Codecs

When sending video frames, the following codecs are supported:

- **`Codec::Uyvy`** - 16bpp YUV format (2 bytes per pixel)
- **`Codec::Yuy2`** - 16bpp YUV format, YUYV pixel order (2 bytes per pixel)
- **`Codec::Bgra`** - 32bpp RGBA format (4 bytes per pixel)
- **`Codec::Uyva`** - 16bpp YUV with separate alpha plane
- **`Codec::Nv12`** - Planar 4:2:0 YUV format
- **`Codec::Yv12`** - Planar 4:2:0 YUV format
- **`Codec::P216`** - Planar 4:2:2 YUV format (16-bit)
- **`Codec::Pa16`** - P216 with alpha plane
- **`Codec::Vmx1`** - Pre-compressed VMX1 format

### Video Frame Properties

#### Dimensions and Stride

```rust
VideoFrameBuilder::new()
    .dimensions(1920, 1080)
    .stride(1920 * 2)  // For UYVY: width * 2
```

If stride is not specified, it will be automatically calculated based on the codec:
- UYVY/YUY2: `width * 2`
- BGRA: `width * 4`
- Planar formats (NV12, YV12): `width`

#### Frame Rate

Frame rate is specified as a numerator and denominator:

```rust
VideoFrameBuilder::new()
    .frame_rate(30, 1)      // 30 fps
    .frame_rate(60, 1)      // 60 fps
    .frame_rate(30000, 1001) // 29.97 fps (NTSC)
```

#### Video Flags

```rust
use omt::VideoFlags;

VideoFrameBuilder::new()
    .flags(VideoFlags::INTERLACED)
    .flags(VideoFlags::ALPHA | VideoFlags::PRE_MULTIPLIED)
```

Available flags:
- `VideoFlags::NONE` - No special flags
- `VideoFlags::INTERLACED` - Frame is interlaced
- `VideoFlags::ALPHA` - Frame contains alpha channel
- `VideoFlags::PRE_MULTIPLIED` - Alpha is premultiplied
- `VideoFlags::PREVIEW` - 1/8th preview frame
- `VideoFlags::HIGH_BIT_DEPTH` - High bit depth (P216/PA16)

#### Color Space

```rust
use omt::ColorSpace;

VideoFrameBuilder::new()
    .color_space(ColorSpace::Bt709)
```

Options:
- `ColorSpace::Undefined` - Auto-select based on resolution (default)
- `ColorSpace::Bt601` - BT.601 color space
- `ColorSpace::Bt709` - BT.709 color space

#### Timestamps

Timestamps are in OMT units where **1 second = 10,000,000 units**:

```rust
VideoFrameBuilder::new()
    .timestamp(-1)  // Auto-generate timestamps (default)
    .timestamp(0)   // Start at time 0
    .timestamp(10_000_000)  // 1 second
```

#### Per-Frame Metadata

You can attach metadata to individual frames (max 65536 bytes):

```rust
VideoFrameBuilder::new()
    .frame_metadata("<frame_info>...</frame_info>".to_string())
```

## Audio Frames

### Basic Usage

```rust
use omt::AudioFrameBuilder;

let sample_rate = 48000;
let channels = 2;
let samples_per_channel = 1600;

// Create planar audio data (all channel 0 samples, then all channel 1 samples, etc.)
let mut audio_samples = vec![0.0f32; samples_per_channel * channels];

// Generate a test tone
let frequency = 440.0; // A4 note
for i in 0..samples_per_channel {
    let t = i as f32 / sample_rate as f32;
    let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1;
    audio_samples[i] = sample; // Left channel
    audio_samples[samples_per_channel + i] = sample; // Right channel
}

// Convert f32 samples to bytes
let data = audio_samples
    .iter()
    .flat_map(|&f| f.to_ne_bytes())
    .collect::<Vec<u8>>();

// Build the audio frame
let frame = AudioFrameBuilder::new()
    .sample_rate(sample_rate)
    .channels(channels)
    .samples_per_channel(samples_per_channel)
    .data(data)
    .build()?;

// Send the frame
let media_frame = frame.as_media_frame();
sender.send(&media_frame)?;
```

### Audio Format

Audio frames use the **FPA1** codec (32-bit floating-point planar audio):

- **Format**: 32-bit float samples per channel
- **Layout**: Planar (all samples for channel 0, then channel 1, etc.)
- **Channels**: 1-32 channels supported
- **Sample rates**: Common rates like 44100, 48000, etc.

### Audio Data Layout

Audio data must be in planar format:

```
[Ch0_Sample0, Ch0_Sample1, ..., Ch0_SampleN,
 Ch1_Sample0, Ch1_Sample1, ..., Ch1_SampleN,
 ...
 ChM_Sample0, ChM_Sample1, ..., ChM_SampleN]
```

Each sample is a 32-bit float (4 bytes), so the total data size is:
```
samples_per_channel * channels * 4 bytes
```

### Example: Stereo Audio

```rust
let samples_per_channel = 1024;
let channels = 2;

// Create left and right channel data
let left_channel: Vec<f32> = (0..samples_per_channel)
    .map(|i| generate_sample(i, 440.0))
    .collect();
    
let right_channel: Vec<f32> = (0..samples_per_channel)
    .map(|i| generate_sample(i, 880.0))
    .collect();

// Combine into planar format
let audio_samples: Vec<f32> = left_channel
    .into_iter()
    .chain(right_channel.into_iter())
    .collect();

// Convert to bytes
let data = audio_samples
    .iter()
    .flat_map(|&f| f.to_ne_bytes())
    .collect::<Vec<u8>>();
```

## Metadata Frames

### Basic Usage

```rust
use omt::MetadataFrameBuilder;

let metadata = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
    <source>My Application</source>
    <frame_count>1234</frame_count>
</metadata>"#;

let frame = MetadataFrameBuilder::new()
    .metadata(metadata)
    .build()?;

let media_frame = frame.as_media_frame();
sender.send(&media_frame)?;
```

### Metadata Format

- **Encoding**: UTF-8
- **Format**: Typically XML, but any UTF-8 text is supported
- **Null termination**: Automatically added by the builder

## Working with OwnedMediaFrame

### Converting to MediaFrame

To send an owned frame, convert it to a borrowed `MediaFrame`:

```rust
let owned_frame = VideoFrameBuilder::new()
    // ... configuration
    .build()?;

let media_frame = owned_frame.as_media_frame();
sender.send(&media_frame)?;
```

**Important**: The `OwnedMediaFrame` must remain valid while `media_frame` is in use, since `media_frame` borrows data from it.

### Accessing Frame Data

```rust
let frame = VideoFrameBuilder::new()
    .codec(Codec::Uyvy)
    .dimensions(1920, 1080)
    .data(vec![0u8; 1920 * 1080 * 2])
    .build()?;

// Read frame properties
println!("Frame type: {:?}", frame.frame_type());
println!("Timestamp: {}", frame.timestamp());
println!("Codec: {:?}", frame.codec());

// Access data
let data: &[u8] = frame.data();

// Modify data
let data_mut: &mut [u8] = frame.data_mut();
```

### Modifying Timestamps

You can update the timestamp on an existing frame:

```rust
let mut frame = VideoFrameBuilder::new()
    // ... configuration
    .build()?;

frame.set_timestamp(10_000_000); // 1 second
```

## Complete Example

Here's a complete example that sends video, audio, and metadata:

```rust
use omt::{
    Sender, Quality, SenderInfo,
    VideoFrameBuilder, AudioFrameBuilder, MetadataFrameBuilder,
    Codec, VideoFlags,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sender
    let sender = Sender::new("My Source", Quality::High)?;
    
    let info = SenderInfo::new(
        "My App".to_string(),
        "My Company".to_string(),
        "1.0.0".to_string(),
    );
    sender.set_sender_information(&info)?;
    
    let start_time = std::time::Instant::now();
    let mut frame_count = 0;
    
    loop {
        // Calculate timestamp
        let elapsed = start_time.elapsed();
        let timestamp = (elapsed.as_secs_f64() * 10_000_000.0) as i64;
        
        // Send video frame
        let video_data = create_video_frame_data();
        let video_frame = VideoFrameBuilder::new()
            .codec(Codec::Uyvy)
            .dimensions(1920, 1080)
            .stride(1920 * 2)
            .frame_rate(30, 1)
            .aspect_ratio(16.0 / 9.0)
            .timestamp(timestamp)
            .data(video_data)
            .build()?;
        sender.send(&video_frame.as_media_frame())?;
        
        // Send audio frame
        let audio_data = create_audio_frame_data();
        let audio_frame = AudioFrameBuilder::new()
            .sample_rate(48000)
            .channels(2)
            .samples_per_channel(1600)
            .timestamp(timestamp)
            .data(audio_data)
            .build()?;
        sender.send(&audio_frame.as_media_frame())?;
        
        // Send metadata every 30 frames
        if frame_count % 30 == 0 {
            let metadata = format!(
                r#"<metadata><frame>{}</frame></metadata>"#,
                frame_count
            );
            let meta_frame = MetadataFrameBuilder::new()
                .metadata(metadata)
                .timestamp(timestamp)
                .build()?;
            sender.send(&meta_frame.as_media_frame())?;
        }
        
        frame_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
```

## Error Handling

All builders return `Result<OwnedMediaFrame, Error>`. Common errors:

- **`InvalidParameter`** - Missing required fields or invalid values
- **`BufferTooSmall`** - Data exceeds maximum size (e.g., frame metadata > 65536 bytes)
- **`NulError`** - String contains null byte

Example:

```rust
let result = VideoFrameBuilder::new()
    .codec(Codec::Uyvy)
    .dimensions(1920, 1080)
    .data(vec![0u8; 100]) // Too small!
    .build();

match result {
    Ok(frame) => { /* ... */ }
    Err(e) => eprintln!("Failed to build frame: {}", e),
}
```

## Best Practices

1. **Reuse buffers**: Create data buffers once and reuse them to avoid allocations
2. **Validate data sizes**: Ensure data buffer matches expected size for codec and dimensions
3. **Use auto timestamps**: Set timestamp to -1 to let the library generate timestamps
4. **Check connections**: Use `sender.connections()` to verify receivers are connected
5. **Monitor statistics**: Use `sender.get_video_statistics()` to track performance
6. **Handle errors**: Always check build results and send results

## Performance Tips

1. **Pre-allocate data buffers**:
   ```rust
   let mut video_buffer = vec![0u8; 1920 * 1080 * 2];
   // Reuse video_buffer for each frame
   ```

2. **Avoid unnecessary conversions**:
   ```rust
   // Good: Generate data directly as bytes
   let data = generate_raw_bytes();
   
   // Avoid: Converting back and forth
   let floats = generate_floats();
   let data = floats_to_bytes(floats); // Extra work!
   ```

3. **Batch frame creation**:
   ```rust
   // Create multiple frames at once if you have the data ready
   let frames: Vec<OwnedMediaFrame> = frame_data_batch
       .into_iter()
       .map(|data| build_frame(data))
       .collect::<Result<Vec<_>, _>>()?;
   ```

## Thread Safety

`OwnedMediaFrame` is both `Send` and `Sync`, so you can:
- Create frames in one thread and send them in another
- Share frames between threads (with proper synchronization)
- Use parallel frame generation

Example:
```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();

// Frame generation thread
std::thread::spawn(move || {
    let frame = VideoFrameBuilder::new()
        // ... configuration
        .build().unwrap();
    tx.send(frame).unwrap();
});

// Sending thread
let frame = rx.recv().unwrap();
sender.send(&frame.as_media_frame())?;
```
