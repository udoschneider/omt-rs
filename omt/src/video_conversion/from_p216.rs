//! P216 and PA16 video frame conversion functions.
//!
//! P216 is a planar 4:2:2 YUV format with 16-bit Y plane followed by interleaved 16-bit UV plane.
//! PA16 is the same as P216 followed by an additional 16-bit alpha plane.
//!
//! Since the `yuv` crate doesn't have direct P216 decoding functions, we de-interleave the UV
//! plane into separate U and V planes and use the `i216_to_rgb16`/`i216_to_rgba16` functions.

use rgb::*;
use yuv::{YuvPlanarImage, YuvRange, YuvStandardMatrix};

/// De-interleave a 16-bit UV plane into separate U and V planes.
///
/// P216 stores UV data as interleaved pairs: U0, V0, U1, V1, ...
/// This function splits them into separate U and V planes for use with planar YUV functions.
fn deinterleave_uv_plane(uv_plane: &[u16], width: usize, height: usize) -> (Vec<u16>, Vec<u16>) {
    // For 4:2:2, UV width is half of Y width
    let uv_width = width.div_ceil(2);
    let uv_count = uv_width * height;

    let mut u_plane = Vec::with_capacity(uv_count);
    let mut v_plane = Vec::with_capacity(uv_count);

    // UV plane is interleaved: U0, V0, U1, V1, ...
    for row in 0..height {
        let row_start = row * uv_width * 2;
        for i in 0..uv_width {
            let idx = row_start + i * 2;
            if idx + 1 < uv_plane.len() {
                u_plane.push(uv_plane[idx]);
                v_plane.push(uv_plane[idx + 1]);
            }
        }
    }

    (u_plane, v_plane)
}

