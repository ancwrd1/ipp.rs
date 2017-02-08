# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://dremon.github.io/ipp.rs/doc/ipp)

Usage example:

```rust
extern crate ipp;
use ipp::consts::tag::PRINTER_ATTRIBUTES_TAG;
use ipp::{GetPrinterAttributes, IppClient};
pub fn main() {
    let client = IppClient::new("http://localhost:631/printers/test-printer");
    let mut operation = GetPrinterAttributes::new();

    let attrs = client.send(&mut operation).unwrap();

    for v in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap().values() {
        println!("{}: {}", v.name(), v.value());
    }
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE](LICENSE-APACHE))
