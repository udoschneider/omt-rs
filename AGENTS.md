# USAGE rules

## Usage & Development Guidelines

This document outlines the standards, conventions, and rules for contributing to **[Project Name]**. These rules ensure the codebase remains idiomatic, safe, performant, and maintainable.

### 1. Toolchain & Environment

*   **Rust Version:** We support the latest stable Rust release.
    *   *Optional:* We support a MSRV (Minimum Supported Rust Version) of `1.xx.0`.
*   **Formatting:** All code must be formatted using `rustfmt`.
    *   Run: `cargo fmt --all` before committing.
    *   CI will fail if code is not formatted.
*   **Linting:** We strictly follow `clippy` suggestions.
    *   Run: `cargo clippy -- -D warnings` before committing.
    *   **Note:** If you believe a lint is a false positive, allow it explicitly at the call site with a comment explaining why: `#[allow(clippy::lint_name)] // Reason...`.

### 2. Code Style & Idioms

We aim for "Idiomatic Rust." Please adhere to the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

*   **Naming Conventions:**
    *   `Snake_case`: Variables, functions, modules, macros.
    *   `UpperCamelCase`: Types, Traits, Enums, Structs.
    *   `SCREAMING_SNAKE_CASE`: Constants and Statics.
*   **Imports:** Group imports logically.
    *   `std` / `core` first.
    *   External crates (3rd party) second.
    *   Internal crate modules (`crate::...`) last.
*   **Early Returns:** Prefer early returns (guard clauses) over deep nesting.

### 3. Safety & `unsafe`

Rust guarantees memory safety; `unsafe` subverts it.

*   **Avoid Unsafe:** Do not use `unsafe` code unless strictly necessary (e.g., FFI, primitive performance optimizations backed by benchmarks).
*   **Documentation Requirement:** Every `unsafe` block **must** include a `// SAFETY:` comment explaining why the operation is safe and what invariants the caller must uphold.

```rust
// GOOD
// SAFETY: We checked that `index` is less than `len` in the line above.
unsafe {
    *ptr.add(index)
}
```

### 4. Error Handling

*   **No Panics:** Avoid `unwrap()` and `expect()` in library code or production logic.
    *   *Exceptions:* Tests, prototypes, or situations where a panic is the only logical outcome (e.g., locking a mutex that should never be poisoned).
*   **Return Types:** Use `Result<T, E>` for recoverable errors.
*   **Error Crates:** Use `thiserror` for libraries (implementing `std::error::Error`) and `anyhow` for applications/binaries.

### 5. Documentation

We treat documentation as a first-class citizen.

*   **Public API:** All public structs, enums, traits, and functions must have doc comments (`///`).
*   **Examples:** Include usage examples in doc comments. These are automatically tested via `cargo test`.
*   **Module Docs:** Include a `//!` comment at the top of main modules explaining their purpose.
*   **Summary/Progress Files:** You must not create files that document the changes made unless explicitly requested by the project maintainer!

```rust
/// Calculates the sum of two numbers.
///
/// # Examples
///
/// ```
/// let result = my_crate::add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 { ... }
```

### 6. Testing

*   **Unit Tests:** Place unit tests in a `mod tests` module within the same file as the source code.
*   **Integration Tests:** Place end-to-end tests in the `tests/` directory.
*   **Test Coverage:** Aim to test both success paths and failure paths (error handling).
*   Ensure that `cargo build` is allways successfull
*   Ensure that `cargo build --examples` is allways successfull.

### 7. Dependencies

*   **Bloat:** Be mindful of dependency weight. Avoid pulling in heavy crates for simple utility functions.
*   **Features:** When depending on large crates (e.g., `tokio`, `serde`), only enable the feature flags strictly required.
*   **Vetting:** Verify that new dependencies are well-maintained and use a compatible license (MIT/Apache 2.0).

### 8. Git & Version Control

*   **Commit Messages:** We follow [Conventional Commits](https://www.conventionalcommits.org/).
    *   `feat: add new parser`
    *   `fix: resolve memory leak`
    *   `docs: update readme`
*   **Lockfile:**
    *   **Libraries:** Do not commit `Cargo.lock` (unless this is a workspace).
    *   **Binaries/Apps:** Do commit `Cargo.lock` to ensure reproducible builds.

***

### Summary Checklist for PRs

- [ ] `cargo fmt` passes.
- [ ] `cargo clippy` passes with no warnings.
- [ ] `cargo test` passes.
- [ ] New code is covered by tests.
- [ ] Public API is documented.
- [ ] No `unwrap()` in production code.

### 9. Additional documentation

If you encounter unknown concepts (e.g. codec names) check `libomt.h` for details.
