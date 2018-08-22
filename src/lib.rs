//!
//! IPP protocol implementation for Rust
//!
//! Usage examples:
//!
//!```rust
//! // using raw API
//! use ipp::{IppRequestResponse, IppClient};
//! use ipp::consts::operation::Operation;
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
//!```rust
//! // using operation API
//! use ipp::{GetPrinterAttributes, IppClient};

//! let operation = GetPrinterAttributes::new();
//! let client = IppClient::new("http://localhost:631/printers/test-printer");
//! if let Ok(attrs) = client.send(operation) {
//!     for (_, v) in attrs.get_printer_attributes().unwrap() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//! }

//!```

extern crate byteorder;
extern crate clap;
extern crate num_traits;
extern crate reqwest;
extern crate url;
#[macro_use]
extern crate enum_primitive_derive;
#[macro_use]
extern crate log;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{self, Read, Write};
use std::result;

pub mod consts {
    //! This module holds IPP constants such as attribute names, operations and tags
    pub mod attribute;
    pub mod operation;
    pub mod statuscode;
    pub mod tag;
}

pub mod attribute;
pub mod client;
pub mod operation;
pub mod parser;
pub mod request;
pub mod server;
pub mod util;
pub mod value;

pub use attribute::{IppAttribute, IppAttributeList};
pub use client::IppClient;
pub use operation::{CreateJob, GetPrinterAttributes, IppOperation, PrintJob, SendDocument};
pub use request::IppRequestResponse;
pub use value::IppValue;
pub const IPP_VERSION: u16 = 0x0101;

use consts::statuscode::StatusCode;

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP error
    HttpError(reqwest::Error),
    /// Network or file I/O error
    IOError(::std::io::Error),
    /// IPP request error
    RequestError(String),
    /// IPP attribute error
    AttributeError(String),
    /// IPP status error
    StatusError(consts::statuscode::StatusCode),
    /// IPP binary tag error
    TagError(u8),
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
            IppError::AttributeError(_) => 5,
            IppError::StatusError(_) => 6,
            IppError::TagError(_) => 7,
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
            IppError::AttributeError(ref e) => write!(f, "IPP attribute error: {}", e),
            IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            IppError::TagError(ref e) => write!(f, "IPP tag error: {:0x}", e),
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

pub type Result<T> = result::Result<T, IppError>;

/// IPP request and response header
#[derive(Clone, Debug)]
pub struct IppHeader {
    /// IPP protocol version in big endian encoding, for example 0x0101 for version 1.1
    pub version: u16,
    /// Operation tag for requests, status for responses
    pub operation_status: u16,
    /// ID of the request
    pub request_id: u32,
}

impl IppHeader {
    pub fn from_reader(reader: &mut Read) -> Result<IppHeader> {
        let retval = IppHeader::new(
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?,
        );
        Ok(retval)
    }

    /// Create IPP header
    pub fn new(version: u16, status: u16, request_id: u32) -> IppHeader {
        IppHeader {
            version,
            operation_status: status,
            request_id,
        }
    }

    /// Write header to a given writer
    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        writer.write_u16::<BigEndian>(self.version)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}

/// Trait which adds two methods to Read implementations: `read_string` and `read_vec`
pub trait ReadIppExt: Read {
    fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&self.read_vec(len)?).to_string())
    }

    fn read_vec(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut namebuf: Vec<u8> = Vec::with_capacity(len);
        unsafe { namebuf.set_len(len) };

        self.read_exact(&mut namebuf)?;

        Ok(namebuf)
    }
}

impl<R: io::Read + ?Sized> ReadIppExt for R {}

#[test]
fn it_works() {}
