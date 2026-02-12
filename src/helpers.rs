use crate::{Address, Discovery, Error, Timeout};
use std::env;

/// Strips trailing null terminators from a byte slice and converts to a string.
///
/// This is used when reading metadata from the C API, which includes
/// null terminators in the length, but Rust strings should not include them.
///
/// Handles edge cases:
/// - Multiple trailing null bytes (strips all of them)
/// - Empty slices (returns empty string)
/// - Only null bytes (returns empty string)
/// - Embedded null bytes (preserved in the output)
/// - Invalid UTF-8 (returns error)
///
/// # Panics
///
/// Panics if the input contains invalid UTF-8.
///
/// # Examples
///
/// ```
/// use omt::helpers::without_null_terminator;
///
/// let with_null = b"<test>data</test>\0";
/// let stripped = without_null_terminator(with_null);
/// assert_eq!(stripped, "<test>data</test>");
///
/// let without_null = b"<test>data</test>";
/// let unchanged = without_null_terminator(without_null);
/// assert_eq!(unchanged, "<test>data</test>");
///
/// // Multiple trailing null bytes are all stripped
/// let multiple_nulls = b"data\0\0\0";
/// let stripped = without_null_terminator(multiple_nulls);
/// assert_eq!(stripped, "data");
///
/// // Empty slice
/// let empty = b"";
/// let result = without_null_terminator(empty);
/// assert_eq!(result, "");
///
/// // Only null bytes
/// let only_nulls = b"\0\0\0";
/// let result = without_null_terminator(only_nulls);
/// assert_eq!(result, "");
///
/// // Embedded nulls are preserved
/// let embedded = b"data\0middle\0";
/// let result = without_null_terminator(embedded);
/// assert_eq!(result, "data\0middle");
/// ```
pub fn without_null_terminator(slice: &[u8]) -> &str {
    // Find the last non-null byte position
    let end = slice
        .iter()
        .rposition(|&b| b != 0)
        .map(|pos| pos + 1)
        .unwrap_or(0);

    // Convert the non-trailing-null portion to a string
    // This will panic if the input contains invalid UTF-8, which is acceptable
    // since the C API should only return valid UTF-8 strings
    std::str::from_utf8(&slice[..end]).expect("C API returned invalid UTF-8 string")
}

/// Creates a null-terminated byte vector from a string.
///
/// Validates that the string does not contain any null bytes (which would
/// interfere with C string handling), then creates a vector with the string
/// bytes followed by a null terminator.
///
/// # Errors
///
/// Returns `Error::InvalidCString` if the input string contains any null bytes.
///
/// # Examples
///
/// ```
/// use omt::helpers::null_terminated_bytes;
///
/// let result = null_terminated_bytes("<test>data</test>").unwrap();
/// assert_eq!(result, b"<test>data</test>\0");
/// assert_eq!(result.len(), 18); // 17 chars + 1 null
///
/// // String with null byte fails
/// let result = null_terminated_bytes("test\0data");
/// assert!(result.is_err());
/// ```
pub fn null_terminated_bytes<S: AsRef<str>>(s: S) -> Result<Vec<u8>, Error> {
    let s = s.as_ref();
    if s.contains('\0') {
        return Err(Error::InvalidCString);
    }
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    Ok(bytes)
}

/// Discover OMT sender addresses with configurable backoff.
///
/// Reads environment variables to configure discovery behavior:
/// - `OMTRS_DISCOVERY_ATTEMPTS`: number of discovery attempts (default: 5)
/// - `OMTRS_DISCOVERY_INITIAL_DELAY_MS`: initial delay between attempts in milliseconds (default: 200)
/// - `OMTRS_DISCOVERY_MAX_DELAY_MS`: maximum delay between attempts in milliseconds (default: same as initial_delay_ms)
/// - `OMTRS_DISCOVERY_BACKOFF`: backoff factor (default: 1.0)
pub fn discover_addresses() -> Vec<Address> {
    let attempts = env::var("OMTRS_DISCOVERY_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    let initial_delay_ms = env::var("OMTRS_DISCOVERY_INITIAL_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(200);
    let max_delay_ms = env::var("OMTRS_DISCOVERY_MAX_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(initial_delay_ms);
    let backoff = env::var("OMTRS_DISCOVERY_BACKOFF")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);

    Discovery::get_addresses_with_backoff(
        attempts,
        Timeout::from_millis(initial_delay_ms).as_duration(),
        Timeout::from_millis(max_delay_ms).as_duration(),
        backoff,
    )
}

/// Find the first discovered sender address.
pub fn discover_first_sender() -> Option<Address> {
    discover_addresses().into_iter().next()
}

/// Find a sender address matching optional sender and stream name filters.
///
/// - `sender`: if provided, the address must start with this string (case-insensitive)
/// - `stream`: if provided, the address must contain `(stream)` (case-insensitive)
pub fn discover_matching_sender(sender: Option<&str>, stream: Option<&str>) -> Option<Address> {
    let addresses = discover_addresses();

    let sender_lc = sender.map(|s| s.to_lowercase());
    let stream_lc = stream.map(|s| s.to_lowercase());

    for address in addresses {
        let address_lc = address.as_str().to_lowercase();

        if let Some(sender) = sender_lc.as_deref() {
            if !address_lc.starts_with(sender) {
                continue;
            }
        }

        if let Some(stream) = stream_lc.as_deref() {
            let needle = format!("({})", stream);
            if !address_lc.contains(&needle) {
                continue;
            }
        }

        return Some(address);
    }

    None
}
