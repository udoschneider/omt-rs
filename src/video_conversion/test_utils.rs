//! Tests Utils for video frame format conversion utilities.
//!
//! This module provides helper functions for generating test data in various pixel formats.
//! The helper functions create predictable color patterns based on CGA (Color Graphics Adapter)
//! color palette with alpha variations for testing video conversion functions.
//!
//! Also provides common YUV test utilities for generating test data in various YUV formats.

use rgb::*;

/// RGB test utilities for generating test data in various RGB formats.
#[cfg(test)]
pub mod rgb_utils {
    use super::*;

    /// Returns 48 bytes (16 RGB8 pixels) of test data.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt_rs::video_conversion::tests::rgb8_data;
    /// let data = rgb8_data();
    /// assert_eq!(data.len(), 48); // 16 pixels × 3 bytes each
    /// ```
    pub fn rgb8_data() -> Vec<u8> {
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
    pub fn rgba8_data() -> Vec<u8> {
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
    pub fn rgb16_data() -> Vec<u8> {
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
    pub fn rgba16_data() -> Vec<u8> {
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
    pub fn rgb8_colors() -> Vec<Rgb<u8>> {
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
    pub fn rgb16_colors() -> Vec<Rgb<u16>> {
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
    pub fn rgba16_colors() -> Vec<Rgba<u16>> {
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
    pub fn cga_colors() -> Vec<Rgb<f32>> {
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
}

/// YUV test utilities for generating test data in various YUV formats.
#[cfg(test)]
pub mod yuv_utils {
    use yuv::YuvRange;

    /// Get Y value for middle gray based on YUV range.
    ///
    /// # Arguments
    ///
    /// * `yuv_range` - The YUV range (Limited or Full)
    ///
    /// # Returns
    ///
    /// Y value for middle gray:
    /// - Limited range: 118 (middle of 16-235)
    /// - Full range: 128 (middle of 0-255)
    pub fn middle_gray_y(yuv_range: YuvRange) -> u8 {
        match yuv_range {
            YuvRange::Limited => 118,
            YuvRange::Full => 128,
        }
    }

    /// Get black and white Y values based on YUV range.
    ///
    /// # Arguments
    ///
    /// * `yuv_range` - The YUV range (Limited or Full)
    ///
    /// # Returns
    ///
    /// Tuple of (black_y, white_y):
    /// - Limited range: (16, 235)
    /// - Full range: (0, 255)
    pub fn black_white_y(yuv_range: YuvRange) -> (u8, u8) {
        match yuv_range {
            YuvRange::Limited => (16, 235),
            YuvRange::Full => (0, 255),
        }
    }

    /// Get Y value for a color bar based on index and YUV range.
    ///
    /// # Arguments
    ///
    /// * `bar_index` - Index of the color bar (0-7)
    /// * `yuv_range` - The YUV range (Limited or Full)
    ///
    /// # Returns
    ///
    /// Y value for the specified color bar:
    /// - 0: Black
    /// - 1: White
    /// - 2: Red (approximate)
    /// - 3: Green (approximate)
    /// - 4: Blue (approximate)
    /// - 5: Yellow (approximate)
    /// - 6: Cyan (approximate)
    /// - 7: Magenta (approximate)
    pub fn color_bar_y(bar_index: usize, yuv_range: YuvRange) -> u8 {
        let (black_y, white_y) = black_white_y(yuv_range);

        match bar_index {
            0 => black_y, // Black
            1 => white_y, // White
            2 => 76,      // Red (approximate)
            3 => 150,     // Green (approximate)
            4 => 29,      // Blue (approximate)
            5 => 225,     // Yellow (approximate)
            6 => 179,     // Cyan (approximate)
            7 => 105,     // Magenta (approximate)
            _ => black_y,
        }
    }

    /// Get UV values for a color bar based on index.
    ///
    /// # Arguments
    ///
    /// * `bar_index` - Index of the color bar (0-7)
    ///
    /// # Returns
    ///
    /// Tuple of (u_value, v_value) for the specified color bar:
    /// - 0: (128, 128) - Black
    /// - 1: (128, 128) - White
    /// - 2: (84, 255)  - Red
    /// - 3: (149, 43)  - Green
    /// - 4: (255, 107) - Blue
    /// - 5: (0, 148)   - Yellow
    /// - 6: (168, 0)   - Cyan
    /// - 7: (255, 212) - Magenta
    pub fn color_bar_uv(bar_index: usize) -> (u8, u8) {
        match bar_index {
            0 => (128, 128), // Black
            1 => (128, 128), // White
            2 => (84, 255),  // Red
            3 => (149, 43),  // Green
            4 => (255, 107), // Blue
            5 => (0, 148),   // Yellow
            6 => (168, 0),   // Cyan
            7 => (255, 212), // Magenta
            _ => (128, 128),
        }
    }

    /// Get neutral UV values (no color).
    ///
    /// # Returns
    ///
    /// Tuple of (u_value, v_value) = (128, 128)
    pub fn neutral_uv() -> (u8, u8) {
        (128, 128)
    }

    /// Test dimensions for packed 4:2:2 formats (UYVY, YUY2).
    ///
    /// Returns a vector of (width, height) pairs with even widths
    /// as required by packed 4:2:2 formats.
    pub fn packed_422_test_dimensions() -> Vec<(usize, usize)> {
        vec![
            (2, 2),   // Minimum size (even width required)
            (4, 4),   // Small even dimensions
            (6, 4),   // Even width, even height
            (8, 6),   // Even dimensions
            (16, 9),  // Common aspect ratio (even width)
            (32, 24), // Larger dimensions
        ]
    }

    /// Test dimensions for planar 4:2:0 formats (NV12, YV12).
    ///
    /// Returns a vector of (width, height) pairs with even widths
    /// and heights as required by 4:2:0 chroma subsampling.
    pub fn planar_420_test_dimensions() -> Vec<(usize, usize)> {
        vec![
            (2, 2),   // Minimum size for 4:2:0 (must be even)
            (4, 4),   // Small even dimensions
            (6, 4),   // Even width, even height
            (8, 6),   // Even dimensions
            (16, 8),  // Common aspect ratio with even height
            (32, 24), // Larger dimensions
        ]
    }
}
