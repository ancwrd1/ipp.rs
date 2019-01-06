//!
//! IPP client
//!
//! Usage examples:
//!
//!```rust
//! // using raw API
//! extern crate ipp_client;
//! extern crate ipp_parse;
//! extern crate ipp_proto;
//! use ipp_proto::request::IppRequestResponse;
//! use ipp_client::IppClient;
//! use ipp_parse::ipp::Operation;
//!
//! fn main() {
//!     let uri = "http://localhost:631/printers/test-printer";
//!     let req = IppRequestResponse::new(Operation::GetPrinterAttributes, uri);
//!     let client = IppClient::new(uri);
//!     if let Ok(resp) = client.send_request(req) {
//!         if resp.header().operation_status <= 2 {
//!             println!("result: {:?}", resp.attributes());
//!         }
//!     }
//! }
//!```
//!```rust
//! // using high level API
//! extern crate ipp_client;
//! extern crate ipp_proto;
//! use ipp_proto::IppOperationBuilder;
//! use ipp_client::IppClientBuilder;
//!
//! fn main() {
//!     let operation = IppOperationBuilder::get_printer_attributes().build();
//!     let client = IppClientBuilder::new("http://localhost:631/printers/test-printer").build();
//!     if let Ok(attrs) = client.send(operation) {
//!         for (_, v) in attrs.printer_attributes().unwrap() {
//!             println!("{}: {}", v.name(), v.value());
//!         }
//!     }
//! }
//!```

use std::{
    fmt, io,
    path::{Path, PathBuf},
};

use ipp_parse::ipp::StatusCode;

pub mod client;

pub use crate::client::IppClient;

const DEFAULT_TIMEOUT: u64 = 30;

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP error
    HttpError(reqwest::Error),
    /// Network or file I/O error
    IOError(::std::io::Error),
    /// IPP request error
    RequestError(String),
    /// IPP status error
    StatusError(StatusCode),
    /// Printer state error
    PrinterStateError(Vec<String>),
    /// Parameter error
    ParamError(String),
}

impl IppError {
    pub fn as_exit_code(&self) -> i32 {
        match *self {
            IppError::HttpError(_) => 2,
            IppError::IOError(_) => 3,
            IppError::RequestError(_) => 4,
            IppError::StatusError(_) => 6,
            IppError::ParamError(_) => 7,
            IppError::PrinterStateError(_) => 8,
        }
    }
}

impl fmt::Display for IppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppError::HttpError(ref e) => write!(f, "{}", e),
            IppError::IOError(ref e) => write!(f, "{}", e),
            IppError::RequestError(ref e) => write!(f, "IPP request error: {}", e),
            IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            IppError::ParamError(ref e) => write!(f, "IPP param error: {}", e),
            IppError::PrinterStateError(ref e) => write!(f, "IPP printer state error: {:?}", e),
        }
    }
}

impl From<io::Error> for IppError {
    fn from(error: io::Error) -> IppError {
        IppError::IOError(error)
    }
}

impl From<StatusCode> for IppError {
    fn from(code: StatusCode) -> IppError {
        IppError::StatusError(code)
    }
}

impl From<reqwest::Error> for IppError {
    fn from(error: reqwest::Error) -> IppError {
        IppError::HttpError(error)
    }
}

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
            timeout: DEFAULT_TIMEOUT,
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
        let mut client = IppClient::new(&self.uri);
        client.set_verify_hostname(self.verify_hostname);
        client.set_verify_certificate(self.verify_certificate);
        for cert in self.ca_certs {
            client.add_root_certificate(&cert);
        }
        client.set_timeout(self.timeout);
        client
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