/// Convert P216 data to RGB16 format.
///
/// P216 is a planar 4:2:2 YUV format:
/// - 16-bit Y plane (full resolution)
/// - Interleaved 16-bit UV plane (half horizontal resolution)
///
/// # Arguments
///
/// * `raw_data` - The raw P216 data as bytes (Y plane followed by interleaved UV plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the Y plane
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGB16>)` on success, or `None` if the conversion fails.
pub fn p216_to_rgb16(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB16>> {
    // Stride is in bytes, but we're working with u16 values (2 bytes each)
    let y_stride_u16 = stride / 2;

    // Y plane size in u16 elements
    let y_plane_size = y_stride_u16 * height;

    // UV plane: half width, full height, interleaved (so width UV pairs per row)
    let uv_width = width.div_ceil(2);
    let uv_plane_size = uv_width * 2 * height; // *2 for interleaved U and V

    // Total size needed in u16 elements
    let total_u16_elements = y_plane_size + uv_plane_size;

    // Validate data size
    if raw_data.len() < total_u16_elements * 2 {
        return None;
    }

    // Reinterpret raw bytes as u16 slice
    let data_u16: &[u16] = bytemuck::cast_slice(&raw_data[..total_u16_elements * 2]);

    // Extract Y plane
    let y_plane = &data_u16[0..y_plane_size];

    // Extract interleaved UV plane
    let uv_plane = &data_u16[y_plane_size..y_plane_size + uv_plane_size];

    // De-interleave UV into separate U and V planes
    let (u_plane, v_plane) = deinterleave_uv_plane(uv_plane, width, height);

    // Create planar image for yuv crate
    let planar_image = YuvPlanarImage {
        y_plane,
        y_stride: y_stride_u16 as u32,
        u_plane: &u_plane,
        u_stride: uv_width as u32,
        v_plane: &v_plane,
        v_stride: uv_width as u32,
        width: width as u32,
        height: height as u32,
    };

    // Allocate output buffer
    let mut rgb_data = vec![0u16; width * height * 3];
    let rgb_stride = (width * 3) as u32;

    // Convert using yuv crate's i216_to_rgb16
    yuv::i216_to_rgb16(
        &planar_image,
        &mut rgb_data,
        rgb_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    // Convert u16 slice to RGB16 vec
    let rgb16_data: Vec<RGB16> = rgb_data
        .chunks_exact(3)
        .map(|chunk| RGB16::new(chunk[0], chunk[1], chunk[2]))
        .collect();

    Some(rgb16_data)
}

/// Convert P216 data to RGBA16 format.
///
/// P216 is a planar 4:2:2 YUV format:
/// - 16-bit Y plane (full resolution)
/// - Interleaved 16-bit UV plane (half horizontal resolution)
///
/// Alpha channel is set to maximum (65535) for all pixels.
///
/// # Arguments
///
/// * `raw_data` - The raw P216 data as bytes (Y plane followed by interleaved UV plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the Y plane
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGBA16>)` on success, or `None` if the conversion fails.
pub fn p216_to_rgba16(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA16>> {
    // Stride is in bytes, but we're working with u16 values (2 bytes each)
    let y_stride_u16 = stride / 2;

    // Y plane size in u16 elements
    let y_plane_size = y_stride_u16 * height;

    // UV plane: half width, full height, interleaved (so width UV pairs per row)
    let uv_width = width.div_ceil(2);
    let uv_plane_size = uv_width * 2 * height; // *2 for interleaved U and V

    // Total size needed in u16 elements
    let total_u16_elements = y_plane_size + uv_plane_size;

    // Validate data size
    if raw_data.len() < total_u16_elements * 2 {
        return None;
    }

    // Reinterpret raw bytes as u16 slice
    let data_u16: &[u16] = bytemuck::cast_slice(&raw_data[..total_u16_elements * 2]);

    // Extract Y plane
    let y_plane = &data_u16[0..y_plane_size];

    // Extract interleaved UV plane
    let uv_plane = &data_u16[y_plane_size..y_plane_size + uv_plane_size];

    // De-interleave UV into separate U and V planes
    let (u_plane, v_plane) = deinterleave_uv_plane(uv_plane, width, height);

    // Create planar image for yuv crate
    let planar_image = YuvPlanarImage {
        y_plane,
        y_stride: y_stride_u16 as u32,
        u_plane: &u_plane,
        u_stride: uv_width as u32,
        v_plane: &v_plane,
        v_stride: uv_width as u32,
        width: width as u32,
        height: height as u32,
    };

    // Allocate output buffer
    let mut rgba_data = vec![0u16; width * height * 4];
    let rgba_stride = (width * 4) as u32;

    // Convert using yuv crate's i216_to_rgba16
    yuv::i216_to_rgba16(
        &planar_image,
        &mut rgba_data,
        rgba_stride,
        yuv_range,
        yuv_matrix,
    )
    .ok()?;

    // Convert u16 slice to RGBA16 vec
    let rgba16_data: Vec<RGBA16> = rgba_data
        .chunks_exact(4)
        .map(|chunk| RGBA16::new(chunk[0], chunk[1], chunk[2], chunk[3]))
        .collect();

    Some(rgba16_data)
}

/// Convert PA16 data to RGB16 format.
///
/// PA16 is the same as P216 followed by an additional 16-bit alpha plane.
/// Since RGB16 has no alpha channel, this function simply converts the P216 portion
/// and discards the alpha plane.
///
/// # Arguments
///
/// * `raw_data` - The raw PA16 data as bytes (Y plane, interleaved UV plane, alpha plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the Y plane
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGB16>)` on success, or `None` if the conversion fails.
pub fn pa16_to_rgb16(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGB16>> {
    // Stride is in bytes, but we're working with u16 values (2 bytes each)
    let y_stride_u16 = stride / 2;

    // Y plane size in u16 elements
    let y_plane_size = y_stride_u16 * height;

    // UV plane: half width, full height, interleaved
    let uv_width = width.div_ceil(2);
    let uv_plane_size = uv_width * 2 * height;

    // Alpha plane size (same as Y plane dimensions)
    let alpha_plane_size = width * height;

    // Total size needed in u16 elements (P216 portion + alpha)
    let total_u16_elements = y_plane_size + uv_plane_size + alpha_plane_size;

    // Validate data size
    if raw_data.len() < total_u16_elements * 2 {
        return None;
    }

    // For RGB16 output, we just use the P216 portion and ignore alpha
    p216_to_rgb16(raw_data, width, height, stride, yuv_range, yuv_matrix)
}

/// Convert PA16 data to RGBA16 format.
///
/// PA16 is the same as P216 followed by an additional 16-bit alpha plane.
/// This function converts the P216 portion to RGB and applies the alpha values.
///
/// # Arguments
///
/// * `raw_data` - The raw PA16 data as bytes (Y plane, interleaved UV plane, alpha plane)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
/// * `stride` - The stride (bytes per row) of the Y plane
/// * `yuv_range` - The YUV range (limited or full)
/// * `yuv_matrix` - The YUV standard matrix (BT.601 or BT.709)
///
/// # Returns
///
/// Returns `Some(Vec<RGBA16>)` on success, or `None` if the conversion fails.
pub fn pa16_to_rgba16(
    raw_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    yuv_range: YuvRange,
    yuv_matrix: YuvStandardMatrix,
) -> Option<Vec<RGBA16>> {
    // Stride is in bytes, but we're working with u16 values (2 bytes each)
    let y_stride_u16 = stride / 2;

    // Y plane size in u16 elements
    let y_plane_size = y_stride_u16 * height;

    // UV plane: half width, full height, interleaved
    let uv_width = width.div_ceil(2);
    let uv_plane_size = uv_width * 2 * height;

    // Alpha plane size (same as Y plane dimensions, but using width not stride)
    let alpha_plane_size = width * height;

    // Total size needed in u16 elements (P216 portion + alpha)
    let total_u16_elements = y_plane_size + uv_plane_size + alpha_plane_size;

    // Validate data size
    if raw_data.len() < total_u16_elements * 2 {
        return None;
    }

    // First convert P216 portion to RGBA16 (alpha will be set to 65535)
    let mut rgba_data = p216_to_rgba16(raw_data, width, height, stride, yuv_range, yuv_matrix)?;

    // Extract alpha plane from raw data
    let p216_size = y_plane_size + uv_plane_size;
    let alpha_start = p216_size * 2; // Convert to byte offset
    let alpha_end = alpha_start + alpha_plane_size * 2;

    let alpha_plane: &[u16] = bytemuck::cast_slice(&raw_data[alpha_start..alpha_end]);

    // Apply alpha values from the alpha plane
    for (pixel, &alpha) in rgba_data.iter_mut().zip(alpha_plane.iter()) {
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

    /// Create P216 test data for a gray image.
    ///
    /// P216 layout:
    /// - Y plane: 16-bit values, full resolution
    /// - UV plane: interleaved 16-bit U, V pairs, half horizontal resolution
    fn create_gray_p216_data(width: usize, height: usize, yuv_range: YuvRange) -> Vec<u8> {
        let y_stride = width; // In u16 elements
        let uv_width = (width + 1) / 2;

        let y_plane_size = y_stride * height;
        let uv_plane_size = uv_width * 2 * height; // *2 for interleaved U, V

        let mut data = vec![0u16; y_plane_size + uv_plane_size];

        // Y value for middle gray (scaled to 16-bit)
        let y_value_8 = yuv_utils::middle_gray_y(yuv_range);
        let y_value = (y_value_8 as u16) << 8; // Scale 8-bit to 16-bit

        // UV neutral value (128 scaled to 16-bit)
        let uv_neutral = 128u16 << 8;

        // Fill Y plane
        for i in 0..y_plane_size {
            data[i] = y_value;
        }

        // Fill UV plane (interleaved)
        let uv_start = y_plane_size;
        for row in 0..height {
            for i in 0..uv_width {
                let idx = uv_start + row * uv_width * 2 + i * 2;
                data[idx] = uv_neutral; // U
                data[idx + 1] = uv_neutral; // V
            }
        }

        // Convert to bytes
        bytemuck::cast_slice(&data).to_vec()
    }

    /// Create PA16 test data for a gray image with specified alpha.
    fn create_gray_pa16_data(
        width: usize,
        height: usize,
        yuv_range: YuvRange,
        alpha_value: u16,
    ) -> Vec<u8> {
        let y_stride = width;
        let uv_width = (width + 1) / 2;

        let y_plane_size = y_stride * height;
        let uv_plane_size = uv_width * 2 * height;
        let alpha_plane_size = width * height;

        let mut data = vec![0u16; y_plane_size + uv_plane_size + alpha_plane_size];

        // Y value for middle gray (scaled to 16-bit)
        let y_value_8 = yuv_utils::middle_gray_y(yuv_range);
        let y_value = (y_value_8 as u16) << 8;

        // UV neutral value
        let uv_neutral = 128u16 << 8;

        // Fill Y plane
        for i in 0..y_plane_size {
            data[i] = y_value;
        }

        // Fill UV plane (interleaved)
        let uv_start = y_plane_size;
        for row in 0..height {
            for i in 0..uv_width {
                let idx = uv_start + row * uv_width * 2 + i * 2;
                data[idx] = uv_neutral; // U
                data[idx + 1] = uv_neutral; // V
            }
        }

        // Fill alpha plane
        let alpha_start = y_plane_size + uv_plane_size;
        for i in 0..alpha_plane_size {
            data[alpha_start + i] = alpha_value;
        }

        bytemuck::cast_slice(&data).to_vec()
    }

    #[test]
    fn test_p216_to_rgb16_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2; // Stride in bytes

        let p216_data = create_gray_p216_data(width, height, Limited);

        let result = p216_to_rgb16(&p216_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some(), "p216_to_rgb16 should return Some");

        let rgb_colors = result.unwrap();
        assert_eq!(rgb_colors.len(), width * height);

        // All pixels should be gray (R ≈ G ≈ B)
        for (i, color) in rgb_colors.iter().enumerate() {
            let diff_rg = (color.r as i32 - color.g as i32).abs();
            let diff_gb = (color.g as i32 - color.b as i32).abs();
            // Allow some tolerance due to conversion
            assert!(
                diff_rg < 512 && diff_gb < 512,
                "Pixel {} should be gray: R={}, G={}, B={}",
                i,
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn test_p216_to_rgb16_bt709_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let p216_data = create_gray_p216_data(width, height, Limited);

        let result = p216_to_rgb16(&p216_data, width, height, stride, Limited, Bt709);
        assert!(result.is_some(), "p216_to_rgb16 should return Some");

        let rgb_colors = result.unwrap();
        assert_eq!(rgb_colors.len(), width * height);
    }

    #[test]
    fn test_p216_to_rgba16_bt601_limited() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        let p216_data = create_gray_p216_data(width, height, Limited);

        let result = p216_to_rgba16(&p216_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some(), "p216_to_rgba16 should return Some");

        let rgba_colors = result.unwrap();
        assert_eq!(rgba_colors.len(), width * height);

        // Check that alpha is set to maximum (65535)
        for (i, color) in rgba_colors.iter().enumerate() {
            assert_eq!(
                color.a, 65535,
                "Pixel {} should have alpha=65535, got {}",
                i, color.a
            );
        }
    }

    #[test]
    fn test_pa16_to_rgb16() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let alpha = 32768u16; // 50% alpha

        let pa16_data = create_gray_pa16_data(width, height, Limited, alpha);

        let result = pa16_to_rgb16(&pa16_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some(), "pa16_to_rgb16 should return Some");

        let rgb_colors = result.unwrap();
        assert_eq!(rgb_colors.len(), width * height);

        // RGB output - alpha is ignored
        for (i, color) in rgb_colors.iter().enumerate() {
            let diff_rg = (color.r as i32 - color.g as i32).abs();
            let diff_gb = (color.g as i32 - color.b as i32).abs();
            assert!(diff_rg < 512 && diff_gb < 512, "Pixel {} should be gray", i);
        }
    }

    #[test]
    fn test_pa16_to_rgba16() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 32768u16;

        let pa16_data = create_gray_pa16_data(width, height, Limited, expected_alpha);

        let result = pa16_to_rgba16(&pa16_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some(), "pa16_to_rgba16 should return Some");

        let rgba_colors = result.unwrap();
        assert_eq!(rgba_colors.len(), width * height);

        // Check alpha values
        for (i, color) in rgba_colors.iter().enumerate() {
            assert_eq!(
                color.a, expected_alpha,
                "Pixel {} should have alpha={}, got {}",
                i, expected_alpha, color.a
            );
        }
    }

    #[test]
    fn test_pa16_to_rgba16_full_alpha() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 65535u16;

        let pa16_data = create_gray_pa16_data(width, height, Limited, expected_alpha);

        let result = pa16_to_rgba16(&pa16_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some());

        let rgba_colors = result.unwrap();
        for color in &rgba_colors {
            assert_eq!(color.a, expected_alpha);
        }
    }

    #[test]
    fn test_pa16_to_rgba16_zero_alpha() {
        let width = 8;
        let height = 8;
        let stride = width * 2;
        let expected_alpha = 0u16;

        let pa16_data = create_gray_pa16_data(width, height, Limited, expected_alpha);

        let result = pa16_to_rgba16(&pa16_data, width, height, stride, Limited, Bt601);
        assert!(result.is_some());

        let rgba_colors = result.unwrap();
        for color in &rgba_colors {
            assert_eq!(color.a, expected_alpha);
        }
    }

    #[test]
    fn test_p216_insufficient_data() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        // Create data that's too small
        let small_data = vec![0u8; 100];

        let result = p216_to_rgb16(&small_data, width, height, stride, Limited, Bt601);
        assert!(result.is_none(), "Should fail with insufficient data");

        let result = p216_to_rgba16(&small_data, width, height, stride, Limited, Bt601);
        assert!(result.is_none(), "Should fail with insufficient data");
    }

    #[test]
    fn test_pa16_insufficient_data() {
        let width = 8;
        let height = 8;
        let stride = width * 2;

        // Create P216 data (without alpha plane)
        let p216_data = create_gray_p216_data(width, height, Limited);

        // PA16 requires additional alpha plane, so P216 data should fail
        let result = pa16_to_rgba16(&p216_data, width, height, stride, Limited, Bt601);
        assert!(result.is_none(), "Should fail when alpha plane is missing");
    }

    #[test]
    fn test_p216_various_dimensions() {
        // Test with various even dimensions
        for (w, h) in [(2, 2), (4, 4), (8, 6), (16, 8), (32, 24)] {
            let stride = w * 2;
            let data = create_gray_p216_data(w, h, Limited);

            let rgb_result = p216_to_rgb16(&data, w, h, stride, Limited, Bt601);
            assert!(
                rgb_result.is_some(),
                "RGB conversion should work for {}x{}",
                w,
                h
            );
            assert_eq!(rgb_result.unwrap().len(), w * h);

            let rgba_result = p216_to_rgba16(&data, w, h, stride, Limited, Bt601);
            assert!(
                rgba_result.is_some(),
                "RGBA conversion should work for {}x{}",
                w,
                h
            );
            assert_eq!(rgba_result.unwrap().len(), w * h);
        }
    }

    #[test]
    fn test_deinterleave_uv_plane() {
        // Create test UV data: U0, V0, U1, V1, U2, V2, U3, V3
        let uv_data: Vec<u16> = vec![100, 200, 101, 201, 102, 202, 103, 203];

        let (u_plane, v_plane) = deinterleave_uv_plane(&uv_data, 8, 1);

        assert_eq!(u_plane, vec![100, 101, 102, 103]);
        assert_eq!(v_plane, vec![200, 201, 202, 203]);
    }
}
