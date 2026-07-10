use std::env;
use std::path::{Path, PathBuf};

/// Directories searched for `libomt` in addition to any `OMT_LIB_DIR` override
/// and the linker's own defaults (`LIBRARY_PATH`, etc.).
const DEFAULT_SEARCH_DIRS: &[&str] = &["/usr/local/lib", "/usr/lib", "/opt/homebrew/lib"];

fn main() {
    // Re-run the build script when the library location can change out from
    // under us. `CargoCallbacks` (below) already re-runs on header changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OMT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");

    // Collect the search directories: an optional `OMT_LIB_DIR` override first,
    // then the conventional install locations.
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(dir) = env::var("OMT_LIB_DIR") {
        search_dirs.push(PathBuf::from(dir));
    }
    search_dirs.extend(DEFAULT_SEARCH_DIRS.iter().map(PathBuf::from));

    for dir in &search_dirs {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }

    // Surface a clear, actionable diagnostic when the shared library is nowhere
    // to be found, instead of leaving the user with a bare linker error. This
    // is only a heuristic (the linker also consults `LIBRARY_PATH` and system
    // defaults we don't inspect here), so it warns rather than fails the build.
    if !library_is_discoverable(&search_dirs) {
        println!(
            "cargo:warning=libomt shared library not found in {} or via OMT_LIB_DIR. \
             Install libomt (see the crate README) or set OMT_LIB_DIR / LIBRARY_PATH \
             to the directory containing libomt.so / libomt.dylib.",
            search_dirs
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Link against the native OMT shared library.
    println!("cargo:rustc-link-lib=omt");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .prepend_enum_name(false)
        // Don't copy the C header's Doxygen comments into the bindings: rustdoc
        // parses their `@param[in]`/`[out]` markers as (broken) intra-doc links,
        // which fails `cargo doc -D warnings`. The vendored `libomt.h` remains
        // the authoritative reference for these declarations.
        .generate_comments(false)
        // The input header we would like to generate
        // bindings for.
        .header("libomt.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Bindgen shells out to libclang; the usual failure is a missing
        // libclang, so name it in the panic message.
        .expect("Unable to generate bindings from libomt.h (is libclang installed?)");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings to OUT_DIR");
}

/// Best-effort check for a `libomt` shared library under any search directory.
fn library_is_discoverable(search_dirs: &[PathBuf]) -> bool {
    // Cover the platform-specific shared-library names.
    const CANDIDATES: &[&str] = &["libomt.so", "libomt.dylib", "omt.dll", "libomt.dll"];
    search_dirs.iter().any(|dir| {
        CANDIDATES
            .iter()
            .any(|name| Path::new(dir).join(name).exists())
    })
}
