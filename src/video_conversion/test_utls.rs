//! Tests Utils for video frame format conversion utilities.
//!
//! This module provides helper functions for generating test data in various pixel formats.
//! The helper functions create predictable color patterns based on CGA (Color Graphics Adapter)
//! color palette with alpha variations for testing video conversion functions.

use rgb::*;

/// Returns 48 bytes (16 RGB8 pixels) of test data.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgb8_data;
/// let data = rgb8_data();
/// assert_eq!(data.len(), 48); // 16 pixels × 3 bytes each
/// ```
#[cfg(test)]
fn rgb8_data() -> Vec<u8> {
    let colors: Vec<Rgb<u8>> = rgb8_colors();
    let bytes: &[u8] = rgb::bytemuck::cast_slice(&colors);
    return bytes.to_vec();
}

/// Returns 192 bytes (64 RGBA8 pixels) of test data.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgba8_data;
/// let data = rgba8_data();
/// assert_eq!(data.len(), 192); // 64 pixels × 4 bytes each
/// ```
#[cfg(test)]
fn rgba8_data() -> Vec<u8> {
    let colors: Vec<Rgba<u8>> = rgba8_colors();
    let bytes: &[u8] = rgb::bytemuck::cast_slice(&colors);
    return bytes.to_vec();
}

/// Returns 96 bytes (16 RGB16 pixels) of test data.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgb16_data;
/// let data = rgb16_data();
/// assert_eq!(data.len(), 96); // 16 pixels × 6 bytes each (3 × u16)
/// ```
#[cfg(test)]
fn rgb16_data() -> Vec<u8> {
    let colors: Vec<Rgb<u16>> = rgb16_colors();
    let bytes: &[u8] = rgb::bytemuck::cast_slice(&colors);
    return bytes.to_vec();
}

/// Returns 384 bytes (64 RGBA16 pixels) of test data.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgba16_data;
/// let data = rgba16_data();
/// assert_eq!(data.len(), 384); // 64 pixels × 8 bytes each (4 × u16)
/// ```
#[cfg(test)]
fn rgba16_data() -> Vec<u8> {
    let colors: Vec<Rgba<u16>> = rgba16_colors();
    let bytes: &[u8] = rgb::bytemuck::cast_slice(&colors);
    return bytes.to_vec();
}

/// Returns 16 RGB8 colors based on CGA palette.
///
/// The colors are generated from the CGA color palette with 16 colors.
/// Each color component is scaled from [0.0, 1.0] float range to [0, 255] u8 range.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgb8_colors;
/// use rgb::Rgb;
/// let colors = rgb8_colors();
/// assert_eq!(colors.len(), 16);
/// assert!(colors[0].r <= 255);
/// ```
#[cfg(test)]
fn rgb8_colors() -> Vec<Rgb<u8>> {
    return cga_colors()
        .iter()
        .map(|c| Rgb::<u8> {
            r: (c.r * 255.0) as u8,
            g: (c.g * 255.0) as u8,
            b: (c.b * 255.0) as u8,
        })
        .collect();
}

/// Returns 64 RGBA8 colors with alpha variations.
///
/// The colors are generated from the CGA color palette with 4 alpha levels (0%, 33%, 67%, 100%).
/// Each color component is scaled from [0.0, 1.0] float range to [0, 255] u8 range.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgba8_colors;
/// use rgb::Rgba;
/// let colors = rgba8_colors();
/// assert_eq!(colors.len(), 64); // 16 colors × 4 alpha levels
/// assert!(colors[0].a <= 255);
/// ```
#[cfg(test)]
pub fn rgba8_colors() -> Vec<Rgba<u8>> {
    return cga_alpha_colors()
        .iter()
        .map(|c| Rgba::<u8> {
            r: (c.r * 255.0) as u8,
            g: (c.g * 255.0) as u8,
            b: (c.b * 255.0) as u8,
            a: (c.a * 255.0) as u8,
        })
        .collect();
}

/// Returns 16 RGB16 colors based on CGA palette.
///
/// The colors are generated from the CGA color palette with 16 colors.
/// Each color component is scaled from [0.0, 1.0] float range to [0, 65535] u16 range.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgb16_colors;
/// use rgb::Rgb;
/// let colors = rgb16_colors();
/// assert_eq!(colors.len(), 16);
/// assert!(colors[0].r <= 65535);
/// ```
#[cfg(test)]
fn rgb16_colors() -> Vec<Rgb<u16>> {
    return cga_colors()
        .iter()
        .map(|c| Rgb::<u16> {
            r: (c.r * 65535.0) as u16,
            g: (c.g * 65535.0) as u16,
            b: (c.b * 65535.0) as u16,
        })
        .collect();
}

