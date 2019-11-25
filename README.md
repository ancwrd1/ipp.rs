# ipp.rs

IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

This crate implements IPP protocol as defined in RFC 2911. Not all features are implemented yet.<br/>
Transport is based on asynchronous HTTP client from the `isahc` crate.

Usage example:

```rust,no_run
use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
        std::process::exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = IppOperationBuilder::get_printer_attributes()
        .attributes(&args[2..])
        .build();

    let attrs = futures::executor::block_on(client.send(operation))?;

    for v in attrs.groups_of(DelimiterTag::PrinterAttributes)[0]
        .attributes()
        .values()
    {
        println!("{}: {}", v.name(), v.value());
    }

    Ok(())
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
