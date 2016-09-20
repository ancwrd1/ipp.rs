//!
//! IPP protocol implementation for Rust
//!
//! Usage examples:
//!
//!```rust
//! // using raw API
//! let mut req = IppRequest::new(GET_PRINTER_ATTRIBUTES, "http://localhost:631/printers/test-printer");
//! let client = IppClient::new();
//! let attrs = client.send(&mut req).unwrap();
//! for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
//!     println!("{}: {}", v.name(), v.value());
//! }
//!
//! // using operation API
//! let mut operation = GetPrinterAttributes::new("http://localhost:631/printers/test-printer");
//! let attrs = operation.execute().unwrap();
//! for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
//!     println!("{}: {}", v.name(), v.value());
//! }

//!```

extern crate byteorder;
extern crate hyper;
extern crate mime;

#[macro_use]
extern crate log;

use std::result;
use std::convert::From;
use std::io::{self, Read};

const IPP_VERSION: u16 = 0x0101;

/// IPP value
#[derive(Debug)]
pub enum IppError {
    HttpError(hyper::Error),
    IOError(::std::io::Error),
    RequestError,
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

/// Trait which adds two methods to Read implementations: read_string and read_vec
pub trait ReadIppExt: Read {
    fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&try!(self.read_vec(len))).to_string())
    }

    fn read_vec(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut namebuf: Vec<u8> = Vec::with_capacity(len);
        unsafe { namebuf.set_len(len) };

        try!(self.read_exact(&mut namebuf[..]));

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
