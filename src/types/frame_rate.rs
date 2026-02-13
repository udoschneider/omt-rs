//! Frame rate representation for video frames.
//!
//! Frame rates are represented as rational numbers (numerator/denominator) to allow
//! exact representation of common video frame rates like 29.97 fps (30000/1001) or
//! 59.94 fps (60000/1001).

use std::fmt;

/// Error type for frame rate validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameRateError {
    /// Frame rate numerator must be positive.
    InvalidNumerator(i32),
    /// Frame rate denominator must be positive.
    InvalidDenominator(i32),
}

impl fmt::Display for FrameRateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameRateError::InvalidNumerator(n) => {
                write!(f, "frame rate numerator must be positive, got {}", n)
            }
            FrameRateError::InvalidDenominator(d) => {
                write!(f, "frame rate denominator must be positive, got {}", d)
            }
        }
    }
}

impl std::error::Error for FrameRateError {}

/// Represents a frame rate as a rational number (numerator/denominator).
///
/// Frame rates are represented as fractions to allow exact representation of
/// common video frame rates like 29.97 fps (30000/1001) or 59.94 fps (60000/1001).
///
/// # Examples
///
/// ```
/// use omt::FrameRate;
///
/// // 30 fps
/// let fps_30 = FrameRate::new(30, 1).unwrap();
/// assert_eq!(fps_30.value(), 30.0);
///
/// // 29.97 fps (NTSC)
/// let fps_ntsc = FrameRate::new(30000, 1001).unwrap();
/// assert!((fps_ntsc.value() - 29.97).abs() < 0.01);
///
/// // Invalid frame rates return an error
/// assert!(FrameRate::new(0, 1).is_err());
/// assert!(FrameRate::new(30, 0).is_err());
/// assert!(FrameRate::new(-30, 1).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
    frame_rate_n: i32,
    frame_rate_d: i32,
}

impl FrameRate {
    /// Creates a new frame rate from a numerator and denominator.
    ///
    /// # Parameters
    ///
    /// - `numerator`: The frame rate numerator (must be positive)
    /// - `denominator`: The frame rate denominator (must be positive)
    ///
    /// # Errors
    ///
    /// Returns `FrameRateError::InvalidNumerator` if numerator is <= 0.
    /// Returns `FrameRateError::InvalidDenominator` if denominator is <= 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::FrameRate;
    ///
    /// let fps = FrameRate::new(60, 1).unwrap();  // 60 fps
    /// assert_eq!(fps.numerator(), 60);
    /// assert_eq!(fps.denominator(), 1);
    ///
    /// // Invalid values return errors
    /// assert!(FrameRate::new(0, 1).is_err());
    /// assert!(FrameRate::new(30, 0).is_err());
    /// assert!(FrameRate::new(-30, 1).is_err());
    /// ```
    pub fn new(numerator: i32, denominator: i32) -> Result<Self, FrameRateError> {
        if numerator <= 0 {
            return Err(FrameRateError::InvalidNumerator(numerator));
        }
        if denominator <= 0 {
            return Err(FrameRateError::InvalidDenominator(denominator));
        }
        Ok(Self {
            frame_rate_n: numerator,
            frame_rate_d: denominator,
        })
    }

    /// Creates a frame rate of 23.976 fps (24000/1001).
    ///
    /// This is commonly used for film content in NTSC regions.
    pub fn fps_23_976() -> Self {
        Self {
            frame_rate_n: 24000,
            frame_rate_d: 1001,
        }
    }

    /// Creates a frame rate of 24 fps.
    ///
    /// This is the standard film frame rate.
    pub fn fps_24() -> Self {
        Self {
            frame_rate_n: 24,
            frame_rate_d: 1,
        }
    }

    /// Creates a frame rate of 25 fps.
    ///
    /// This is the standard PAL video frame rate.
    pub fn fps_25() -> Self {
        Self {
            frame_rate_n: 25,
            frame_rate_d: 1,
        }
    }

