//! NV12 video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvBiPlanarImage, YuvConversionMode, YuvRange, YuvStandardMatrix};

pub fn nv12_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let y_plane = &raw_data[0..height * stride];
    let uv_plane = &raw_data[height * stride..];

    let bi_planar_image = YuvBiPlanarImage {
        y_plane,
        y_stride: stride as u32,
        uv_plane,
        uv_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::yuv_nv12_to_rgb(
        &bi_planar_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
        YuvConversionMode::Balanced,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn nv12_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let y_plane = &raw_data[0..height * stride];
    let uv_plane = &raw_data[height * stride..];

    let bi_planar_image = YuvBiPlanarImage {
        y_plane,
        y_stride: stride as u32,
        uv_plane,
        uv_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::yuv_nv12_to_rgba(
        &bi_planar_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
        YuvConversionMode::Balanced,
    )
    .ok()?;

    Some(rgba_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yuv::YuvRange::*;
    use yuv::YuvStandardMatrix::*;

    /// Create simple test NV12 data for gray image
    fn create_gray_nv12_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For NV12, we need Y plane followed by interleaved UV plane
        let y_plane_size = width * height;
        let uv_plane_size = (width * height) / 2;
        let mut nv12_data = vec![0u8; y_plane_size + uv_plane_size * 2];

        // Set Y plane to middle gray
        let y_value = match yuv_range {
            Limited => 118, // Middle gray in limited range (16-235)
            Full => 128,    // Middle gray in full range (0-255)
        };

        for i in 0..y_plane_size {
            nv12_data[i] = y_value;
        }

        // Set UV plane to neutral (128, 128) - no color
        for i in 0..uv_plane_size {
            nv12_data[y_plane_size + i * 2] = 128; // U
            nv12_data[y_plane_size + i * 2 + 1] = 128; // V
        }

        nv12_data
    }

    /// Create simple test NV12 data for color bars
    fn create_color_bars_nv12_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For NV12, we need Y plane followed by interleaved UV plane
        let y_plane_size = width * height;
        let uv_plane_size = (width * height) / 2;
        let mut nv12_data = vec![0u8; y_plane_size + uv_plane_size * 2];

        // Create simple color bars: black, white, red, green, blue, yellow, cyan, magenta
        // Note: bar_width = width / 8 (not used directly but implied by bar_index calculation)

        // Y values for different colors in limited/full range
        let (black_y, white_y) = match yuv_range {
            Limited => (16, 235),
            Full => (0, 255),
        };

        // Fill Y plane with color bars
        for y in 0..height {
            for x in 0..width {
                let bar_index = (x * 8) / width;
                let y_value = match bar_index {
                    0 => black_y, // Black
                    1 => white_y, // White
                    2 => 76,      // Red (approximate)
                    3 => 150,     // Green (approximate)
                    4 => 29,      // Blue (approximate)
                    5 => 225,     // Yellow (approximate)
                    6 => 179,     // Cyan (approximate)
                    7 => 105,     // Magenta (approximate)
                    _ => black_y,
                };
                nv12_data[y * width + x] = y_value;
            }
        }

        // Fill UV plane with approximate values for color bars
        // This is simplified - real UV values would be more precise
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                let bar_index = (x * 8) / width;
                let (u_value, v_value) = match bar_index {
                    0 => (128, 128), // Black
                    1 => (128, 128), // White
                    2 => (84, 255),  // Red
                    3 => (149, 43),  // Green
                    4 => (255, 107), // Blue
                    5 => (0, 148),   // Yellow
                    6 => (168, 0),   // Cyan
                    7 => (255, 212), // Magenta
                    _ => (128, 128),
                };

                let uv_index = y_plane_size + (y / 2) * width + x;
                nv12_data[uv_index] = u_value;
                nv12_data[uv_index + 1] = v_value;
            }
        }

        nv12_data
    }

    #[test]
    fn test_nv12_to_rgb8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Limited);

        // Convert NV12 to RGB8
        let actual_rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "nv12_to_rgb8 should return Some for BT601 Limited range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgb_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) since UV plane is neutral
        for (i, color) in actual_rgb_colors.iter().enumerate() {
            // With neutral UV (128, 128) and Y=118 (limited range middle gray),
            // we should get a gray value
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn test_nv12_to_rgb8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Full);

        // Convert NV12 to RGB8
        let actual_rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "nv12_to_rgb8 should return Some for BT601 Full range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgb_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) since UV plane is neutral
        for (i, color) in actual_rgb_colors.iter().enumerate() {
            // With neutral UV (128, 128) and Y=128 (full range middle gray),
            // we should get a gray value
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn test_nv12_to_rgb8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Limited);

        // Convert NV12 to RGB8
        let actual_rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "nv12_to_rgb8 should return Some for BT709 Limited range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgb_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) since UV plane is neutral
        for (i, color) in actual_rgb_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn test_nv12_to_rgb8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Full);

        // Convert NV12 to RGB8
        let actual_rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "nv12_to_rgb8 should return Some for BT709 Full range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgb_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) since UV plane is neutral
        for (i, color) in actual_rgb_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn test_nv12_to_rgba8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Limited);

        // Convert NV12 to RGBA8
        let actual_rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "nv12_to_rgba8 should return Some for BT601 Limited range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgba_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) and alpha should be 255
        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
            assert_eq!(
                color.a, 255,
                "Alpha should be 255 at index {}: actual {}",
                i, color.a
            );
        }
    }

    #[test]
    fn test_nv12_to_rgba8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Full);

        // Convert NV12 to RGBA8
        let actual_rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "nv12_to_rgba8 should return Some for BT601 Full range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgba_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) and alpha should be 255
        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
            assert_eq!(
                color.a, 255,
                "Alpha should be 255 at index {}: actual {}",
                i, color.a
            );
        }
    }

    #[test]
    fn test_nv12_to_rgba8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Limited);

        // Convert NV12 to RGBA8
        let actual_rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "nv12_to_rgba8 should return Some for BT709 Limited range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgba_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) and alpha should be 255
        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
            assert_eq!(
                color.a, 255,
                "Alpha should be 255 at index {}: actual {}",
                i, color.a
            );
        }
    }

    #[test]
    fn test_nv12_conversion_with_color_bars() {
        let width = 64;
        let height = 8;
        let stride = width;

        // Create test data for color bars
        let nv12_data = create_color_bars_nv12_data(width, height, Limited);

        // Test RGB8 conversion with BT601
        let rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Limited, Bt601);
        assert!(
            rgb_result.is_some(),
            "nv12_to_rgb8 should return Some for color bars"
        );

        // Test RGBA8 conversion with BT601
        let rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Limited, Bt601);
        assert!(
            rgba_result.is_some(),
            "nv12_to_rgba8 should return Some for color bars"
        );

        let rgb_colors = rgb_result.unwrap();
        let rgba_colors = rgba_result.unwrap();

        // Verify dimensions
        assert_eq!(
            rgb_colors.len(),
            width * height,
            "RGB8 output should have correct number of pixels"
        );
        assert_eq!(
            rgba_colors.len(),
            width * height,
            "RGBA8 output should have correct number of pixels"
        );

        // Verify RGB and RGBA conversions produce same RGB values
        for i in 0..rgb_colors.len() {
            let rgb = &rgb_colors[i];
            let rgba = &rgba_colors[i];

            assert_eq!(rgb.r, rgba.r, "Red mismatch at index {}", i);
            assert_eq!(rgb.g, rgba.g, "Green mismatch at index {}", i);
            assert_eq!(rgb.b, rgba.b, "Blue mismatch at index {}", i);
            assert_eq!(rgba.a, 255, "Alpha should be 255 at index {}", i);
        }

        // Test with BT709 as well
        let rgb_result_709 = nv12_to_rgb8(&nv12_data, width, height, stride, Limited, Bt709);
        assert!(
            rgb_result_709.is_some(),
            "nv12_to_rgb8 should return Some for color bars with BT709"
        );

        let rgba_result_709 = nv12_to_rgba8(&nv12_data, width, height, stride, Limited, Bt709);
        assert!(
            rgba_result_709.is_some(),
            "nv12_to_rgba8 should return Some for color bars with BT709"
        );
    }

    #[test]
    fn test_nv12_conversion_edge_cases() {
        // Test with different image dimensions
        let test_cases = vec![
            (4, 4),   // Small image
            (16, 16), // Power of two
            (2, 2),   // Minimum for 4:2:0
        ];

        for (width, height) in test_cases {
            let stride = width;

            // Create test data for gray image
            let nv12_data = create_gray_nv12_data(width, height, Limited);

            // Test RGB8 conversion
            let rgb_result = nv12_to_rgb8(&nv12_data, width, height, stride, Limited, Bt601);
            assert!(
                rgb_result.is_some(),
                "nv12_to_rgb8 should return Some for {}x{} image",
                width,
                height
            );

            // Test RGBA8 conversion
            let rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Limited, Bt601);
            assert!(
                rgba_result.is_some(),
                "nv12_to_rgba8 should return Some for {}x{} image",
                width,
                height
            );

            let rgb_colors = rgb_result.unwrap();
            let rgba_colors = rgba_result.unwrap();

            // Verify dimensions
            assert_eq!(
                rgb_colors.len(),
                width * height,
                "RGB8 output should have correct number of pixels for {}x{}",
                width,
                height
            );
            assert_eq!(
                rgba_colors.len(),
                width * height,
                "RGBA8 output should have correct number of pixels for {}x{}",
                width,
                height
            );
        }
    }

    #[test]
    fn test_nv12_to_rgba8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let nv12_data = create_gray_nv12_data(width, height, Full);

        // Convert NV12 to RGBA8
        let actual_rgba_result = nv12_to_rgba8(&nv12_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "nv12_to_rgba8 should return Some for BT709 Full range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(
            actual_rgba_colors.len(),
            width * height,
            "Number of pixels should match"
        );

        // All pixels should be gray (R=G=B) and alpha should be 255
        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
            assert_eq!(
                color.a, 255,
                "Alpha should be 255 at index {}: actual {}",
                i, color.a
            );
        }
    }
}
