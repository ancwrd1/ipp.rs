# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

Usage example:

```rust
extern crate ipp;
use ipp::{GetPrinterAttributes, IppClient};
pub fn main() {
    let client = IppClient::new("http://localhost:631/printers/test-printer");
    let operation = GetPrinterAttributes::new();

    let attrs = client.send(operation).unwrap();

    for v in attrs.get_printer_attributes().unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