    /// Creates a frame rate of 29.97 fps (30000/1001).
    ///
    /// This is the standard NTSC video frame rate.
    pub fn fps_29_97() -> Self {
        Self {
            frame_rate_n: 30000,
            frame_rate_d: 1001,
        }
    }

    /// Creates a frame rate of 30 fps.
    pub fn fps_30() -> Self {
        Self {
            frame_rate_n: 30,
            frame_rate_d: 1,
        }
    }

    /// Creates a frame rate of 50 fps.
    ///
    /// This is commonly used for high frame rate PAL content.
    pub fn fps_50() -> Self {
        Self {
            frame_rate_n: 50,
            frame_rate_d: 1,
        }
    }

    /// Creates a frame rate of 59.94 fps (60000/1001).
    ///
    /// This is commonly used for high frame rate NTSC content.
    pub fn fps_59_94() -> Self {
        Self {
            frame_rate_n: 60000,
            frame_rate_d: 1001,
        }
    }

    /// Creates a frame rate of 60 fps.
    pub fn fps_60() -> Self {
        Self {
            frame_rate_n: 60,
            frame_rate_d: 1,
        }
    }

    /// Creates a frame rate of 119.88 fps (120000/1001).
    ///
    /// This is commonly used for very high frame rate NTSC content.
    pub fn fps_119_88() -> Self {
        Self {
            frame_rate_n: 120000,
            frame_rate_d: 1001,
        }
    }

    /// Creates a frame rate of 120 fps.
    pub fn fps_120() -> Self {
        Self {
            frame_rate_n: 120,
            frame_rate_d: 1,
        }
    }

    /// Returns the frame rate numerator.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::FrameRate;
    ///
    /// let fps = FrameRate::new(30000, 1001).unwrap();
    /// assert_eq!(fps.numerator(), 30000);
    /// ```
    pub fn numerator(&self) -> i32 {
        self.frame_rate_n
    }

    /// Returns the frame rate denominator.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::FrameRate;
    ///
    /// let fps = FrameRate::new(30000, 1001).unwrap();
    /// assert_eq!(fps.denominator(), 1001);
    /// ```
    pub fn denominator(&self) -> i32 {
        self.frame_rate_d
    }

    /// Returns the frame rate as a floating point value (frames per second).
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::FrameRate;
    ///
    /// let fps_30 = FrameRate::new(30, 1).unwrap();
    /// assert_eq!(fps_30.value(), 30.0);
    ///
    /// let fps_ntsc = FrameRate::new(30000, 1001).unwrap();
    /// assert!((fps_ntsc.value() - 29.97).abs() < 0.01);
    /// ```
    pub fn value(&self) -> f32 {
        self.frame_rate_n as f32 / self.frame_rate_d as f32
    }
}

impl fmt::Display for FrameRate {
    /// Formats the frame rate in a human-readable format.
    ///
    /// Displays the frame rate with up to 3 decimal places, removing trailing zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::FrameRate;
    ///
    /// let fps_24 = FrameRate::new(24, 1).unwrap();
    /// assert_eq!(format!("{}", fps_24), "24 fps");
    ///
    /// let fps_ntsc = FrameRate::new(30000, 1001).unwrap();
    /// assert_eq!(format!("{}", fps_ntsc), "29.97 fps");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.value();

        // Round to 3 decimal places
        let rounded = (value * 1000.0).round() / 1000.0;

        // Check if it's effectively an integer (no significant fractional part)
        if (rounded - rounded.round()).abs() < 0.0001 {
            write!(f, "{} fps", rounded.round() as i32)
        } else {
            // Format with up to 3 decimal places, removing trailing zeros
            let formatted = format!("{:.3}", rounded);
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            write!(f, "{} fps", trimmed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fps = FrameRate::new(60, 1).unwrap();
        assert_eq!(fps.numerator(), 60);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 60.0);
    }

