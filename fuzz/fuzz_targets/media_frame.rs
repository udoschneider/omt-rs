//! Fuzzes `MediaFrame`'s handling of attacker-controlled frame headers.
//!
//! A received frame pairs a buffer libomt really owns with `Codec`, `Width`,
//! `Height`, `Stride`, `Channels` and `SamplesPerChannel` fields that came off
//! the network. `video_conversion::required_input_len` is the single gate that
//! keeps those fields from driving an out-of-bounds slice or an unbounded
//! allocation in the RGB converters — this target attacks that gate.
//!
//! The assertions live in `omt::fuzzing::fuzz_media_frame` so the same
//! invariants can be swept deterministically from an ordinary stable-toolchain
//! unit test. See that module for what is checked and why the data pointer
//! stays truthful while the header does not.
//!
//! Run with:
//!
//! ```sh
//! cargo +nightly fuzz run media_frame
//! ```

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    omt::fuzzing::fuzz_media_frame(data);
});
