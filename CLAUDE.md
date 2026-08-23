# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Unofficial Rust bindings for [Open Media Transport (OMT)](https://github.com/openmediatransport/libomt), a protocol for low-latency transmission of video, audio, and metadata over IP. A two-crate Cargo workspace wrapping the `libomt` C library.

## Prerequisites

The `libomt` C shared library **must** be installed before building — the `omt-sys` build script links against it and generates bindings from its header.

- Toolchain is pinned via `mise.toml` (Rust 1.93.0). Both crates use edition 2024.
- On macOS/Windows the `omt-sys` build script downloads the **pinned** prebuilt `libomt` release (SHA-256-verified) from `libomtnet/releases` into `~/.cargo/omt/<version>/` and links against it. `OMT_LIB_DIR` overrides this (offline/system install); `OMT_CACHE_DIR` relocates the cache. On Linux there is no prebuilt binary, so the library is searched in `/usr/local/lib`, `/usr/lib`, `/opt/homebrew/lib` (or `OMT_LIB_DIR`).
- The pinned version is recorded in `omt-sys/build.rs` (`OMT_VERSION`, `OMT_ZIP_URL`, `OMT_ZIP_SHA256`). Bump all three **and** `omt-sys/libomt.h` together.
- `omt-sys/libomt.h` is the vendored C header. It is the authoritative reference for codec names, frame semantics, and lifetimes — **consult it whenever you hit an unknown OMT concept.**

## Commands

```bash
cargo build                        # build all crates (requires libomt installed)
cargo build --examples             # must always succeed — examples are part of the build contract
cargo test                         # all tests (unit + integration + doctests)
cargo test -p omt                  # single crate
cargo test <name> -- --nocapture   # single test with output
cargo run --example view_stream    # run an example (see omt/examples/)
cargo fmt --all
cargo clippy --all -- -D warnings  # CI fails on warnings
cargo doc --all --no-deps --open
```

Doc examples are compiled and run by `cargo test`, so keep `///` code blocks valid (or mark them `no_run`/`ignore`).

## Architecture

Three layers, bottom to top:

1. **`libomt`** — native C library (external, must be installed).
2. **`omt-sys`** — raw FFI. `build.rs` runs `bindgen` over `libomt.h` at build time into `$OUT_DIR/bindings.rs`; `src/lib.rs` re-exports them. Everything here is `unsafe`. Regenerated automatically when the header changes — don't hand-edit bindings.
3. **`omt`** — the safe, idiomatic API most work happens in. All public types re-exported from `omt/src/lib.rs`.

### Key patterns in the `omt` crate

- **`MediaFrame<'a>` (`frame/mod.rs`)** is the central type — one struct for video/audio/metadata, distinguished at runtime via `frame_type()`. Type-specific methods live in separate impl blocks (`frame/{video,audio,metadata}.rs`). The lifetime `'a` is the core safety mechanism: received frames borrow C-owned memory valid only until the next receive call, and `'a` enforces this at compile time. An `owned: Option<OwnedBuffers>` field distinguishes borrowed frames (`None`) from deep-copied ones (`Some`) — `Clone` performs a full deep copy (potentially ~64MB for 4K) into heap boxes the frame owns; those boxes free themselves on drop via ordinary RAII, so there is no hand-written `Drop` impl.

- **Hybrid receiver API (`receiver.rs`)** — `receive(&mut self)` is the safe, recommended path (borrow checker prevents holding stale frames); `receive_unchecked(&self)` is an `unsafe` performance escape hatch where the caller must uphold the "no previously-held frame" invariant manually. New feature work should prefer and document the safe path.

- **Frame construction** goes through builders in `frame_builder.rs` (`VideoFrameBuilder`, `AudioFrameBuilder`, `MetadataFrameBuilder`), producing an `OwnedMediaFrame` that owns its data.

- **Video conversion (`video_conversion/`)** — one `from_<format>.rs` per source codec, exposed as `MediaFrame::to_rgb8/rgba8/rgb16/rgba16`. Backed by the SIMD-optimized `yuv` crate; **only conversions `yuv` supports natively are implemented — the rest return `None` by design** (see the module doc before adding one). Return types are `RGB8`/`RGBA8`/etc. (from the `rgb` crate), not raw `u8`, to ease iteration. Color matrix is auto-selected from the frame color space, falling back — when the color space is undefined — to the libomt default of height ≥ 720 → BT.709, else BT.601. Range is always limited (OMT has no full-range signaling).

- **`types/`** wraps C enums/flags into Rust types, each with `from_ffi`/`to_ffi` conversions. Flag types use the `bitflags` crate. This `to_ffi`/`from_ffi` boundary is the single conversion point between safe and raw layers.

- **Discovery (`discovery.rs`)** runs in a C background thread, so the *first* `get_addresses()` call typically returns an empty/incomplete list — callers must poll/retry (examples do). Strings are copied into owned `String`s to avoid the C API's "valid until next call" dangling-pointer hazard.

## Conventions (from AGENTS.md)

- **No panics in library code** — no `unwrap()`/`expect()` outside tests. Use `Result<T, E>`; `omt` uses `thiserror`, binaries/examples use `anyhow`.
- **Every `unsafe` block needs a `// SAFETY:` comment** explaining why it holds. `unsafe` is confined to FFI boundaries and the frame lifetime machinery.
- Public items need `///` docs; modules need `//!` headers. `omt/src/lib.rs` sets `#![warn(missing_docs)]`.
- Conventional Commits (`feat:`, `fix:`, `docs:`). `Cargo.lock` is committed (workspace).
- Unit tests in `#[cfg(test)] mod tests` in-file; integration tests in `omt/tests/` (see `memory_safety_tests.rs`, `discovery_safety_tests.rs` — safety invariants are tested explicitly).
- **Do not create summary/changelog files** documenting your changes unless explicitly asked.