    #[test]
    fn test_new_validation() {
        // Valid frame rates
        assert!(FrameRate::new(1, 1).is_ok());
        assert!(FrameRate::new(30, 1).is_ok());
        assert!(FrameRate::new(60000, 1001).is_ok());

        // Invalid numerator (zero)
        assert!(matches!(
            FrameRate::new(0, 1),
            Err(FrameRateError::InvalidNumerator(0))
        ));

        // Invalid numerator (negative)
        assert!(matches!(
            FrameRate::new(-30, 1),
            Err(FrameRateError::InvalidNumerator(-30))
        ));

        // Invalid denominator (zero)
        assert!(matches!(
            FrameRate::new(30, 0),
            Err(FrameRateError::InvalidDenominator(0))
        ));

        // Invalid denominator (negative)
        assert!(matches!(
            FrameRate::new(30, -1),
            Err(FrameRateError::InvalidDenominator(-1))
        ));

        // Both invalid
        assert!(matches!(
            FrameRate::new(0, 0),
            Err(FrameRateError::InvalidNumerator(0))
        ));
    }

    #[test]
    fn test_new_unchecked() {
        // Test direct struct construction (internal use only)
        let fps = FrameRate {
            frame_rate_n: 30,
            frame_rate_d: 1,
        };
        assert_eq!(fps.numerator(), 30);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 30.0);
    }

    #[test]
    fn test_frame_rate_error_display() {
        let err = FrameRateError::InvalidNumerator(-30);
        assert_eq!(
            format!("{}", err),
            "frame rate numerator must be positive, got -30"
        );

        let err = FrameRateError::InvalidDenominator(0);
        assert_eq!(
            format!("{}", err),
            "frame rate denominator must be positive, got 0"
        );
    }

    #[test]
    fn test_fps_23_976() {
        let fps = FrameRate::fps_23_976();
        assert_eq!(fps.numerator(), 24000);
        assert_eq!(fps.denominator(), 1001);
        assert!((fps.value() - 23.976).abs() < 0.001);
    }

    #[test]
    fn test_fps_24() {
        let fps = FrameRate::fps_24();
        assert_eq!(fps.numerator(), 24);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 24.0);
    }

    #[test]
    fn test_fps_25() {
        let fps = FrameRate::fps_25();
        assert_eq!(fps.numerator(), 25);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 25.0);
    }

    #[test]
    fn test_fps_29_97() {
        let fps = FrameRate::fps_29_97();
        assert_eq!(fps.numerator(), 30000);
        assert_eq!(fps.denominator(), 1001);
        assert!((fps.value() - 29.97).abs() < 0.01);
    }

    #[test]
    fn test_fps_30() {
        let fps = FrameRate::fps_30();
        assert_eq!(fps.numerator(), 30);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 30.0);
    }

    #[test]
    fn test_fps_50() {
        let fps = FrameRate::fps_50();
        assert_eq!(fps.numerator(), 50);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 50.0);
    }

    #[test]
    fn test_fps_59_94() {
        let fps = FrameRate::fps_59_94();
        assert_eq!(fps.numerator(), 60000);
        assert_eq!(fps.denominator(), 1001);
        assert!((fps.value() - 59.94).abs() < 0.01);
    }

    #[test]
    fn test_fps_60() {
        let fps = FrameRate::fps_60();
        assert_eq!(fps.numerator(), 60);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 60.0);
    }

    #[test]
    fn test_fps_119_88() {
        let fps = FrameRate::fps_119_88();
        assert_eq!(fps.numerator(), 120000);
        assert_eq!(fps.denominator(), 1001);
        assert!((fps.value() - 119.88).abs() < 0.01);
    }

    #[test]
    fn test_fps_120() {
        let fps = FrameRate::fps_120();
        assert_eq!(fps.numerator(), 120);
        assert_eq!(fps.denominator(), 1);
        assert_eq!(fps.value(), 120.0);
    }

    #[test]
    fn test_display_integer_fps() {
        assert_eq!(format!("{}", FrameRate::fps_24()), "24 fps");
        assert_eq!(format!("{}", FrameRate::fps_25()), "25 fps");
        assert_eq!(format!("{}", FrameRate::fps_30()), "30 fps");
        assert_eq!(format!("{}", FrameRate::fps_50()), "50 fps");
        assert_eq!(format!("{}", FrameRate::fps_60()), "60 fps");
        assert_eq!(format!("{}", FrameRate::fps_120()), "120 fps");
    }

    #[test]
    fn test_display_fractional_fps() {
        assert_eq!(format!("{}", FrameRate::fps_23_976()), "23.976 fps");
        assert_eq!(format!("{}", FrameRate::fps_29_97()), "29.97 fps");
        assert_eq!(format!("{}", FrameRate::fps_59_94()), "59.94 fps");
        assert_eq!(format!("{}", FrameRate::fps_119_88()), "119.88 fps");
    }

    #[test]
    fn test_display_custom_fps() {
        let fps = FrameRate::new(48, 1).unwrap();
        assert_eq!(format!("{}", fps), "48 fps");

        let fps = FrameRate::new(15, 1).unwrap();
        assert_eq!(format!("{}", fps), "15 fps");

        let fps = FrameRate::new(1000, 1).unwrap();
        assert_eq!(format!("{}", fps), "1000 fps");
    }

    #[test]
    fn test_display_trailing_zeros_removed() {
        // 25.5 fps should display as "25.5 fps", not "25.500 fps"
        let fps = FrameRate::new(51, 2).unwrap();
        assert_eq!(format!("{}", fps), "25.5 fps");

        // Test another case
        let fps = FrameRate::new(100, 3).unwrap();
        let display = format!("{}", fps);
        assert!(display.starts_with("33.3"));
        assert!(display.ends_with(" fps"));
    }

    #[test]
    fn test_equality() {
        let fps1 = FrameRate::fps_30();
        let fps2 = FrameRate::fps_30();
        let fps3 = FrameRate::new(30, 1).unwrap();
        let fps4 = FrameRate::fps_60();

        assert_eq!(fps1, fps2);
        assert_eq!(fps1, fps3);
        assert_ne!(fps1, fps4);
    }

    #[test]
    fn test_clone() {
        let fps1 = FrameRate::fps_30();
        let fps2 = fps1.clone();
        assert_eq!(fps1, fps2);
    }

    #[test]
    fn test_copy() {
        let fps1 = FrameRate::fps_30();
        let fps2 = fps1; // Copy, not move
        assert_eq!(fps1, fps2);
        // Both should still be usable
        assert_eq!(fps1.value(), 30.0);
        assert_eq!(fps2.value(), 30.0);
    }

    #[test]
    fn test_debug() {
        let fps = FrameRate::fps_30();
        let debug_str = format!("{:?}", fps);
        assert!(debug_str.contains("FrameRate"));
        assert!(debug_str.contains("30"));
    }

    #[test]
    fn test_fractional_denominator() {
        let fps = FrameRate::new(24000, 1001).unwrap();
        assert_eq!(fps.numerator(), 24000);
        assert_eq!(fps.denominator(), 1001);
        let value = fps.value();
        assert!((value - 23.976).abs() < 0.001);
    }

    #[test]
    fn test_value_calculation() {
        // Test various frame rate calculations
        let fps = FrameRate::new(100, 4).unwrap();
        assert_eq!(fps.value(), 25.0);

        let fps = FrameRate::new(120, 2).unwrap();
        assert_eq!(fps.value(), 60.0);

        let fps = FrameRate::new(1, 1).unwrap();
        assert_eq!(fps.value(), 1.0);
    }

    #[test]
    fn test_minimum_valid_frame_rate() {
        // Test minimum valid frame rate (1/1)
        let fps = FrameRate::new(1, 1).unwrap();
        assert_eq!(fps.value(), 1.0);
        assert_eq!(format!("{}", fps), "1 fps");

        // Test very slow frame rate
        let fps = FrameRate::new(1, 100).unwrap();
        assert_eq!(fps.value(), 0.01);
    }

    #[test]
    fn test_large_frame_rates() {
        let fps = FrameRate::new(240, 1).unwrap();
        assert_eq!(fps.value(), 240.0);
        assert_eq!(format!("{}", fps), "240 fps");

        let fps = FrameRate::new(1000, 1).unwrap();
        assert_eq!(fps.value(), 1000.0);
        assert_eq!(format!("{}", fps), "1000 fps");
    }

    #[test]
    fn test_display_precision() {
        // Test that display rounds to 3 decimal places
        let fps = FrameRate::new(10000, 3001).unwrap();
        let display = format!("{}", fps);
        assert!(display.contains("3.332") || display.contains("3.333"));
        assert!(display.ends_with(" fps"));
    }

    #[test]
    fn test_all_convenience_constructors_unique() {
        // Ensure each convenience constructor produces unique values
        let rates = vec![
            FrameRate::fps_23_976(),
            FrameRate::fps_24(),
            FrameRate::fps_25(),
            FrameRate::fps_29_97(),
            FrameRate::fps_30(),
            FrameRate::fps_50(),
            FrameRate::fps_59_94(),
            FrameRate::fps_60(),
            FrameRate::fps_119_88(),
            FrameRate::fps_120(),
        ];

        for (i, rate1) in rates.iter().enumerate() {
            for (j, rate2) in rates.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        rate1, rate2,
                        "fps constructors at indices {} and {} should be different",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_eq_reflexive() {
        // Test reflexive property: x == x
        let fps = FrameRate::fps_30();
        assert_eq!(fps, fps);

        let fps = FrameRate::fps_29_97();
        assert_eq!(fps, fps);

        let fps = FrameRate::new(48, 1).unwrap();
        assert_eq!(fps, fps);
    }

    #[test]
    fn test_eq_symmetric() {
        // Test symmetric property: if x == y, then y == x
        let fps1 = FrameRate::fps_30();
        let fps2 = FrameRate::fps_30();
        assert_eq!(fps1, fps2);
        assert_eq!(fps2, fps1);

        let fps3 = FrameRate::fps_29_97();
        let fps4 = FrameRate::fps_29_97();
        assert_eq!(fps3, fps4);
        assert_eq!(fps4, fps3);
    }

    #[test]
    fn test_eq_transitive() {
        // Test transitive property: if x == y and y == z, then x == z
        let fps1 = FrameRate::fps_60();
        let fps2 = FrameRate::fps_60();
        let fps3 = FrameRate::new(60, 1).unwrap();

        assert_eq!(fps1, fps2);
        assert_eq!(fps2, fps3);
        assert_eq!(fps1, fps3);
    }

    #[test]
    fn test_eq_with_convenience_constructors() {
        // Test Eq using all convenience constructors
        assert_eq!(FrameRate::fps_23_976(), FrameRate::fps_23_976());
        assert_eq!(FrameRate::fps_24(), FrameRate::fps_24());
        assert_eq!(FrameRate::fps_25(), FrameRate::fps_25());
        assert_eq!(FrameRate::fps_29_97(), FrameRate::fps_29_97());
        assert_eq!(FrameRate::fps_30(), FrameRate::fps_30());
        assert_eq!(FrameRate::fps_50(), FrameRate::fps_50());
        assert_eq!(FrameRate::fps_59_94(), FrameRate::fps_59_94());
        assert_eq!(FrameRate::fps_60(), FrameRate::fps_60());
        assert_eq!(FrameRate::fps_119_88(), FrameRate::fps_119_88());
        assert_eq!(FrameRate::fps_120(), FrameRate::fps_120());
    }

    #[test]
    fn test_eq_convenience_constructors_vs_new() {
        // Test that convenience constructors equal new() with same values
        assert_eq!(
            FrameRate::fps_23_976(),
            FrameRate::new(24000, 1001).unwrap()
        );
        assert_eq!(FrameRate::fps_24(), FrameRate::new(24, 1).unwrap());
        assert_eq!(FrameRate::fps_25(), FrameRate::new(25, 1).unwrap());
        assert_eq!(FrameRate::fps_29_97(), FrameRate::new(30000, 1001).unwrap());
        assert_eq!(FrameRate::fps_30(), FrameRate::new(30, 1).unwrap());
        assert_eq!(FrameRate::fps_50(), FrameRate::new(50, 1).unwrap());
        assert_eq!(FrameRate::fps_59_94(), FrameRate::new(60000, 1001).unwrap());
        assert_eq!(FrameRate::fps_60(), FrameRate::new(60, 1).unwrap());
        assert_eq!(
            FrameRate::fps_119_88(),
            FrameRate::new(120000, 1001).unwrap()
        );
        assert_eq!(FrameRate::fps_120(), FrameRate::new(120, 1).unwrap());
    }

    #[test]
    fn test_eq_inequality_between_different_rates() {
        // Test inequality between different frame rates
        assert_ne!(FrameRate::fps_24(), FrameRate::fps_25());
        assert_ne!(FrameRate::fps_30(), FrameRate::fps_29_97());
        assert_ne!(FrameRate::fps_60(), FrameRate::fps_59_94());
        assert_ne!(FrameRate::fps_120(), FrameRate::fps_119_88());
    }

    #[test]
    fn test_eq_same_value_different_representation() {
        // Frame rates with same value but different representation should NOT be equal
        // because Eq compares exact numerator and denominator, not computed value
        let fps1 = FrameRate::new(30, 1).unwrap();
        let fps2 = FrameRate::new(60, 2).unwrap(); // Same value (30.0) but different representation
        assert_ne!(fps1, fps2);
        assert_eq!(fps1.value(), fps2.value()); // But values are equal

        let fps3 = FrameRate::new(25, 1).unwrap();
        let fps4 = FrameRate::new(50, 2).unwrap(); // Same value (25.0)
        assert_ne!(fps3, fps4);
        assert_eq!(fps3.value(), fps4.value());
    }

    #[test]
    fn test_eq_with_hash_collections() {
        use std::collections::HashSet;

        // Test that Eq works correctly with hash-based collections
        let mut set = HashSet::new();
        set.insert(FrameRate::fps_30());
        set.insert(FrameRate::fps_60());
        set.insert(FrameRate::fps_30()); // Duplicate, should not add

        assert_eq!(set.len(), 2);
        assert!(set.contains(&FrameRate::fps_30()));
        assert!(set.contains(&FrameRate::fps_60()));
        assert!(!set.contains(&FrameRate::fps_25()));
    }

    #[test]
    fn test_eq_edge_cases() {
        // Test edge cases for Eq - only valid values are allowed now
        let fps1 = FrameRate::new(1, 1).unwrap();
        let fps2 = FrameRate::new(1, 1).unwrap();
        assert_eq!(fps1, fps2);

        let large1 = FrameRate::new(1000000, 1).unwrap();
        let large2 = FrameRate::new(1000000, 1).unwrap();
        assert_eq!(large1, large2);

        let frac1 = FrameRate::new(1000, 999).unwrap();
        let frac2 = FrameRate::new(1000, 999).unwrap();
        assert_eq!(frac1, frac2);
    }

    #[test]
    fn test_eq_consistency_with_clone() {
        // Test that cloned values are equal
        let fps1 = FrameRate::fps_29_97();
        let fps2 = fps1.clone();
        assert_eq!(fps1, fps2);

        let fps3 = FrameRate::new(100, 3).unwrap();
        let fps4 = fps3.clone();
        assert_eq!(fps3, fps4);
    }
}
