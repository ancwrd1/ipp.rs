//!
//! IPP print protocol implementation for Rust
//!
//! Usage examples:
//!
//!```rust,no_run
//! // using raw API
//! use ipp::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
//!     let req = IppRequestResponse::new(
//!         IppVersion::v1_1(),
//!         Operation::GetPrinterAttributes,
//!         Some(uri.to_string())
//!     );
//!     let client = IppClientBuilder::new(uri).build();
//!     let resp = futures::executor::block_on(client.send_request(req))?;
//!     if resp.header().operation_status <= 2 {
//!         println!("result: {:?}", resp.attributes());
//!     }
//!     Ok(())
//! }
//!```
//!```rust,no_run
//! // using operations API
//! use ipp::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let operation = IppOperationBuilder::get_printer_attributes().build();
//!     let client = IppClientBuilder::new("http://localhost:631/printers/test-printer".parse()?).build();
//!     let attrs = futures::executor::block_on(client.send(operation))?;
//!     for (_, v) in attrs.groups_of(DelimiterTag::PrinterAttributes)[0].attributes() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//!     Ok(())
//! }
//!```

pub mod proto;

#[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
pub mod client;

pub mod prelude {
    #[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
    pub use super::client::{IppClient, IppClientBuilder, IppError};
    pub use super::proto::{
        model::*, FromPrimitive as _, IppAttribute, IppAttributeGroup, IppAttributes, IppHeader, IppOperationBuilder,
        IppParser, IppPayload, IppRequestResponse, IppValue, IppVersion,
    };
    #[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
    pub use http::Uri;
}
