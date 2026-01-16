# ipp.rs

[![github actions](https://github.com/ancwrd1/ipp.rs/workflows/CI/badge.svg)](https://github.com/ancwrd1/ipp.rs/actions)
[![crates](https://img.shields.io/crates/v/ipp.svg)](https://crates.io/crates/ipp)
[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![license](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![docs.rs](https://docs.rs/ipp/badge.svg)](https://docs.rs/ipp)

IPP protocol implementation for Rust.
This crate implements IPP protocol as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010), [RFC 8011](https://tools.ietf.org/html/rfc8011).

It supports both synchronous and asynchronous operations (requests and responses) which is controlled by the `async` feature flag.

The following build-time features are supported:

* `async` - enables asynchronous APIs.
* `async-client` - enables an asynchronous IPP client based on `reqwest` crate, implies `async` feature.
* `async-client-rustls` - enables an asynchronous IPP client with TLS, using `rustls` backend. Implies `async-client` feature.
* `async-client-tls` - enables an asynchronous IPP client with TLS, using `native-tls` backend. Implies `async-client` feature.
* `client` - enables a blocking IPP client based on `ureq` crate.
* `client-rustls` - enables a blocking IPP client with TLS, using `rustls` backend. Implies `client` feature.
* `client-tls` - enables a blocking IPP client with TLS, using `native-tls` backend. Implies `client` feature.

By default, the `async-client-rustls` feature is enabled. Some old printers may not support the latest TLS standards;
in that case you can choose to use `async-client-tls` or `client-tls` instead which will use platform-specific `native-tls` instead of `rustls`. 

[Documentation](https://docs.rs/ipp/latest/ipp/)

Usage example for an async client:

```rust
use ipp::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
    let operation = IppOperationBuilder::get_printer_attributes(uri.clone()).build();
    let client = AsyncIppClient::new(uri);
    let resp = client.send(operation).await?;
    if resp.header().status_code().is_success() {
        let printer_attrs = resp
            .attributes()
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap();
        for (_, v) in printer_attrs.attributes() {
            println!("{}: {}", v.name(), v.value());
        }
    }
    Ok(())
}
```

For more usage examples please check the [examples folder](https://github.com/ancwrd1/ipp.rs/tree/master/examples).

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
