use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The libomtnet release these bindings are pinned to. The vendored
/// `libomt.h` and the prebuilt binaries must all come from this exact release,
/// so bump `OMT_VERSION` and `OMT_ZIP_SHA256` together — and update
/// `omt-sys/libomt.h` to match — when upgrading.
const OMT_VERSION: &str = "1.0.0.16";
const OMT_ZIP_URL: &str = "https://github.com/openmediatransport/libomtnet/releases/download/v1.0.0.16/OpenMediaTransport.Binaries.Release.v1.0.0.16.zip";
const OMT_ZIP_SHA256: &str = "c70e67f7e2a7ed5b4c389d99af62796a8c9c7be23c8debfae3fd8020c1dc66b9";

/// Conventional system install locations. Searched as a fallback when the
/// platform has no prebuilt binary (Linux) or the download fails; always passed
/// to the linker so a manually installed `libomt` is found.
const DEFAULT_SEARCH_DIRS: &[&str] = &["/usr/local/lib", "/usr/lib", "/opt/homebrew/lib"];

/// Shared-library file names probed when detecting an existing install.
const LIB_CANDIDATES: &[&str] = &["libomt.so", "libomt.dylib", "omt.dll", "libomt.dll"];

/// A platform that ships prebuilt binaries in the release archive.
#[derive(Clone, Copy)]
enum Prebuilt {
    MacOs,
    WinX64,
    WinArm64,
}

impl Prebuilt {
    fn for_target(target: &str) -> Option<Prebuilt> {
        if target.contains("apple-darwin") {
            Some(Prebuilt::MacOs)
        } else if target.contains("windows-msvc") {
            if target.starts_with("aarch64") {
                Some(Prebuilt::WinArm64)
            } else {
                Some(Prebuilt::WinX64)
            }
        } else {
            None
        }
    }

    /// Sub-directory inside the release archive holding this platform's libs.
    fn archive_dir(self) -> &'static str {
        match self {
            Prebuilt::MacOs => "Libraries/MacOS",
            Prebuilt::WinX64 => "Libraries/Winx64",
            Prebuilt::WinArm64 => "Libraries/Winarm64",
        }
    }

    /// Cache sub-directory name for this platform (under the version root).
    fn cache_dir(self) -> &'static str {
        match self {
            Prebuilt::MacOs => "macos",
            Prebuilt::WinX64 => "win-x64",
            Prebuilt::WinArm64 => "win-arm64",
        }
    }

    /// Files to extract from the archive (paths relative to the archive root).
    ///
    /// `libomt` is the link-time dependency. `libvmx` is dlopen'd lazily by
    /// `libomt` at runtime only for compressed VMX1 video, but it must sit next
    /// to the application, so it is co-located here too. `libomt.h` is extracted
    /// solely to verify it matches the vendored header.
    fn files(self) -> Vec<String> {
        let dir = self.archive_dir();
        match self {
            Prebuilt::MacOs => vec![
                format!("{dir}/libomt.dylib"),
                format!("{dir}/libvmx.dylib"),
                format!("{dir}/libomt.h"),
            ],
            Prebuilt::WinX64 | Prebuilt::WinArm64 => vec![
                format!("{dir}/libomt.dll"),
                format!("{dir}/libomt.lib"),
                format!("{dir}/libvmx.dll"),
                format!("{dir}/libomt.h"),
            ],
        }
    }
}

/// A successfully resolved `libomt` location.
struct ResolvedLib {
    /// Directory containing the shared library.
    dir: PathBuf,
    /// File name of the shared library (`libomt.dylib`, `libomt.so`, ...).
    file: String,
}

impl ResolvedLib {
    fn from_path(path: PathBuf) -> Option<ResolvedLib> {
        let file = path.file_name()?.to_string_lossy().into_owned();
        let dir = path.parent()?.to_path_buf();
        Some(ResolvedLib { dir, file })
    }
}

