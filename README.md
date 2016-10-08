# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://dremon.github.io/ipp.rs/doc/ipp)

Usage example:

```rust
pub fn main() {
    let mut operation = GetPrinterAttributes::new("http://localhost:631/printers/test-printer");
    let attrs = operation.execute().unwrap();
    for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
        println!("{}: {}", v.name(), v.value());
    }
}
```

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
