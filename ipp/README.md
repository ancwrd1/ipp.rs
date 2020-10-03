# ipp.rs

Asynchronous IPP protocol implementation for Rust

[Documentation](https://docs.rs/ipp)

This crate implements IPP protocol as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010), [RFC 8011](https://tools.ietf.org/html/rfc8011).

Transport support can be selected by feature options: `client-isahc` or `client-reqwest`.
The default client is `isahc`.

Note: for the `reqwest` client a runtime is needed such as `tokio`.

Usage example (no runtime, simple future blocking):

```rust
use ipp::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri [attrs]", args[0]);
    } else {
        let uri: Uri = args[1].parse()?;
        let client = IppClient::new(uri.clone());
        let operation = IppOperationBuilder::get_printer_attributes(uri)
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
    Ok(())
}
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