fn main() {
    // Re-run the build script when the library location can change out from
    // under us. `CargoCallbacks` (below) already re-runs on header changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OMT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=OMT_CACHE_DIR");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");

    let target = env::var("TARGET").expect("cargo sets TARGET for build scripts");

    // rustc's `-l` semantics differ per toolchain: Unix linkers re-apply the
    // `lib` prefix (`-l omt` -> `libomt.{so,dylib}`), while MSVC resolves the
    // name verbatim plus `.lib` (`-l omt` -> `omt.lib`). The release ships
    // `libomt.lib`, so MSVC needs `-l libomt`.
    let link_name = if target.contains("windows-msvc") {
        "libomt"
    } else {
        "omt"
    };

    // Resolve the libomt shared library and emit linker directives for it.
    match resolve_lib(&target) {
        Some(resolved) => {
            println!("cargo:rustc-link-search=native={}", resolved.dir.display());
            println!("cargo:rustc-link-lib={link_name}");

            // Expose the resolved location so dependent crates (via `links`) and
            // the compiled `omt-sys` crate (for downstream app bundling) can
            // find the library.
            println!("cargo:LIBDIR={}", resolved.dir.display());
            println!("cargo:LIBFILE={}", resolved.file);
            println!(
                "cargo:rustc-env=OMT_RESOLVED_LIB_DIR={}",
                resolved.dir.display()
            );
            println!("cargo:rustc-env=OMT_RESOLVED_LIB_FILE={}", resolved.file);
        }
        None => {
            // No explicit directory: fall back to the linker's own search
            // (`LIBRARY_PATH`) plus the conventional install locations.
            for dir in DEFAULT_SEARCH_DIRS {
                println!("cargo:rustc-link-search=native={dir}");
            }
            println!("cargo:rustc-link-lib={link_name}");
            warn_missing();
        }
    }

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

/// Resolve the location of the `libomt` shared library.
///
/// Order of preference:
/// 1. `OMT_LIB_DIR` (explicit override — never touches the network).
/// 2. A previously downloaded/extracted cache (offline after first build).
/// 3. Download + extract the pinned prebuilt release (macOS/Windows only).
/// 4. Conventional system install locations (the only path on Linux).
///
/// Returns `None` only when nothing is found and the linker's own defaults
/// (`LIBRARY_PATH`, system paths) remain the sole fallback.
fn resolve_lib(target: &str) -> Option<ResolvedLib> {
    // 1. Explicit override.
    if let Ok(dir) = env::var("OMT_LIB_DIR") {
        let dir = PathBuf::from(dir);
        if let Some(path) = find_lib_in(&dir) {
            return ResolvedLib::from_path(path);
        }
        panic!(
            "OMT_LIB_DIR is set to '{}' but no libomt shared library was found there \
             (expected one of: {}). Point OMT_LIB_DIR at a directory containing the \
             libomt shared library, or unset it to use the automatic download.",
            dir.display(),
            LIB_CANDIDATES.join(", ")
        );
    }

    let prebuilt = Prebuilt::for_target(target);

    if let Some(prebuilt) = prebuilt {
        let cache = cache_dir(prebuilt);

        // 2. Offline cache from a previous build.
        if let Some(path) = find_lib_in(&cache) {
            return ResolvedLib::from_path(path);
        }

        // 3. Download + extract the pinned release. Fall back to a system
        //    install (or a bare linker error) on failure so offline users with
        //    a preinstalled libomt still build.
        if let Err(err) = ensure_prebuilt(prebuilt, &cache) {
            println!(
                "cargo:warning=omt-sys: could not fetch prebuilt libomt v{OMT_VERSION}: {err}"
            );
            println!("cargo:warning=omt-sys: falling back to a system-installed libomt, if any.");
        } else if let Some(path) = find_lib_in(&cache) {
            return ResolvedLib::from_path(path);
        }
    }

    // 4. System install locations.
    for dir in DEFAULT_SEARCH_DIRS {
        if let Some(path) = find_lib_in(Path::new(dir)) {
            return ResolvedLib::from_path(path);
        }
    }

    None
}

/// Return the full path of a `libomt` shared library in `dir`, if present.
fn find_lib_in(dir: &Path) -> Option<PathBuf> {
    LIB_CANDIDATES
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.exists())
}

/// Root directory for the extracted prebuilt libraries (per pinned version).
///
/// Defaults to `$CARGO_HOME/omt/<version>` so the 35 MB download is shared
/// across projects and survives `cargo clean`; override with `OMT_CACHE_DIR`.
fn cache_root() -> PathBuf {
    if let Ok(dir) = env::var("OMT_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    let base = env::var("CARGO_HOME").unwrap_or_else(|_| {
        env::var("HOME")
            .map(|home| format!("{home}/.cargo"))
            .unwrap_or_else(|_| ".cargo".to_string())
    });
    PathBuf::from(base).join("omt").join(OMT_VERSION)
}

fn cache_dir(prebuilt: Prebuilt) -> PathBuf {
    cache_root().join(prebuilt.cache_dir())
}

