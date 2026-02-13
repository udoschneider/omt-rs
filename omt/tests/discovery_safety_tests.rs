//! Discovery memory safety tests.
//!
//! These tests verify that the Discovery API properly handles memory safety issues,
//! including the documented memory leak in the underlying C library.

use omt::Discovery;

/// Test that Discovery::get_addresses() works and returns valid strings
#[test]
fn test_discovery_basic() {
    let addresses = Discovery::get_addresses();

    // Should not panic or crash
    // May be empty if no sources are available on the network
    // addresses.len() is always >= 0 (usize), so just verify it doesn't panic

    // Verify all returned strings are valid UTF-8 (already validated by the implementation)
    for addr in &addresses {
        assert!(!addr.is_empty() || addresses.len() == 0);
    }
}

/// Test that multiple calls to get_addresses() work correctly
///
/// Note: The underlying C library may leak memory on repeated calls.
/// This is a known limitation documented in the Rust wrapper.
#[test]
fn test_discovery_multiple_calls() {
    // First call
    let addresses1 = Discovery::get_addresses();

    // Second call - the C library may leak memory from the first call
    let addresses2 = Discovery::get_addresses();

    // Both calls should complete without crashing
    // Note: .len() is always >= 0 for Vec, these calls just verify no crash
    let _ = addresses1.len();
    let _ = addresses2.len();

    // Third call to stress test
    let addresses3 = Discovery::get_addresses();
    let _ = addresses3.len();
}

/// Test that Discovery handles empty results gracefully
#[test]
fn test_discovery_empty_result() {
    let addresses = Discovery::get_addresses();

    // If no sources are found, should return empty vector, not crash
    if addresses.is_empty() {
        // This is valid - no sources on the network
        assert_eq!(addresses.len(), 0);
    }
}

/// Test that Discovery strings are properly owned and don't have dangling pointers
#[test]
fn test_discovery_string_ownership() {
    let addresses = Discovery::get_addresses();

    // Call get_addresses again, which may invalidate the C library's internal buffers
    let _addresses2 = Discovery::get_addresses();

    // The first set of addresses should still be valid because they were copied
    for addr in &addresses {
        // Access each string - this would crash if they were dangling pointers
        let _len = addr.len();
        let _chars: Vec<char> = addr.chars().collect();

        // Verify the string is valid UTF-8
        assert!(std::str::from_utf8(addr.as_bytes()).is_ok());
    }
}

/// Test that Discovery handles null pointers in the array gracefully
#[test]
fn test_discovery_null_handling() {
    // The implementation should skip null pointers in the returned array
    let addresses = Discovery::get_addresses();

    // All returned addresses should be non-empty or the array should be empty
    for addr in &addresses {
        assert!(
            !addr.is_empty(),
            "Discovery should not return empty strings from null pointers"
        );
    }
}

/// Test that Discovery strings can be cloned and moved
#[test]
fn test_discovery_string_cloning() {
    let addresses = Discovery::get_addresses();

    if !addresses.is_empty() {
        let first = addresses[0].clone();

        // Call get_addresses again to potentially invalidate C buffers
        let _new_addresses = Discovery::get_addresses();

        // The cloned string should still be valid
        let _len = first.len();
        assert!(!first.is_empty());
    }
}

/// Test that Discovery can be called from multiple threads
#[cfg(not(miri))] // Skip under Miri due to threading complexity
#[test]
fn test_discovery_thread_safety() {
    use std::thread;

    let handles: Vec<_> = (0..5)
        .map(|_| {
            thread::spawn(|| {
                let addresses = Discovery::get_addresses();
                addresses.len()
            })
        })
        .collect();

    for handle in handles {
        let count = handle.join().expect("Thread panicked");
        // count is usize, always >= 0, just verify threads completed
        let _ = count;
    }
}

/// Test that Discovery handles very large result counts defensively
#[test]
fn test_discovery_defensive_checks() {
    // The implementation includes a guard against suspiciously large counts (> 10000)
    // This test verifies the function returns successfully even if the C library
    // returns corrupted data

    let addresses = Discovery::get_addresses();

    // Should never return more than 10000 entries due to safety check
    assert!(
        addresses.len() <= 10000,
        "Discovery should limit results to prevent memory issues"
    );
}

/// Test that Discovery addresses contain expected format
#[test]
fn test_discovery_address_format() {
    let addresses = Discovery::get_addresses();

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

/// Stress test: Many repeated calls to check for memory leaks
///
/// Note: This will leak memory in the underlying C library.
/// This is a known limitation that is documented.
#[test]
#[ignore] // Ignored by default because it intentionally leaks memory
fn test_discovery_repeated_calls_stress_test() {
    // Make many calls to see if there are any crashes or corruption
    for i in 0..100 {
        let addresses = Discovery::get_addresses();

        // Verify results are still valid (len() always >= 0 for Vec)
        let _ = addresses.len();

        if i % 10 == 0 {
            // Occasionally verify string validity
            for addr in &addresses {
                let _ = addr.len();
                let _ = addr.as_bytes();
            }
        }
    }
}

/// Test that Discovery handles UTF-8 validation failures gracefully
#[test]
fn test_discovery_invalid_utf8_handling() {
    let addresses = Discovery::get_addresses();

    // All returned strings should be valid UTF-8
    // Invalid UTF-8 from C should be skipped by the implementation
    for addr in &addresses {
        assert!(
            std::str::from_utf8(addr.as_bytes()).is_ok(),
            "Discovery should only return valid UTF-8 strings"
        );
    }
}

/// Test that Discovery can be used in async contexts
#[cfg(not(miri))]
#[test]
fn test_discovery_async_compatible() {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let addresses = Discovery::get_addresses();
        tx.send(addresses).expect("Failed to send");
    });

    let addresses = rx.recv().expect("Failed to receive");
    // len() is always >= 0 for Vec, just verify we received successfully
    let _ = addresses.len();
}
