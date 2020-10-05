//!
//! IPP print protocol implementation for Rust. This crate can be used in several ways:
//! * using the low-level request/response API and building the requests manually.
//! * using the higher-level operations API with builders. Currently only a subset of all IPP operations is supported.
//! * using the built-in IPP client based on `reqwest` or `isahc` crates.
//! (selected via `client-isahc` or `client-reqwest`) features.
//! * using any third-party HTTP client and send the serialized request manually.
//!
//! Implementation notes:
//! * all RFC IPP values are supported including arrays and collections, for both de- and serialization.
//! * the **Accept-Encoding** HTTP header seems to cause problems with some older printers,
//! therefore it is disabled by default.
//! * this crate is also suitable for building IPP servers, however the example is not provided yet.
//! * some operations (e.g. CUPS-specific) require authorization which can be supplied in the printer URI.
//!
//! Usage examples:
//!
//!```rust,no_run
//! // using low-level API
//! use ipp::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
//!     let req = IppRequestResponse::new(
//!         IppVersion::v1_1(),
//!         Operation::GetPrinterAttributes,
//!         Some(uri.clone())
//!     );
//!     let client = IppClient::new(uri);
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
//!     let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
//!     let operation = IppOperationBuilder::get_printer_attributes(uri.clone()).build();
//!     let client = IppClient::new(uri);
//!     let attrs = futures::executor::block_on(client.send(operation))?;
//!     for (_, v) in attrs.groups_of(DelimiterTag::PrinterAttributes).next().unwrap().attributes() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//!     Ok(())
//! }
//!```

pub mod proto;

#[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
pub mod client;
pub mod util;

pub mod prelude {
    //!
    //! Common imports
    //!
    pub use http::Uri;
    pub use num_traits::FromPrimitive as _;

    #[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
    pub use super::client::{IppClient, IppError};
    pub use super::proto::{
        attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
        builder::IppOperationBuilder,
        model::*,
        request::IppRequestResponse,
        value::IppValue,
        IppHeader, IppPayload,
    };
}
