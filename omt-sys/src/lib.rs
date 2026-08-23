#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Directory containing the `libomt` shared library the build linked against,
/// when it was resolved to a specific location (the download cache or
/// `OMT_LIB_DIR`).
///
/// This is the directory to copy `OMT_LIB_FILE` from when bundling the native
/// library next to (or inside) your application at deployment time. It is
/// `None` when the linker's default search paths were used (e.g. a Linux system
/// install), in which case the library is already on the system loader path.
pub const OMT_LIB_DIR: Option<&str> = option_env!("OMT_RESOLVED_LIB_DIR");

/// File name of the `libomt` shared library the build linked against
/// (`libomt.dylib`, `libomt.so`, or `libomt.dll`). See [`OMT_LIB_DIR`].
pub const OMT_LIB_FILE: Option<&str> = option_env!("OMT_RESOLVED_LIB_FILE");
