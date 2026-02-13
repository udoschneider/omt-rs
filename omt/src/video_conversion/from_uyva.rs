//! UYVA video frame conversion functions.
//!
//! UYVA is a packed 4:2:2 YUV format (UYVY) immediately followed by an alpha plane.
//! The UYVY portion uses 2 bytes per pixel (16bpp), and the alpha plane uses 1 byte per pixel.
//! Total data size: width * height * 2 (UYVY) + width * height (alpha) = width * height * 3 bytes.

use rgb::bytemuck;
use rgb::*;
use yuv::{YuvPackedImage, YuvRange, YuvStandardMatrix};

/// Convert UYVA data to RGB8 format.
///
/// Since RGB8 has no alpha channel, this function simply converts the UYVY portion
/// and discards the alpha plane.
///
/// # Arguments
///
/// * `raw_data` - The raw UYVA data (UYVY portion followed by alpha plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the UYVY portion
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGB8>)` on success, or `None` if the conversion fails.
pub fn uyva_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB8>> {
    // UYVA = UYVY + alpha plane
    // UYVY size: height * stride (where stride >= width * 2)
    let uyvy_size = height * stride;

    // Validate that we have enough data
    let alpha_size = width * height;
    if raw_data.len() < uyvy_size + alpha_size {
        return None;
    }

    // Extract the UYVY portion (alpha is discarded for RGB output)
    let uyvy_data = &raw_data[0..uyvy_size];

    let packed_image = YuvPackedImage {
        yuy: uyvy_data,
        yuy_stride: stride as u32,
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

/// Convert UYVA data to RGBA8 format.
///
/// This function converts the UYVY portion to RGB and then applies the alpha values
/// from the alpha plane.
///
/// # Arguments
///
/// * `raw_data` - The raw UYVA data (UYVY portion followed by alpha plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the UYVY portion
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGBA8>)` on success, or `None` if the conversion fails.
pub fn uyva_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA8>> {
    // UYVA = UYVY + alpha plane
    // UYVY size: height * stride (where stride >= width * 2)
    let uyvy_size = height * stride;

    // Alpha plane size: width * height
    let alpha_size = width * height;

    // Validate that we have enough data
    if raw_data.len() < uyvy_size + alpha_size {
        return None;
    }

    // Extract the UYVY and alpha portions
    let uyvy_data = &raw_data[0..uyvy_size];
    let alpha_data = &raw_data[uyvy_size..uyvy_size + alpha_size];

    let packed_image = YuvPackedImage {
        yuy: uyvy_data,
        yuy_stride: stride as u32,
        width: width as u32,
        height: height as u32,
    };

    // First convert UYVY to RGBA (alpha will be set to 255 by the converter)
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

    // Apply alpha values from the alpha plane
    for (pixel, &alpha) in rgba_data.iter_mut().zip(alpha_data.iter()) {
        pixel.a = alpha;
    }

    Some(rgba_data)
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::yuv_utils;
    use super::*;
    use yuv::YuvRange::*;
    use yuv::YuvStandardMatrix::*;

    /// Create simple test UYVA data for gray image with varying alpha
    fn create_gray_uyva_data(
        width: usize,
        height: usize,
        yuv_range: YuvRange,
        alpha_value: u8,
    ) -> Vec<u8> {
        // UYVY portion: 2 bytes per pixel
        let uyvy_size = width * height * 2;
        // Alpha plane: 1 byte per pixel
        let alpha_size = width * height;
        let total_size = uyvy_size + alpha_size;

        let mut uyva_data = vec![0u8; total_size];

        // Set Y values to middle gray
        let y_value = yuv_utils::middle_gray_y(yuv_range);

        // Set UV values to neutral (128, 128) - no color
        for i in 0..(width * height / 2) {
            let base_idx = i * 4;
            let (u_value, v_value) = yuv_utils::neutral_uv();
            uyva_data[base_idx] = u_value; // U
            uyva_data[base_idx + 1] = y_value; // Y0
            uyva_data[base_idx + 2] = v_value; // V
            uyva_data[base_idx + 3] = y_value; // Y1
        }

        // Fill alpha plane with the specified alpha value
        for i in uyvy_size..total_size {
            uyva_data[i] = alpha_value;
        }

        uyva_data
    }

    /// Create UYVA data with gradient alpha
    fn create_gradient_alpha_uyva_data(
        width: usize,
        height: usize,
        yuv_range: YuvRange,
    ) -> Vec<u8> {
        // UYVY portion: 2 bytes per pixel
        let uyvy_size = width * height * 2;
        // Alpha plane: 1 byte per pixel
        let alpha_size = width * height;
        let total_size = uyvy_size + alpha_size;

        let mut uyva_data = vec![0u8; total_size];

        // Set Y values to middle gray
        let y_value = yuv_utils::middle_gray_y(yuv_range);

        // Set UV values to neutral (128, 128) - no color
        for i in 0..(width * height / 2) {
            let base_idx = i * 4;
            let (u_value, v_value) = yuv_utils::neutral_uv();
            uyva_data[base_idx] = u_value; // U
            uyva_data[base_idx + 1] = y_value; // Y0
            uyva_data[base_idx + 2] = v_value; // V
            uyva_data[base_idx + 3] = y_value; // Y1
        }

        // Fill alpha plane with gradient (row-based)
        for y in 0..height {
            let alpha = ((y * 255) / height.max(1)) as u8;
            for x in 0..width {
                uyva_data[uyvy_size + y * width + x] = alpha;
            }
        }

        uyva_data
    }

    /// Create simple test UYVA data for color bars with specified alpha
    fn create_color_bars_uyva_data(
        width: usize,
        height: usize,
        yuv_range: YuvRange,
        alpha_value: u8,
    ) -> Vec<u8> {
        // UYVY portion: 2 bytes per pixel
        let uyvy_size = width * height * 2;
        // Alpha plane: 1 byte per pixel
        let alpha_size = width * height;
        let total_size = uyvy_size + alpha_size;

        let mut uyva_data = vec![0u8; total_size];

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
                uyva_data[base_idx] = u_value; // U
                uyva_data[base_idx + 1] = y_value0; // Y0
                uyva_data[base_idx + 2] = v_value; // V
                uyva_data[base_idx + 3] = y_value1; // Y1
            }
        }

        // Fill alpha plane with the specified alpha value
        for i in uyvy_size..total_size {
            uyva_data[i] = alpha_value;
        }

        uyva_data
    }

    #[test]
    fn test_uyva_to_rgb8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // UYVY uses 2 bytes per pixel

        // Create test data for gray image (alpha is ignored for RGB)
        let uyva_data = create_gray_uyva_data(width, height, Limited, 128);

        // Convert UYVA to RGB8
        let actual_rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "uyva_to_rgb8 should return Some for BT601 Limited range"
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
    fn test_uyva_to_rgb8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let uyva_data = create_gray_uyva_data(width, height, Full, 255);

        let actual_rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgb_result.is_some(),
            "uyva_to_rgb8 should return Some for BT601 Full range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();
        assert_eq!(actual_rgb_colors.len(), width * height);

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
    fn test_uyva_to_rgb8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let uyva_data = create_gray_uyva_data(width, height, Limited, 64);

        let actual_rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "uyva_to_rgb8 should return Some for BT709 Limited range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();
        assert_eq!(actual_rgb_colors.len(), width * height);

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
    fn test_uyva_to_rgb8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let uyva_data = create_gray_uyva_data(width, height, Full, 200);

        let actual_rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgb_result.is_some(),
            "uyva_to_rgb8 should return Some for BT709 Full range"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();
        assert_eq!(actual_rgb_colors.len(), width * height);

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
    fn test_uyva_to_rgba8_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 128u8;

        let uyva_data = create_gray_uyva_data(width, height, Limited, expected_alpha);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "uyva_to_rgba8 should return Some for BT601 Limited range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();
        assert_eq!(actual_rgba_colors.len(), width * height);

        for (i, color) in actual_rgba_colors.iter().enumerate() {
            // Check gray values
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
            // Check alpha value
            assert_eq!(
                color.a, expected_alpha,
                "Pixel at index {} should have alpha={}, got {}",
                i, expected_alpha, color.a
            );
        }
    }

    #[test]
    fn test_uyva_to_rgba8_bt601_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 255u8;

        let uyva_data = create_gray_uyva_data(width, height, Full, expected_alpha);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Full, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "uyva_to_rgba8 should return Some for BT601 Full range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();
        assert_eq!(actual_rgba_colors.len(), width * height);

        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray",
                i
            );
            assert_eq!(
                color.a, expected_alpha,
                "Alpha should be {}",
                expected_alpha
            );
        }
    }

    #[test]
    fn test_uyva_to_rgba8_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 64u8;

        let uyva_data = create_gray_uyva_data(width, height, Limited, expected_alpha);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "uyva_to_rgba8 should return Some for BT709 Limited range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();
        assert_eq!(actual_rgba_colors.len(), width * height);

        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray",
                i
            );
            assert_eq!(
                color.a, expected_alpha,
                "Alpha should be {}",
                expected_alpha
            );
        }
    }

    #[test]
    fn test_uyva_to_rgba8_bt709_full() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 200u8;

        let uyva_data = create_gray_uyva_data(width, height, Full, expected_alpha);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Full, Bt709);
        assert!(
            actual_rgba_result.is_some(),
            "uyva_to_rgba8 should return Some for BT709 Full range"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();
        assert_eq!(actual_rgba_colors.len(), width * height);

        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert!(
                color.r == color.g && color.g == color.b,
                "Pixel at index {} should be gray",
                i
            );
            assert_eq!(
                color.a, expected_alpha,
                "Alpha should be {}",
                expected_alpha
            );
        }
    }

    #[test]
    fn test_uyva_to_rgba8_gradient_alpha() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let uyva_data = create_gradient_alpha_uyva_data(width, height, Limited);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(
            actual_rgba_result.is_some(),
            "uyva_to_rgba8 should return Some for gradient alpha"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();
        assert_eq!(actual_rgba_colors.len(), width * height);

        // Verify alpha gradient (each row should have same alpha, increasing down)
        for y in 0..height {
            let expected_alpha = ((y * 255) / height.max(1)) as u8;
            for x in 0..width {
                let idx = y * width + x;
                assert_eq!(
                    actual_rgba_colors[idx].a, expected_alpha,
                    "Alpha at ({}, {}) should be {}, got {}",
                    x, y, expected_alpha, actual_rgba_colors[idx].a
                );
            }
        }
    }

    #[test]
    fn test_uyva_to_rgba8_zero_alpha() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 0u8;

        let uyva_data = create_gray_uyva_data(width, height, Limited, expected_alpha);

        let actual_rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(actual_rgba_result.is_some());

        let actual_rgba_colors = actual_rgba_result.unwrap();
        for (i, color) in actual_rgba_colors.iter().enumerate() {
            assert_eq!(
                color.a, expected_alpha,
                "Pixel at index {} should have alpha=0",
                i
            );
        }
    }

    #[test]
    fn test_uyva_conversion_with_color_bars() {
        let width = 16;
        let height = 8;
        let stride = width * 2;

        let uyva_data = create_color_bars_uyva_data(width, height, Limited, 192);

        // Test RGB conversion
        let rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(rgb_result.is_some(), "RGB conversion should succeed");
        let rgb_colors = rgb_result.unwrap();
        assert_eq!(rgb_colors.len(), width * height);

        // Test RGBA conversion
        let rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(rgba_result.is_some(), "RGBA conversion should succeed");
        let rgba_colors = rgba_result.unwrap();
        assert_eq!(rgba_colors.len(), width * height);

        // Verify RGB values match between RGB and RGBA outputs
        for (i, (rgb, rgba)) in rgb_colors.iter().zip(rgba_colors.iter()).enumerate() {
            assert_eq!(
                rgb.r, rgba.r,
                "Red mismatch at index {}: RGB={}, RGBA={}",
                i, rgb.r, rgba.r
            );
            assert_eq!(
                rgb.g, rgba.g,
                "Green mismatch at index {}: RGB={}, RGBA={}",
                i, rgb.g, rgba.g
            );
            assert_eq!(
                rgb.b, rgba.b,
                "Blue mismatch at index {}: RGB={}, RGBA={}",
                i, rgb.b, rgba.b
            );
            // Check alpha
            assert_eq!(rgba.a, 192, "Alpha should be 192 at index {}", i);
        }
    }

    #[test]
    fn test_uyva_conversion_edge_cases() {
        // Test minimum valid dimensions (2x2 for packed 4:2:2)
        let width = 2;
        let height = 2;
        let stride = width * 2;
        let uyva_data = create_gray_uyva_data(width, height, Limited, 255);

        let rgb_result = uyva_to_rgb8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(rgb_result.is_some(), "2x2 RGB conversion should work");
        assert_eq!(rgb_result.unwrap().len(), 4);

        let rgba_result = uyva_to_rgba8(&uyva_data, width, height, stride, Limited, Bt601);
        assert!(rgba_result.is_some(), "2x2 RGBA conversion should work");
        assert_eq!(rgba_result.unwrap().len(), 4);

        // Test with various sizes
        for (w, h) in yuv_utils::packed_422_test_dimensions() {
            let stride = w * 2;
            let data = create_gray_uyva_data(w, h, Limited, 128);

            let rgb_result = uyva_to_rgb8(&data, w, h, stride, Limited, Bt601);
            assert!(
                rgb_result.is_some(),
                "RGB conversion should work for {}x{}",
                w,
                h
            );
            assert_eq!(
                rgb_result.unwrap().len(),
                w * h,
                "RGB output size should match for {}x{}",
                w,
                h
            );

            let rgba_result = uyva_to_rgba8(&data, w, h, stride, Limited, Bt601);
            assert!(
                rgba_result.is_some(),
                "RGBA conversion should work for {}x{}",
                w,
                h
            );
            assert_eq!(
                rgba_result.unwrap().len(),
                w * h,
                "RGBA output size should match for {}x{}",
                w,
                h
            );
        }
    }

    #[test]
    fn test_uyva_insufficient_data() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        // Create data that's too small (missing alpha plane)
        let uyvy_only = vec![0u8; width * height * 2];

        let rgb_result = uyva_to_rgb8(&uyvy_only, width, height, stride, Limited, Bt601);
        assert!(
            rgb_result.is_none(),
            "Should fail when alpha plane is missing"
        );

        let rgba_result = uyva_to_rgba8(&uyvy_only, width, height, stride, Limited, Bt601);
        assert!(
            rgba_result.is_none(),
            "Should fail when alpha plane is missing"
        );

        // Create data that's partially missing
        let partial_data = vec![0u8; width * height * 2 + width * height / 2];

        let rgb_result2 = uyva_to_rgb8(&partial_data, width, height, stride, Limited, Bt601);
        assert!(
            rgb_result2.is_none(),
            "Should fail when alpha plane is incomplete"
        );

        let rgba_result2 = uyva_to_rgba8(&partial_data, width, height, stride, Limited, Bt601);
        assert!(
            rgba_result2.is_none(),
            "Should fail when alpha plane is incomplete"
        );
    }
}
