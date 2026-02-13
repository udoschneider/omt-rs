//! Tally state management for OMT sources.
//!
//! Tally lights indicate whether a source is in preview or program (on-air).

/// Tally state information.
///
/// Indicates whether a source is in preview or program mode.
/// Values: 0 = off, 1 = on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Tally {
    /// Preview tally state (off-air monitoring).
    pub preview: bool,
    /// Program tally state (on-air/live).
    pub program: bool,
}

impl Tally {
    /// Creates a new tally state.
    ///
    /// # Examples
    ///
    /// ```
    /// use omt::Tally;
    ///
    /// let tally = Tally::new(false, true);
    /// assert!(!tally.preview);
    /// assert!(tally.program);
    /// ```
    pub fn new(preview: bool, program: bool) -> Self {
        Self { preview, program }
    }

    /// Creates a tally with both states off.
    pub fn off() -> Self {
        Self {
            preview: false,
            program: false,
        }
    }

    /// Creates a tally with preview on and program off.
    pub fn preview_only() -> Self {
        Self {
            preview: true,
            program: false,
        }
    }

    /// Creates a tally with program on and preview off.
    pub fn program_only() -> Self {
        Self {
            preview: false,
            program: true,
        }
    }

    /// Returns true if either preview or program is active.
    pub fn is_active(&self) -> bool {
        self.preview || self.program
    }

    /// Returns true if both preview and program are off.
    pub fn is_off(&self) -> bool {
        !self.preview && !self.program
    }

    /// Converts to FFI representation.
    pub(crate) fn to_ffi(&self) -> omt_sys::OMTTally {
        omt_sys::OMTTally {
            preview: if self.preview { 1 } else { 0 },
            program: if self.program { 1 } else { 0 },
        }
    }

    /// Converts from FFI representation.
    pub(crate) fn from_ffi(ffi: &omt_sys::OMTTally) -> Self {
        Self {
            preview: ffi.preview != 0,
            program: ffi.program != 0,
        }
    }
}

impl std::fmt::Display for Tally {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.preview, self.program) {
            (false, false) => write!(f, "Off"),
            (true, false) => write!(f, "Preview"),
            (false, true) => write!(f, "Program"),
            (true, true) => write!(f, "Preview+Program"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tally_new() {
        let tally = Tally::new(true, false);
        assert!(tally.preview);
        assert!(!tally.program);
    }

    #[test]
    fn test_tally_constructors() {
        let off = Tally::off();
        assert!(off.is_off());
        assert!(!off.is_active());

        let preview = Tally::preview_only();
        assert!(preview.preview);
        assert!(!preview.program);
        assert!(preview.is_active());

        let program = Tally::program_only();
        assert!(!program.preview);
        assert!(program.program);
        assert!(program.is_active());
    }

    #[test]
    fn test_tally_ffi_conversion() {
        let tally = Tally::new(true, false);
        let ffi = tally.to_ffi();
        assert_eq!(ffi.preview, 1);
        assert_eq!(ffi.program, 0);

        let converted = Tally::from_ffi(&ffi);
        assert_eq!(tally, converted);
    }

    #[test]
    fn test_tally_display() {
        assert_eq!(Tally::off().to_string(), "Off");
        assert_eq!(Tally::preview_only().to_string(), "Preview");
        assert_eq!(Tally::program_only().to_string(), "Program");
        assert_eq!(Tally::new(true, true).to_string(), "Preview+Program");
    }
}
