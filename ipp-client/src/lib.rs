//!
//! IPP client
//!
//! Usage examples:
//!
//!```rust,ignore
//! // using raw API
//! use ippproto::request::IppRequestResponse;
//! use ippclient::IppClient;
//! use ippparse::ipp::Operation;
//!
//! let uri = "http://localhost:631/printers/test-printer";
//! let req = IppRequestResponse::new(Operation::GetPrinterAttributes, uri);
//! let client = IppClient::new(uri);
//! if let Ok(resp) = client.send_request(req) {
//!     if resp.header().operation_status <= 3 {
//!         println!("result: {:?}", resp.attributes());
//!     }
//! }
//!```
//!
//!```rust,ignore
//! // using operation API
//! use ippproto::IppOperationBuilder;
//! use IppClient;

//! let operation = IppOperationBuilder::get_printer_attributes().build();
//! let client = IppClient::new("http://localhost:631/printers/test-printer");
//! if let Ok(attrs) = client.send(operation) {
//!     for (_, v) in attrs.printer_attributes().unwrap() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//! }

//!```

extern crate clap;
extern crate ippparse;
extern crate ippproto;
extern crate log;
extern crate num_traits;
extern crate reqwest;
extern crate url;

use std::fmt;
use std::io;

use ippparse::ipp::StatusCode;

pub mod client;
pub mod util;

pub use client::IppClient;

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
    /// Command-line parameter error
    ParamError(clap::Error),
    /// Printer state error
    PrinterStateError(Vec<String>),
}

impl IppError {
    pub fn as_exit_code(&self) -> i32 {
        match *self {
            IppError::HttpError(_) => 2,
            IppError::IOError(_) => 3,
            IppError::RequestError(_) => 4,
            IppError::StatusError(_) => 6,
            IppError::ParamError(_) => 1,
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
            IppError::ParamError(ref e) => write!(f, "IPP tag error: {}", e),
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

impl From<clap::Error> for IppError {
    fn from(error: clap::Error) -> IppError {
        IppError::ParamError(error)
    }
}
