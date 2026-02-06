//! YV12 video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvPlanarImage, YuvRange, YuvStandardMatrix};

pub fn yv12_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let y_size = height * stride;
    let uv_size = (height / 2) * (stride / 2);

    let y_plane = &raw_data[0..y_size];
    let v_plane = &raw_data[y_size..y_size + uv_size];
    let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    let swapped_image = YuvPlanarImage {
        y_plane,
        y_stride: stride as u32,
        u_plane: v_plane,
        u_stride: (stride / 2) as u32,
        v_plane: u_plane,
        v_stride: (stride / 2) as u32,
        width: width as u32,
        height: height as u32,
    };

    yuv::yuv420_to_rgb(
        &swapped_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn yv12_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let y_size = height * stride;
    let uv_size = (height / 2) * (stride / 2);

    let y_plane = &raw_data[0..y_size];
    let v_plane = &raw_data[y_size..y_size + uv_size];
    let u_plane = &raw_data[y_size + uv_size..y_size + 2 * uv_size];

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    let swapped_image = YuvPlanarImage {
        y_plane,
        y_stride: stride as u32,
        u_plane: v_plane,
        u_stride: (stride / 2) as u32,
        v_plane: u_plane,
        v_stride: (stride / 2) as u32,
        width: width as u32,
        height: height as u32,
    };

    yuv::yuv420_to_rgba(
        &swapped_image,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgba_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yuv::YuvRange::*;
    use yuv::YuvStandardMatrix::*;

    /// Create simple test YV12 data for gray image
    fn create_gray_yv12_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For YV12, we need Y plane, then V plane, then U plane
        let y_plane_size = width * height;
        let uv_plane_size = (width * height) / 4; // 4:2:0 chroma subsampling
        let mut yv12_data = vec![0u8; y_plane_size + uv_plane_size * 2];

        // Set Y plane to middle gray
        let y_value = match yuv_range {
            Limited => 118, // Middle gray in limited range (16-235)
            Full => 128,    // Middle gray in full range (0-255)
        };

        for i in 0..y_plane_size {
            yv12_data[i] = y_value;
        }

        // Set V and U planes to neutral (128, 128) - no color
        let v_plane_start = y_plane_size;
        let u_plane_start = y_plane_size + uv_plane_size;

        for i in 0..uv_plane_size {
            yv12_data[v_plane_start + i] = 128; // V plane
            yv12_data[u_plane_start + i] = 128; // U plane
        }

        yv12_data
    }

    /// Create simple test YV12 data for color bars
    fn create_color_bars_yv12_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For YV12, we need Y plane, then V plane, then U plane
        let y_plane_size = width * height;
        let uv_plane_size = (width * height) / 4; // 4:2:0 chroma subsampling
        let mut yv12_data = vec![0u8; y_plane_size + uv_plane_size * 2];

        // Y values for different colors in limited/full range
        let (black_y, white_y) = match yuv_range {
            Limited => (16, 235),
            Full => (0, 255),
        };

        // Create simple color bars: black, white, red, green, blue, yellow, cyan, magenta
        // Note: bar_width = width / 8 (not used directly but implied by bar_index calculation)

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
                yv12_data[y * width + x] = y_value;
            }
        }

        // Fill V and U planes with approximate values for color bars
        // This is simplified - real UV values would be more precise
        let v_plane_start = y_plane_size;
        let u_plane_start = y_plane_size + uv_plane_size;

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

                let uv_index = (y / 2) * (width / 2) + (x / 2);
                yv12_data[v_plane_start + uv_index] = v_value; // V plane
                yv12_data[u_plane_start + uv_index] = u_value; // U plane
            }
        }

        yv12_data
    }

    #[test]
    fn test_yv12_to_rgb8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Limited);

        // Convert YV12 to RGB8
        let actual_rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "yv12_to_rgb8 should return Some for BT601 Limited range"
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
    fn test_yv12_to_rgb8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Full);

        // Convert YV12 to RGB8
        let actual_rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "yv12_to_rgb8 should return Some for BT601 Full range"
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
    fn test_yv12_to_rgb8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Limited);

        // Convert YV12 to RGB8
        let actual_rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "yv12_to_rgb8 should return Some for BT709 Limited range"
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
    fn test_yv12_to_rgb8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Full);

        // Convert YV12 to RGB8
        let actual_rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "yv12_to_rgb8 should return Some for BT709 Full range"
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
    fn test_yv12_to_rgba8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Limited);

        // Convert YV12 to RGBA8
        let actual_rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "yv12_to_rgba8 should return Some for BT601 Limited range"
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
    fn test_yv12_to_rgba8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Full);

        // Convert YV12 to RGBA8
        let actual_rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "yv12_to_rgba8 should return Some for BT601 Full range"
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
    fn test_yv12_to_rgba8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Limited);

        // Convert YV12 to RGBA8
        let actual_rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "yv12_to_rgba8 should return Some for BT709 Limited range"
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
    fn test_yv12_to_rgba8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width;

        // Create test data for gray image
        let yv12_data = create_gray_yv12_data(width, height, Full);

        // Convert YV12 to RGBA8
        let actual_rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "yv12_to_rgba8 should return Some for BT709 Full range"
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
    fn test_yv12_conversion_with_color_bars() {
        let width = 16;
        let height = 8;
        let stride = width;

        // Test with BT601 Limited range
        let yv12_data = create_color_bars_yv12_data(width, height, Limited);

        // Convert YV12 to RGB8
        let rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Limited, Bt601);
        assert!(
            rgb_result.is_some(),
            "yv12_to_rgb8 should return Some for color bars"
        );

        // Convert YV12 to RGBA8
        let rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Limited, Bt601);
        assert!(
            rgba_result.is_some(),
            "yv12_to_rgba8 should return Some for color bars"
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

        // Verify RGB and RGBA produce same RGB values (RGBA should have alpha=255)
        for i in 0..rgb_colors.len() {
            let rgb = &rgb_colors[i];
            let rgba = &rgba_colors[i];

            assert_eq!(rgb.r, rgba.r, "Red mismatch at index {}", i);
            assert_eq!(rgb.g, rgba.g, "Green mismatch at index {}", i);
            assert_eq!(rgb.b, rgba.b, "Blue mismatch at index {}", i);
            assert_eq!(rgba.a, 255, "Alpha should be 255 at index {}", i);
        }

        // Test with BT709 as well
        let rgb_result_bt709 = yv12_to_rgb8(&yv12_data, width, height, stride, Limited, Bt709);
        let rgba_result_bt709 = yv12_to_rgba8(&yv12_data, width, height, stride, Limited, Bt709);

        assert!(
            rgb_result_bt709.is_some(),
            "yv12_to_rgb8 should return Some for BT709"
        );
        assert!(
            rgba_result_bt709.is_some(),
            "yv12_to_rgba8 should return Some for BT709"
        );

        let rgb_colors_bt709 = rgb_result_bt709.unwrap();
        let rgba_colors_bt709 = rgba_result_bt709.unwrap();

        assert_eq!(
            rgb_colors_bt709.len(),
            width * height,
            "BT709 RGB8 output should have correct number of pixels"
        );
        assert_eq!(
            rgba_colors_bt709.len(),
            width * height,
            "BT709 RGBA8 output should have correct number of pixels"
        );
    }

    #[test]
    fn test_yv12_conversion_edge_cases() {
        // Test various image dimensions
        // YV12 is a planar 4:2:0 format that requires even width and height
        let test_dimensions = vec![
            (2, 2),   // Minimum size for 4:2:0 (must be even)
            (4, 4),   // Small even dimensions
            (6, 4),   // Even width, even height
            (8, 6),   // Even dimensions
            (16, 8),  // Common aspect ratio with even height
            (32, 24), // Larger dimensions
        ];

        for (width, height) in test_dimensions {
            let stride = width;

            // Create test data for gray image
            let yv12_data = create_gray_yv12_data(width, height, Limited);

            // Convert YV12 to RGB8
            let rgb_result = yv12_to_rgb8(&yv12_data, width, height, stride, Limited, Bt601);
            assert!(
                rgb_result.is_some(),
                "yv12_to_rgb8 should return Some for {}x{}",
                width,
                height
            );

            // Convert YV12 to RGBA8
            let rgba_result = yv12_to_rgba8(&yv12_data, width, height, stride, Limited, Bt601);
            assert!(
                rgba_result.is_some(),
                "yv12_to_rgba8 should return Some for {}x{}",
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

            // All pixels should be gray (R=G=B) since UV plane is neutral
            for (i, color) in rgb_colors.iter().enumerate() {
                assert!(
                    color.r == color.g && color.g == color.b,
                    "Pixel at index {} should be gray for {}x{}: R={}, G={}, B={}",
                    i,
                    width,
                    height,
                    color.r,
                    color.g,
                    color.b
                );
            }

            // RGBA pixels should have alpha=255
            for (i, color) in rgba_colors.iter().enumerate() {
                assert_eq!(
                    color.a, 255,
                    "Alpha should be 255 at index {} for {}x{}: actual {}",
                    i, width, height, color.a
                );
            }
        }
    }
}
