//!
//! IPP protocol implementation for Rust
//!
//! Usage examples:
//!
//!```rust
//! // using raw API
//! let mut req = IppRequest::new(GET_PRINTER_ATTRIBUTES);
//! let client = IppClient::new("http://localhost:631/printers/test-printer");
//! let resp = client.send_request(&mut req).unwrap();
//! if resp.header().status <= 3 {
//!     println!("result: {:?}", resp.attributes());
//! }
//!
//! // using operation API
//! let mut operation = GetPrinterAttributes::new();
//! let client = IppClient::new("http://localhost:631/printers/test-printer");
//! let attrs = client.send(&mut operation).unwrap();
//! for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
//!     println!("{}: {}", v.name(), v.value());
//! }

//!```

extern crate byteorder;
extern crate hyper;
extern crate hyper_openssl;

#[macro_use]
extern crate log;

use std::result;
use std::io::{self, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub const IPP_VERSION: u16 = 0x0101;

/// IPP value
#[derive(Debug)]
pub enum IppError {
    HttpError(hyper::Error),
    IOError(::std::io::Error),
    RequestError(String),
    AttributeError(String),
    StatusError(u16),
    TagError(u8)
}

impl From<io::Error> for IppError {
    fn from(error: io::Error) -> IppError {
        IppError::IOError(error)
    }
}

impl From<hyper::Error> for IppError {
    fn from(error: hyper::Error) -> IppError {
        IppError::HttpError(error)
    }
}

pub type Result<T> = result::Result<T, IppError>;

/// IPP request and response header
#[derive(Clone, Debug)]
pub struct IppHeader {
    pub version: u16,
    pub status: u16,
    pub request_id: u32
}

impl IppHeader {
    pub fn from_reader(reader: &mut Read) -> Result<IppHeader> {
        let retval = IppHeader::new(
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?);
        Ok(retval)
    }

    /// Create IPP header
    pub fn new(version: u16, status: u16, request_id: u32) -> IppHeader {
        IppHeader {version: version, status: status, request_id: request_id}
    }

    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        writer.write_u16::<BigEndian>(self.version)?;
        writer.write_u16::<BigEndian>(self.status)?;
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

pub mod consts {
    //! This module holds IPP constants such as attribute names, operations and tags
    pub mod tag;
    pub mod statuscode;
    pub mod operation;
    pub mod attribute;
}

pub mod value;
pub mod parser;
pub mod request;
pub mod response;
pub mod attribute;
pub mod client;
pub mod server;
pub mod operation;

pub use attribute::{IppAttribute, IppAttributeList};
pub use client::IppClient;
pub use operation::{IppOperation, PrintJob, GetPrinterAttributes, CreateJob, SendDocument};
pub use request::IppRequest;
pub use response::IppResponse;
pub use value::IppValue;