/// Returns 64 RGBA16 colors with alpha variations.
///
/// The colors are generated from the CGA color palette with 4 alpha levels (0%, 33%, 67%, 100%).
/// Each color component is scaled from [0.0, 1.0] float range to [0, 65535] u16 range.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::rgba16_colors;
/// use rgb::Rgba;
/// let colors = rgba16_colors();
/// assert_eq!(colors.len(), 64); // 16 colors × 4 alpha levels
/// assert!(colors[0].a <= 65535);
/// ```
#[cfg(test)]
fn rgba16_colors() -> Vec<Rgba<u16>> {
    return cga_alpha_colors()
        .iter()
        .map(|c| Rgba::<u16> {
            r: (c.r * 65535.0) as u16,
            g: (c.g * 65535.0) as u16,
            b: (c.b * 65535.0) as u16,
            a: (c.a * 65535.0) as u16,
        })
        .collect();
}

/// Returns 16 CGA colors in floating-point RGB format.
///
/// Generates the 16-color CGA palette with RGB components in [0.0, 1.0] range.
/// The CGA palette uses 4-bit color with RGBI encoding (Red, Green, Blue, Intensity).
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::cga_colors;
/// use rgb::Rgb;
/// let colors = cga_colors();
/// assert_eq!(colors.len(), 16);
/// assert!(colors[0].r >= 0.0 && colors[0].r <= 1.0);
/// ```
#[cfg(test)]
fn cga_colors() -> Vec<Rgb<f32>> {
    (48..64).map(|i| cga_alpha_color(i).rgb()).collect()
}

/// Returns 64 CGA colors with alpha variations in floating-point RGBA format.
///
/// Generates 64 colors by combining 16 CGA colors with 4 alpha levels (0%, 33%, 67%, 100%).
/// The alpha value is determined by bits 4-5 of the index (0b110000).
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::cga_alpha_colors;
/// use rgb::Rgba;
/// let colors = cga_alpha_colors();
/// assert_eq!(colors.len(), 64);
/// assert!(colors[0].a >= 0.0 && colors[0].a <= 1.0);
/// ```
#[cfg(test)]
pub fn cga_alpha_colors() -> Vec<Rgba<f32>> {
    (0..64).map(|i| cga_alpha_color(i)).collect()
}

/// Generates a single CGA color with alpha from a 6-bit index.
///
/// The index is decoded as follows:
/// - Bits 0-2: RGB components (Blue, Green, Red)
/// - Bit 3: Intensity bit
/// - Bits 4-5: Alpha level (0-3, scaled to 0.0-1.0)
///
/// Color 6 (brown) has special handling: green component is reduced to 2/3.
///
/// # Arguments
///
/// * `index` - A 6-bit index (0-63) encoding color and alpha information
///
/// # Returns
///
/// An `Rgba<f32>` with components in [0.0, 1.0] range.
///
/// # Examples
///
/// ```
/// use omt_rs::video_conversion::tests::cga_alpha_color;
/// use rgb::Rgba;
///
/// // Index 0: black with 0% alpha
/// let color0 = cga_alpha_color(0);
/// assert_eq!(color0.r, 0.0);
/// assert_eq!(color0.a, 0.0);
///
/// // Index 63: white with 100% alpha
/// let color63 = cga_alpha_color(63);
/// assert_eq!(color63.r, 1.0);
/// assert_eq!(color63.a, 1.0);
/// ```
#[cfg(test)]
pub fn cga_alpha_color(index: usize) -> Rgba<f32> {
    let a: f32 = ((index & 0b110000) >> 4) as f32;
    let i: f32 = ((index & 0b001000) >> 3) as f32;
    let r: f32 = ((index & 0b000100) >> 2) as f32;
    let g: f32 = ((index & 0b000010) >> 1) as f32;
    let b: f32 = ((index & 0b000001) >> 0) as f32;

    let a: f32 = a / 3.0;
    let r: f32 = r * 2.0 / 3.0 + i / 3.0;
    let g: f32 = g * 2.0 / 3.0 + i / 3.0;
    let b: f32 = b * 2.0 / 3.0 + i / 3.0;
    let g: f32 = if index == 6 { g * 2.0 / 3.0 } else { g };
    return Rgba { r, g, b, a };
}
