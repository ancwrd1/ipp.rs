# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

Usage example:

```rust
extern crate ipp_client;
extern crate ipp_proto;

use ipp_proto::IppOperationBuilder;
use ipp_client::IppClientBuilder;

fn main() {
    let operation = IppOperationBuilder::get_printer_attributes().build();
    let client = IppClientBuilder::new("http://localhost:631/printers/test-printer").build();
    if let Ok(attrs) = client.send(operation) {
        for (_, v) in attrs.printer_attributes().unwrap() {
            println!("{}: {}", v.name(), v.value());
        }
    }
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
