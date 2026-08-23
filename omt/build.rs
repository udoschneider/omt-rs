use std::env;

fn main() {
    // `omt-sys` resolves the libomt location and exports it through the
    // `links = "omt"` metadata (`DEP_OMT_LIBDIR`). On macOS the dylib's install
    // name is `@rpath/libomt.dylib`, and the resolved directory (a cargo cache
    // dir, or `OMT_LIB_DIR`) is not a default loader path, so add an rpath
    // entry so this crate's tests and examples can find it at runtime.
    //
    // No action is needed on Linux (no prebuilt binary; system loader paths
    // apply) or Windows (where the DLL is located via the exe directory/PATH
    // and must be co-located at deployment — see the README).
    let is_macos = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos");
    if is_macos && let Ok(dir) = env::var("DEP_OMT_LIBDIR") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
    }
}
