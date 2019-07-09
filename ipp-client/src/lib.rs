//!
//! IPP client
//!
//! Usage examples:
//!
//!```rust,no_run
//! // using raw API
//! use ipp_client::IppClientBuilder;
//! use ipp_proto::{ipp::Operation, request::IppRequestResponse, IppVersion};
//! use tokio::runtime::Runtime;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut runtime = Runtime::new()?;
//!     let uri = "http://localhost:631/printers/test-printer";
//!     let req = IppRequestResponse::new(IppVersion::Ipp11, Operation::GetPrinterAttributes, Some(uri));
//!     let client = IppClientBuilder::new(&uri).build();
//!     let resp = runtime.block_on(client.send_request(req))?;
//!     if resp.header().operation_status <= 2 {
//!         println!("result: {:?}", resp.attributes());
//!     }
//!     Ok(())
//! }
//!```
//!```rust,no_run
//! // using high level API
//! use ipp_proto::{IppOperationBuilder, ipp::DelimiterTag};
//! use ipp_client::IppClientBuilder;
//! use tokio::runtime::Runtime;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut runtime = Runtime::new()?;
//!     let operation = IppOperationBuilder::get_printer_attributes().build();
//!     let client = IppClientBuilder::new("http://localhost:631/printers/test-printer").build();
//!     let attrs = runtime.block_on(client.send(operation))?;
//!     for (_, v) in attrs.groups_of(DelimiterTag::PrinterAttributes)[0].attributes() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//!     Ok(())
//! }
//!```

use std::{
    fmt, io,
    path::{Path, PathBuf},
};

use ipp_proto::{ipp::StatusCode, ParseError};

pub use crate::client::IppClient;

pub mod client;

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP error
    HttpError(reqwest::Error),
    /// Network or file I/O error
    IOError(::std::io::Error),
    /// IPP status error
    StatusError(StatusCode),
    /// Printer state error
    PrinterStateError(Vec<String>),
    /// Printer stopped
    PrinterStopped,
    /// Parameter error
    ParamError(String),
    /// Parsing error
    ParseError(ParseError),
    /// Missing attribute in response
    MissingAttribute,
    /// Invalid attribute type
    InvalidAttributeType,
}

impl fmt::Display for IppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppError::HttpError(ref e) => write!(f, "{}", e),
            IppError::IOError(ref e) => write!(f, "{}", e),
            IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            IppError::ParamError(ref e) => write!(f, "IPP param error: {}", e),
            IppError::PrinterStateError(ref e) => write!(f, "IPP printer state error: {:?}", e),
            IppError::PrinterStopped => write!(f, "IPP printer stopped"),
            IppError::ParseError(ref e) => write!(f, "{}", e),
            IppError::MissingAttribute => write!(f, "Missing attribute in response"),
            IppError::InvalidAttributeType => write!(f, "Invalid attribute type"),
        }
    }
}

impl From<io::Error> for IppError {
    fn from(error: io::Error) -> Self {
        IppError::IOError(error)
    }
}

impl From<StatusCode> for IppError {
    fn from(code: StatusCode) -> Self {
        IppError::StatusError(code)
    }
}

impl From<reqwest::Error> for IppError {
    fn from(error: reqwest::Error) -> Self {
        IppError::HttpError(error)
    }
}

impl From<ParseError> for IppError {
    fn from(error: ParseError) -> Self {
        IppError::ParseError(error)
    }
}

impl std::error::Error for IppError {}

/// Builder to create IPP client
pub struct IppClientBuilder {
    uri: String,
    ca_certs: Vec<PathBuf>,
    verify_hostname: bool,
    verify_certificate: bool,
    timeout: u64,
}

impl IppClientBuilder {
    /// Create a client builder for a given URI
    pub fn new(uri: &str) -> Self {
        IppClientBuilder {
            uri: uri.to_owned(),
            ca_certs: Vec::new(),
            verify_hostname: true,
            verify_certificate: true,
            timeout: 0,
        }
    }

    /// Add CA certificate
    pub fn ca_cert<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.ca_certs.push(path.as_ref().to_owned());
        self
    }

    /// Add CA certificates
    pub fn ca_certs<P>(mut self, paths: &[P]) -> Self
    where
        P: AsRef<Path>,
    {
        self.ca_certs.extend(paths.iter().map(|p| p.as_ref().to_owned()));
        self
    }

    /// Enable or disable host name verification
    pub fn verify_hostname(mut self, verify: bool) -> Self {
        self.verify_hostname = verify;
        self
    }

    /// Enable or disable server certificate verification
    pub fn verify_certificate(mut self, verify: bool) -> Self {
        self.verify_certificate = verify;
        self
    }

    /// Set network timeout in seconds
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the client
    pub fn build(self) -> IppClient {
        IppClient {
            uri: self.uri,
            ca_certs: self.ca_certs,
            verify_hostname: self.verify_hostname,
            verify_certificate: self.verify_certificate,
            timeout: self.timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let mut builder = IppClientBuilder::new("foobar");
        assert_eq!(builder.uri, "foobar");

        let cert = PathBuf::from("mycert");
        builder = builder.ca_cert(&cert);
        assert_eq!(builder.ca_certs, vec![cert.clone()]);

        builder = builder.ca_certs(&[cert.clone()]);
        assert_eq!(builder.ca_certs, vec![cert.clone(), cert.clone()]);

        builder = builder.verify_hostname(false);
        assert!(!builder.verify_hostname);

        builder = builder.verify_certificate(false);
        assert!(!builder.verify_certificate);

        builder = builder.timeout(100);
        assert_eq!(builder.timeout, 100);

        let _ = builder.build();
    }
}
