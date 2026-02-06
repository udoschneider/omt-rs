//! UYVY video frame conversion functions.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvPackedImage, YuvRange, YuvStandardMatrix};

pub fn uyvy_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    let yuy_stride = stride as u32;

    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride,
        width: width as u32,
        height: height as u32,
    };

    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];
    let rgb_stride = (width * 3) as u32;

    yuv::uyvy422_to_rgb(
        &packed_image,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn uyvy_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    let packed_image = YuvPackedImage {
        yuy: raw_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];
    let rgba_stride = (width * 4) as u32;

    yuv::uyvy422_to_rgba(
        &packed_image,
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
    use super::super::test_utls::yuv_utils;
    use super::*;
    use yuv::YuvRange::*;
    use yuv::YuvStandardMatrix::*;

    /// Create simple test UYVY data for gray image
    fn create_gray_uyvy_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For UYVY, each 2 pixels requires 4 bytes: U0, Y0, V0, Y1
        let data_size = width * height * 2;
        let mut uyvy_data = vec![0u8; data_size];

        // Set Y values to middle gray
        let y_value = yuv_utils::middle_gray_y(yuv_range);

        // Set UV values to neutral (128, 128) - no color
        for i in 0..(width * height / 2) {
            let base_idx = i * 4;
            let (u_value, v_value) = yuv_utils::neutral_uv();
            uyvy_data[base_idx] = u_value; // U
            uyvy_data[base_idx + 1] = y_value; // Y0
            uyvy_data[base_idx + 2] = v_value; // V
            uyvy_data[base_idx + 3] = y_value; // Y1
        }

        uyvy_data
    }

    /// Create simple test UYVY data for color bars
    fn create_color_bars_uyvy_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        // For UYVY, each 2 pixels requires 4 bytes: U0, Y0, V0, Y1
        let data_size = width * height * 2;
        let mut uyvy_data = vec![0u8; data_size];

        // Y values for different colors in limited/full range
        let _ = yuv_utils::black_white_y(yuv_range); // Used by color_bar_y internally

        // Create simple color bars: black, white, red, green, blue, yellow, cyan, magenta
        // Note: bar_width = width / 8 (not used directly but implied by bar_index calculation)

        // Fill UYVY data with color bars
        for y in 0..height {
            for x in (0..width).step_by(2) {
                let bar_index0 = (x * 8) / width;
                let bar_index1 = ((x + 1) * 8) / width;

                let y_value0 = yuv_utils::color_bar_y(bar_index0, yuv_range);

                let y_value1 = yuv_utils::color_bar_y(bar_index1, yuv_range);

                // Use average UV values for the two pixels
                let (u_value, v_value) = yuv_utils::color_bar_uv(bar_index0);

                let base_idx = (y * width + x) * 2;
                uyvy_data[base_idx] = u_value; // U
                uyvy_data[base_idx + 1] = y_value0; // Y0
                uyvy_data[base_idx + 2] = v_value; // V
                uyvy_data[base_idx + 3] = y_value1; // Y1
            }
        }

        uyvy_data
    }

    #[test]
    fn test_uyvy_to_rgb8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Limited);

        // Convert UYVY to RGB8
        let actual_rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "uyvy_to_rgb8 should return Some for BT601 Limited range"
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
    fn test_uyvy_to_rgb8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Full);

        // Convert UYVY to RGB8
        let actual_rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "uyvy_to_rgb8 should return Some for BT601 Full range"
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
    fn test_uyvy_to_rgb8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Limited);

        // Convert UYVY to RGB8
        let actual_rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "uyvy_to_rgb8 should return Some for BT709 Limited range"
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
    fn test_uyvy_to_rgb8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Full);

        // Convert UYVY to RGB8
        let actual_rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "uyvy_to_rgb8 should return Some for BT709 Full range"
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
    fn test_uyvy_to_rgba8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Limited);

        // Convert UYVY to RGBA8
        let actual_rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "uyvy_to_rgba8 should return Some for BT601 Limited range"
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
    fn test_uyvy_to_rgba8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Full);

        // Convert UYVY to RGBA8
        let actual_rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "uyvy_to_rgba8 should return Some for BT601 Full range"
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
    fn test_uyvy_to_rgba8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Limited);

        // Convert UYVY to RGBA8
        let actual_rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "uyvy_to_rgba8 should return Some for BT709 Limited range"
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
    fn test_uyvy_to_rgba8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image
        let uyvy_data = create_gray_uyvy_data(width, height, Full);

        // Convert UYVY to RGBA8
        let actual_rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "uyvy_to_rgba8 should return Some for BT709 Full range"
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
    fn test_uyvy_conversion_with_color_bars() {
        let width = 16;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Test with BT601 Limited range
        let uyvy_data = create_color_bars_uyvy_data(width, height, Limited);

        // Convert UYVY to RGB8
        let rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Limited, Bt601);
        assert!(
            rgb_result.is_some(),
            "uyvy_to_rgb8 should return Some for color bars"
        );

        // Convert UYVY to RGBA8
        let rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Limited, Bt601);
        assert!(
            rgba_result.is_some(),
            "uyvy_to_rgba8 should return Some for color bars"
        );

        let rgb_colors = rgb_result.unwrap();
        let rgba_colors = rgba_result.unwrap();

        // Verify dimensions
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
        let rgb_result_bt709 = uyvy_to_rgb8(&uyvy_data, width, height, stride, Limited, Bt709);
        let rgba_result_bt709 = uyvy_to_rgba8(&uyvy_data, width, height, stride, Limited, Bt709);

        assert!(
            rgb_result_bt709.is_some(),
            "uyvy_to_rgb8 should return Some for BT709"
        );
        assert!(
            rgba_result_bt709.is_some(),
            "uyvy_to_rgba8 should return Some for BT709"
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
    fn test_uyvy_conversion_edge_cases() {
        // Test various image dimensions
        // UYVY is a packed 4:2:2 format that requires even width
        let test_dimensions = yuv_utils::packed_422_test_dimensions();

        for (width, height) in test_dimensions {
            let stride = width * 2; // UYVY uses 2 bytes per pixel

            // Create test data for gray image
            let uyvy_data = create_gray_uyvy_data(width, height, Limited);

            // Convert UYVY to RGB8
            let rgb_result = uyvy_to_rgb8(&uyvy_data, width, height, stride, Limited, Bt601);
            assert!(
                rgb_result.is_some(),
                "uyvy_to_rgb8 should return Some for {}x{}",
                width,
                height
            );

            // Convert UYVY to RGBA8
            let rgba_result = uyvy_to_rgba8(&uyvy_data, width, height, stride, Limited, Bt601);
            assert!(
                rgba_result.is_some(),
                "uyvy_to_rgba8 should return Some for {}x{}",
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
