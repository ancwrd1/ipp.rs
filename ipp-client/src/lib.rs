use std::{fmt, io};

use ipp_proto::{ipp::StatusCode, value::ValueParseError, IppParseError};

pub use crate::client::IppClient;

pub mod client;

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP protocol error
    HttpError(http::Error),
    /// Client error
    ClientError(isahc::Error),
    /// HTTP request error
    RequestError(u16),
    /// Network or file I/O error
    IOError(io::Error),
    /// IPP status error
    StatusError(StatusCode),
    /// Printer state error
    PrinterStateError(Vec<String>),
    /// Printer stopped
    PrinterStopped,
    /// Parameter error
    ParamError(String),
    /// Parsing error
    ParseError(IppParseError),
    /// Value parsing error
    ValueParseError(ValueParseError),
    /// Missing attribute in response
    MissingAttribute,
    /// Invalid attribute type
    InvalidAttributeType,
}

impl fmt::Display for IppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppError::HttpError(ref e) => write!(f, "{}", e),
            IppError::ClientError(ref e) => write!(f, "{}", e),
            IppError::RequestError(ref e) => write!(f, "HTTP request failed: {}", e),
            IppError::IOError(ref e) => write!(f, "{}", e),
            IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            IppError::ParamError(ref e) => write!(f, "IPP param error: {}", e),
            IppError::PrinterStateError(ref e) => write!(f, "IPP printer state error: {:?}", e),
            IppError::PrinterStopped => write!(f, "IPP printer stopped"),
            IppError::ParseError(ref e) => write!(f, "{}", e),
            IppError::ValueParseError(ref e) => write!(f, "{}", e),
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

impl From<http::Error> for IppError {
    fn from(error: http::Error) -> Self {
        IppError::HttpError(error)
    }
}

impl From<isahc::Error> for IppError {
    fn from(error: isahc::Error) -> Self {
        IppError::ClientError(error)
    }
}

impl From<IppParseError> for IppError {
    fn from(error: IppParseError) -> Self {
        IppError::ParseError(error)
    }
}

impl From<ValueParseError> for IppError {
    fn from(error: ValueParseError) -> Self {
        IppError::ValueParseError(error)
    }
}

impl std::error::Error for IppError {}

/// Builder to create IPP client
pub struct IppClientBuilder {
    uri: String,
    ignore_tls_errors: bool,
    timeout: u64,
}

impl IppClientBuilder {
    /// Create a client builder for a given URI
    pub fn new(uri: &str) -> Self {
        IppClientBuilder {
            uri: uri.to_owned(),
            ignore_tls_errors: false,
            timeout: 0,
        }
    }

    /// Enable or disable ignoring of TLS handshake errors. Default is false.
    pub fn ignore_tls_errors(mut self, flag: bool) -> Self {
        self.ignore_tls_errors = flag;
        self
    }

    /// Set network timeout in seconds. Default is 0 (no timeout)
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the client
    pub fn build(self) -> IppClient {
        IppClient {
            uri: self.uri,
            ignore_tls_errors: self.ignore_tls_errors,
            timeout: self.timeout,
        }
    }
}
