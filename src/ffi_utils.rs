//! Shared FFI helper utilities.

use std::ffi::CStr;
use std::os::raw::c_char;

/// Converts a fixed-size C char array to a Rust `String`.
pub(crate) fn c_char_array_to_string(arr: &[c_char]) -> String {
    // SAFETY: We cast a pointer to a c_char slice to a u8 slice with the same length.
    // c_char is guaranteed to be a single-byte type (i8 or u8), so this cast is safe.
    // The resulting slice has the same lifetime and length as the input slice.
    let bytes = unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len()) };
    match CStr::from_bytes_until_nul(bytes) {
        Ok(cstr) => cstr.to_string_lossy().into_owned(),
        Err(_) => String::new(),
    }
}

/// Writes a Rust string into a fixed-size C char array with NUL termination.
pub(crate) fn write_c_char_array(dst: &mut [c_char], value: &str) {
    if dst.is_empty() {
        return;
    }
    dst.fill(0);
    let bytes = value.as_bytes();
    let max = dst.len() - 1;
    let copy_len = bytes.len().min(max);
    for (dst_byte, src_byte) in dst.iter_mut().take(copy_len).zip(bytes.iter()) {
        *dst_byte = *src_byte as c_char;
    }
    dst[copy_len] = 0;
}
