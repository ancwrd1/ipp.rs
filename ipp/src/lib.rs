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
//!     let uri = "http://localhost:631/printers/test-printer";
//!     let req = IppRequestResponse::new(
//!         IppVersion::Ipp11,
//!         Operation::GetPrinterAttributes,
//!         Some(uri)
//!     );
//!     let client = IppClientBuilder::new(&uri).build();
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
//!     let client = IppClientBuilder::new("http://localhost:631/printers/test-printer").build();
//!     let attrs = futures::executor::block_on(client.send(operation))?;
//!     for (_, v) in attrs.groups_of(DelimiterTag::PrinterAttributes)[0].attributes() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//!     Ok(())
//! }
//!```

pub mod proto;

#[cfg(feature = "client")]
pub mod client;

pub mod prelude {
    #[cfg(feature = "client")]
    pub use super::client::{IppClient, IppClientBuilder, IppError};
    pub use super::proto::{
        attribute::*, ipp::*, request::IppRequestResponse, FromPrimitive as _, IppAttribute, IppAttributeGroup,
        IppAttributes, IppHeader, IppOperationBuilder, IppParser, IppPayload, IppValue,
    };
}
