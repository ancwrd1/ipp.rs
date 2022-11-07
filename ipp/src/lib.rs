//!
//! IPP print protocol implementation for Rust. This crate can be used in several ways:
//! * using the low-level request/response API and building the requests manually.
//! * using the higher-level operations API with builders. Currently only a subset of all IPP operations is supported.
//! * using the built-in IPP client.
//! * using any third-party HTTP client and send the serialized request manually.
//!
//! This crate supports both synchronous and asynchronous operations. The following feature flags are supported:
//! * `async` - enable async APIs (parser, I/O)
//! * `async-client` - enable async HTTP client via `request` crate
//! * `client` - enable blocking HTTP client via `ureq` crate
//! * `tls` - enable TLS support via `native-tls` crate
//!
//! By default, all features are enabled.
//!
//!
//! Implementation notes:
//! * all RFC IPP values are supported including arrays and collections, for both de- and serialization.
//! * this crate is also suitable for building IPP servers, however the example is not provided yet.
//! * some operations (e.g. CUPS-specific) require authorization which can be supplied in the printer URI.
//!
//! Usage examples:
//!
//!```rust,no_run
//! // using low-level async API
//! use ipp::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
//!     let req = IppRequestResponse::new(
//!         IppVersion::v1_1(),
//!         Operation::GetPrinterAttributes,
//!         Some(uri.clone())
//!     );
//!     let client = AsyncIppClient::new(uri);
//!     let resp = client.send(req).await?;
//!     if resp.header().status_code().is_success() {
//!         println!("{:?}", resp.attributes());
//!     }
//!     Ok(())
//! }
//!```
//!```rust,no_run
//! // using blocking operations API
//! use ipp::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let uri: Uri = "http://localhost:631/printers/test-printer".parse()?;
//!     let operation = IppOperationBuilder::get_printer_attributes(uri.clone()).build();
//!     let client = IppClient::new(uri);
//!     let resp = client.send(operation)?;
//!     if resp.header().status_code().is_success() {
//!         println!("{:?}", resp.attributes());
//!     }
//!     Ok(())
//! }
//!```

use bytes::{BufMut, Bytes, BytesMut};
use num_traits::FromPrimitive;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::model::{IppVersion, StatusCode};

pub mod attribute;
pub mod builder;
#[cfg(any(feature = "client", feature = "async-client"))]
pub mod client;
pub mod error;
pub mod model;
pub mod operation;
pub mod parser;
pub mod payload;
pub mod reader;
pub mod request;
pub mod util;
pub mod value;

pub mod prelude {
    //!
    //! Common imports
    //!
    pub use http::Uri;
    pub use num_traits::FromPrimitive as _;

    pub use crate::{
        attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
        builder::IppOperationBuilder,
        model::*,
        payload::IppPayload,
        request::IppRequestResponse,
        value::IppValue,
    };

    pub use super::error::IppError;

    #[cfg(feature = "async-client")]
    pub use super::client::non_blocking::AsyncIppClient;

    #[cfg(feature = "client")]
    pub use super::client::blocking::IppClient;

    pub use super::IppHeader;
}

/// IPP request and response header
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct IppHeader {
    /// IPP protocol version
    pub version: IppVersion,
    /// Operation tag for requests, status for responses
    pub operation_or_status: u16,
    /// ID of the request
    pub request_id: u32,
}

impl IppHeader {
    /// Create IPP header
    pub fn new(version: IppVersion, operation_or_status: u16, request_id: u32) -> IppHeader {
        IppHeader {
            version,
            operation_or_status,
            request_id,
        }
    }

    /// Write header to a given writer
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u16(self.version.0);
        buffer.put_u16(self.operation_or_status);
        buffer.put_u32(self.request_id);

        buffer.freeze()
    }

    /// Decode and get IPP status code from the header
    pub fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.operation_or_status).unwrap_or(StatusCode::UnknownStatusCode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_to_bytes() {
        let header = IppHeader::new(IppVersion::v2_1(), 0x1234, 0xaa55_aa55);
        let buf = header.to_bytes();
        assert_eq!(buf, vec![0x02, 0x01, 0x12, 0x34, 0xaa, 0x55, 0xaa, 0x55]);
    }
}
