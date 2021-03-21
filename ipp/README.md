# ipp.rs

IPP protocol implementation for Rust.
This crate implements IPP protocol as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010), [RFC 8011](https://tools.ietf.org/html/rfc8011).

[Documentation](https://ancwrd1.github.io/ipp.rs/doc/ipp/)

Usage example:

```rust
use ipp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
    let operation = IppOperationBuilder::get_printer_attributes(uri.clone()).build();
    let client = IppClient::new(uri);
    let resp = client.send(operation).await?;
    if resp.header().get_status_code().is_success() {
        for (_, v) in resp.attributes().groups_of(DelimiterTag::PrinterAttributes).next().unwrap().attributes() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
