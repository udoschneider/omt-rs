//! Discovery memory safety tests.
//!
//! `omt_discovery_getaddresses` returns process-global state that the C library
//! recycles on the next call, so the hazards here are ownership (the returned
//! `String`s must be copies, not views into that state) and concurrency (two
//! overlapping calls must not free the array under each other). These tests
//! exercise both against the real native library.
//!
//! Note what is *not* testable from here: the implementation skips null and
//! non-UTF-8 entries, but neither can be provoked without substituting a fake C
//! library, and `String` is valid UTF-8 by construction — so asserting that
//! returned strings decode proves nothing about the skip logic. Those paths are
//! covered by inspection, not by a test that cannot fail.

use omt::{Discovery, MAX_PLAUSIBLE_SOURCES};

/// A discovery sweep must succeed on a real host whether or not anything is on
/// the network: "no sources" is `Ok(vec![])`, and only a corrupt result from the
/// C library is an error.
#[test]
fn test_discovery_basic() {
    let addresses = Discovery::get_addresses().expect("discovery must not fail on a real host");

    for addr in &addresses {
        assert!(!addr.is_empty(), "discovery returned an empty address");
    }
}

/// Repeated calls must keep succeeding.
///
/// Note: the underlying C library may leak memory on each call. That is a
/// documented limitation of libomt, not of this wrapper.
#[test]
fn test_discovery_multiple_calls() {
    for i in 0..3 {
        Discovery::get_addresses().unwrap_or_else(|e| panic!("discovery call {i} failed: {e}"));
    }
}

/// The returned `String`s must be owned copies, not views into the C library's
/// buffer.
///
/// This is the core ownership guarantee: the C array is "valid until the next
/// call to getaddresses", so a later call is exactly what would invalidate a
/// borrowed view. Every byte of the *first* result is re-read afterwards.
#[test]
fn test_discovery_string_ownership() {
    let addresses = Discovery::get_addresses().expect("first discovery sweep");
    let snapshot: Vec<String> = addresses.clone();

    // Recycle the C library's internal array out from under the first result.
    let _ = Discovery::get_addresses().expect("second discovery sweep");
    let _ = Discovery::get_addresses().expect("third discovery sweep");

    // The first result must be byte-for-byte intact. Reading every byte is what
    // would trip a sanitizer (or crash) had the strings been borrowed views.
    assert_eq!(addresses, snapshot, "addresses changed after later sweeps");
    for addr in &addresses {
        assert!(!addr.is_empty());
        assert_eq!(addr.as_bytes().iter().filter(|&&b| b == 0).count(), 0);
    }
}

/// Test that Discovery can be called from multiple threads
///
/// Regression test for the process-global buffer race: `libomt.h` documents the
/// `char**` from `omt_discovery_getaddresses` as "valid until the next call to
/// getaddresses", so without internal serialization one thread's call frees the
/// array while another is still copying strings out of it — a use-after-free
/// reachable from safe code. `Discovery::get_addresses` holds an internal mutex
/// across both the call and the copy.
///
/// The threads start together on a barrier and hammer the call so the windows
/// genuinely overlap; every returned string is then fully read, which is what
/// would trip ASan/valgrind (or crash outright) if the buffer had been recycled
/// mid-copy.
#[cfg(not(miri))] // Skip under Miri due to threading complexity
#[test]
fn test_discovery_thread_safety() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    const THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                for _ in 0..ITERATIONS {
                    let addresses = Discovery::get_addresses().expect("concurrent discovery sweep");
                    for address in addresses {
                        // Touch every byte: a dangling copy would surface here.
                        assert!(!address.is_empty());
                        assert!(std::str::from_utf8(address.as_bytes()).is_ok());
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

/// A successful sweep never yields more entries than the count ceiling the
/// implementation is willing to trust — beyond it the array is treated as
/// corrupt and reported as an error rather than walked.
#[test]
fn test_discovery_respects_count_ceiling() {
    let addresses = Discovery::get_addresses().expect("discovery must not fail on a real host");

    assert!(
        addresses.len() <= MAX_PLAUSIBLE_SOURCES as usize,
        "discovery returned {} entries, above the {MAX_PLAUSIBLE_SOURCES} ceiling",
        addresses.len()
    );
}

/// Test that Discovery addresses contain expected format
#[test]
fn test_discovery_address_format() {
    let addresses = Discovery::get_addresses().expect("discovery must not fail on a real host");

    for addr in &addresses {
        // Addresses should be either:
        // 1. "HOSTNAME (NAME)" format
        // 2. "omt://hostname:port" format
        // Or contain valid UTF-8 characters

        assert!(
            addr.is_ascii()
                || addr
                    .chars()
                    .all(|c| c.is_alphanumeric() || ":/().-_ ".contains(c)),
            "Address contains unexpected characters: {}",
            addr
        );
    }
}

/// The result must be movable across threads, so a discovery sweep can run off
/// the caller's thread. Asserted statically as well as exercised: the static
/// bound is what actually fails the build if `Send` is ever lost.
#[cfg(not(miri))]
#[test]
fn test_discovery_result_is_send() {
    use std::sync::mpsc;
    use std::thread;

    fn assert_send<T: Send>() {}
    assert_send::<omt::Result<Vec<String>>>();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(Discovery::get_addresses()).expect("Failed to send");
    });

    let addresses = rx
        .recv()
        .expect("Failed to receive")
        .expect("discovery must not fail on a real host");
    for addr in &addresses {
        assert!(!addr.is_empty());
    }
}

/// Stress test: many repeated calls, checking for crashes or corruption.
///
/// Note: this will leak memory in the underlying C library — a known, documented
/// limitation — which is why it is ignored by default.
#[test]
#[ignore] // Ignored by default because it intentionally leaks memory
fn test_discovery_repeated_calls_stress_test() {
    for i in 0..100 {
        let addresses =
            Discovery::get_addresses().unwrap_or_else(|e| panic!("sweep {i} failed: {e}"));

        // Read every byte of every address: corruption from a recycled buffer
        // would surface here rather than as a silent wrong answer.
        for addr in &addresses {
            assert!(!addr.is_empty());
            assert!(std::str::from_utf8(addr.as_bytes()).is_ok());
        }
    }
}
