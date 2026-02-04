fn main() {
    println!("cargo:rerun-if-env-changed=OMTRS_LIB_DIR");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=DYLD_LIBRARY_PATH");

    if let Ok(dir) = std::env::var("OMTRS_LIB_DIR") {
        if !dir.is_empty() {
            println!("cargo:rustc-link-search=native={}", dir);
        }
    }

    for dir in &[
        "/usr/local/lib",
        "/usr/lib",
        "/opt/homebrew/lib",
        "/opt/local/lib",
        "/usr/local/opt/libomt/lib",
    ] {
        println!("cargo:rustc-link-search=native={}", dir);
    }

    println!("cargo:rustc-link-lib=omt");
}
