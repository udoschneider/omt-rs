//! BGRA video frame conversion functions.

use rgb::bytemuck;
use rgb::*;

pub fn bgra_to_rgb8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    _stride: usize,
) -> Option<Vec<RGB8>> {
    let mut rgb_data = vec![RGB8::new(0, 0, 0); width * height];

    let rgba_stride = (width * 4) as u32;
    let rgb_stride = (width * 3) as u32;
    yuv::bgra_to_rgb(
        raw_data,
        rgba_stride,
        bytemuck::cast_slice_mut(&mut rgb_data),
        rgb_stride,
        width as u32,
        height as u32,
    )
    .ok()?;

    Some(rgb_data)
}

pub fn bgra_to_rgba8(
    raw_data: &[u8],
    width: usize,
    height: usize,
    _stride: usize,
) -> Option<Vec<RGBA8>> {
    let mut rgba_data = vec![RGBA8::new(0, 0, 0, 255); width * height];

    let bgra_stride = (width * 4) as u32;
    let rgba_stride = (width * 4) as u32;
    yuv::bgra_to_rgba(
        raw_data,
        bgra_stride,
        bytemuck::cast_slice_mut(&mut rgba_data),
        rgba_stride,
        width as u32,
        height as u32,
    )
    .ok()?;

    Some(rgba_data)
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::rgb_utils;
    use super::*;

    #[test]
    fn test_bgra_to_rgb8() {
        // Get RGBA colors from test utilities
        let expected_rgba_colors = rgb_utils::rgba8_colors();

        // Convert RGBA to BGRA byte order
        let bgra_colors: Vec<Bgra<u8>> = expected_rgba_colors
            .iter()
            .map(|c| Bgra::<u8> {
                r: c.r,
                g: c.g,
                b: c.b,
                a: c.a,
            })
            .collect();
        let bgra_bytes = rgb::bytemuck::cast_slice(&bgra_colors);

        // Create a 8x8 image (64 pixels total from rgba8_colors)
        let width = 8;
        let height = 8;
        let stride = width * 4;

        // Convert BGRA to RGB8
        let actual_rgb_result = bgra_to_rgb8(&bgra_bytes, width, height, stride);
        assert!(
            actual_rgb_result.is_some(),
            "bgra_to_rgb8 should return Some"
        );

        let actual_rgb_colors = actual_rgb_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(actual_rgb_colors.len(), expected_rgba_colors.len());

        // Compare RGB values (ignoring alpha)
        for (i, (a, e)) in actual_rgb_colors
            .iter()
            .zip(expected_rgba_colors.iter())
            .enumerate()
        {
            assert_eq!(
                a.r, e.r,
                "Red component mismatch at index {}: expected {}, actual {}",
                i, e.r, a.r
            );
            assert_eq!(
                a.g, e.g,
                "Green component mismatch at index {}: expected {}, actual {}",
                i, e.g, a.g
            );
            assert_eq!(
                a.b, e.b,
                "Blue component mismatch at index {}: expected {}, actual {}",
                i, e.b, a.b
            );
        }
    }

    #[test]
    fn test_bgra_to_rgba8() {
        // Get RGBA colors from test utilities
        let expected_rgba_colors = rgb_utils::rgba8_colors();

        // Convert RGBA to BGRA byte order
        let bgra_colors: Vec<Bgra<u8>> = expected_rgba_colors
            .iter()
            .map(|c| Bgra::<u8> {
                r: c.r,
                g: c.g,
                b: c.b,
                a: c.a,
            })
            .collect();
        let bgra_bytes = rgb::bytemuck::cast_slice(&bgra_colors);

        // Create a 8x8 image (64 pixels total from rgba8_colors)
        let width = 8;
        let height = 8;
        let stride = width * 4;

        // Convert BGRA to RGBA8
        let actual_rgba_result = bgra_to_rgba8(&bgra_bytes, width, height, stride);
        assert!(
            actual_rgba_result.is_some(),
            "bgra_to_rgba8 should return Some"
        );

        let actual_rgba_colors = actual_rgba_result.unwrap();

        // Verify we have the right number of pixels
        assert_eq!(actual_rgba_colors.len(), expected_rgba_colors.len());

        // Compare all RGBA components (including alpha)
        for (i, (a, e)) in actual_rgba_colors
            .iter()
            .zip(expected_rgba_colors.iter())
            .enumerate()
        {
            assert_eq!(
                a.r, e.r,
                "Red component mismatch at index {}: expected {}, actual {}",
                i, e.r, a.r
            );
            assert_eq!(
                a.g, e.g,
                "Green component mismatch at index {}: expected {}, actual {}",
                i, e.g, a.g
            );
            assert_eq!(
                a.b, e.b,
                "Blue component mismatch at index {}: expected {}, actual {}",
                i, e.b, a.b
            );
            assert_eq!(
                a.a, e.a,
                "Alpha component mismatch at index {}: expected {}, actual {}",
                i, e.a, a.a
            );
        }
    }
}
