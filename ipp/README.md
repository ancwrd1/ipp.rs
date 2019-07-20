# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

This crate implements IPP protocol as defined in RFC 2911. Not all features are implemented yet.<br/>
Transport is based on asynchronous HTTP client from the `reqwest` crate.

Usage example:

```rust,no_run
use ipp::{
    client::IppClientBuilder,
    proto::{ipp::DelimiterTag, IppOperationBuilder},
};
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new()?;

    let operation = IppOperationBuilder::get_printer_attributes().build();
    let client = IppClientBuilder::new("http://localhost:631/printers/test-printer").build();
    let attrs = runtime.block_on(client.send(operation))?;

    for (_, v) in attrs.groups_of(DelimiterTag::PrinterAttributes)[0].attributes() {
        println!("{}: {}", v.name(), v.value());
    }
    Ok(())
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