/// Download, verify, and extract the pinned release into `dest`.
fn ensure_prebuilt(prebuilt: Prebuilt, dest: &Path) -> Result<(), String> {
    let root = dest
        .parent()
        .ok_or_else(|| "prebuilt cache dir has no parent".to_string())?;
    let zip_path = root.join("omt.zip");

    if !zip_path.exists() {
        download_zip(&zip_path)?;
    }
    verify_checksum(&zip_path)?;

    // Extract into a staging dir, then atomically move into place so a
    // concurrent build never observes a half-extracted cache.
    let staging = root.join(format!(".{}-staging", prebuilt.cache_dir()));
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    fs::create_dir_all(&staging)
        .map_err(|e| format!("could not create staging dir {}: {e}", staging.display()))?;

    extract_files(&zip_path, &staging, prebuilt)?;

    // The archive's header must match the vendored one, otherwise the bindings
    // and the binary describe different ABIs.
    verify_header(&staging.join("libomt.h"))?;

    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|e| format!("could not clear {}: {e}", dest.display()))?;
    }
    fs::rename(&staging, dest)
        .map_err(|e| format!("could not move {} into place: {e}", dest.display()))?;

    Ok(())
}

/// Download the release archive with `curl` (present on macOS and modern
/// Windows; this path is never taken on Linux).
fn download_zip(dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create cache dir {}: {e}", parent.display()))?;
    }
    println!("cargo:warning=omt-sys: downloading libomt v{OMT_VERSION} from {OMT_ZIP_URL}");

    let tmp = dest.with_extension("part");
    let status = Command::new("curl")
        .args(["--fail", "--location", "--retry", "3", "--output"])
        .arg(&tmp)
        .arg(OMT_ZIP_URL)
        .status()
        .map_err(|e| format!("failed to launch curl (is it installed?): {e}"))?;

    if !status.success() {
        let _ = fs::remove_file(&tmp);
        return Err(format!("curl exited with status {status}"));
    }
    fs::rename(&tmp, dest).map_err(|e| format!("could not finalize download: {e}"))?;
    Ok(())
}

/// Verify the downloaded archive against the pinned SHA-256 digest.
fn verify_checksum(zip_path: &Path) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let data =
        fs::read(zip_path).map_err(|e| format!("could not read {}: {e}", zip_path.display()))?;
    let digest = Sha256::digest(&data);
    let hex = digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if hex != OMT_ZIP_SHA256 {
        return Err(format!(
            "checksum mismatch for {}: expected {OMT_ZIP_SHA256}, got {hex}",
            zip_path.display()
        ));
    }
    Ok(())
}

/// Extract the platform's files from the archive into `dest`, flattened.
fn extract_files(zip_path: &Path, dest: &Path, prebuilt: Prebuilt) -> Result<(), String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("could not open {}: {e}", zip_path.display()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("could not open archive: {e}"))?;

    for entry in prebuilt.files() {
        let name = Path::new(&entry)
            .file_name()
            .ok_or_else(|| format!("archive entry '{entry}' has no file name"))?
            .to_string_lossy()
            .into_owned();
        let mut source = archive
            .by_name(&entry)
            .map_err(|e| format!("archive is missing '{entry}': {e}"))?;
        let out_path = dest.join(name);
        let mut out = fs::File::create(&out_path)
            .map_err(|e| format!("could not create {}: {e}", out_path.display()))?;
        std::io::copy(&mut source, &mut out)
            .map_err(|e| format!("could not extract '{entry}': {e}"))?;
    }
    Ok(())
}

/// Verify the `libomt.h` shipped with the release matches the vendored header
/// (ignoring CRLF line-ending differences).
fn verify_header(extracted: &Path) -> Result<(), String> {
    let vendored = fs::read_to_string("libomt.h")
        .map_err(|e| format!("could not read vendored libomt.h: {e}"))?;
    let theirs = fs::read_to_string(extracted)
        .map_err(|e| format!("could not read {}: {e}", extracted.display()))?;
    if vendored.replace('\r', "") == theirs.replace('\r', "") {
        Ok(())
    } else {
        Err(format!(
            "the libomt.h shipped with release v{OMT_VERSION} differs from the vendored \
             omt-sys/libomt.h. Copy the archive's header over omt-sys/libomt.h (and \
             update the crate) before pinning this release."
        ))
    }
}

/// Surface a clear, actionable diagnostic when the shared library is nowhere
/// to be found, instead of leaving the user with a bare linker error.
fn warn_missing() {
    println!(
        "cargo:warning=libomt shared library not found in {} or via OMT_LIB_DIR. \
         Install libomt (see the crate README) or set OMT_LIB_DIR / LIBRARY_PATH \
         to the directory containing libomt.so / libomt.dylib.",
        DEFAULT_SEARCH_DIRS.join(", ")
    );
}
