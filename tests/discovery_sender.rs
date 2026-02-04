use libomt::{Discovery, Quality, Sender, Source};
use std::thread::sleep;
use std::time::Duration;

#[test]
fn discovery_finds_sender() {
    let source = Source::from(format!("omt-rs-test-sender-{}", std::process::id()));
    let _sender = Sender::create(&source, Quality::Default).expect("create sender");

    // Give the sender a moment to advertise via DNS-SD / discovery server.
    let mut found = false;

    for _ in 0..5 {
        let addresses = Discovery::get_addresses_with_backoff(
            5,
            Duration::from_millis(100),
            Duration::from_millis(200),
            1.0,
        );

        if addresses
            .iter()
            .any(|addr| addr.as_str().contains(source.as_str()))
        {
            found = true;
            break;
        }

        sleep(Duration::from_millis(200));
    }

    assert!(
        found,
        "Expected discovery to find sender name '{}' in advertised addresses",
        source
    );
}
