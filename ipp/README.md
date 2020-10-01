# ipp.rs

Asynchronous IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

This crate implements IPP protocol as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010), [RFC 8011](https://tools.ietf.org/html/rfc8011).

Not all features are implemented yet. Transport is based on `isahc` client or `reqwest` client depending on the selected feature.
Note: for `reqwest` client a runtime is needed such as `tokio`. 

Usage example:

```rust
use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
    } else {
        let client = IppClientBuilder::new(args[1].parse()?).build();
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
    }
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
