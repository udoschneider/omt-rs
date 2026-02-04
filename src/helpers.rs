use crate::Address;
use std::env;

/// Parse CLI args for `--sender`, `--stream`, or a positional address.
///
/// Returns `(sender, stream, address)`.
pub fn parse_cli() -> (Option<String>, Option<String>, Option<Address>) {
    parse_cli_from(env::args().skip(1))
}

/// Parse CLI args from an iterator for testing or custom argument sources.
pub fn parse_cli_from<I>(mut args: I) -> (Option<String>, Option<String>, Option<Address>)
where
    I: Iterator<Item = String>,
{
    let mut sender: Option<String> = None;
    let mut stream: Option<String> = None;
    let mut address: Option<Address> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sender" => sender = args.next(),
            "--stream" => stream = args.next(),
            _ => {
                if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                } else if address.is_none() {
                    address = Some(Address::from(arg));
                }
            }
        }
    }

    (sender, stream, address)
}
