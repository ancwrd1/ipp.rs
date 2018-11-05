use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

extern crate byteorder;
extern crate log;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod attribute;
pub mod parser;
pub mod rfc2911;
pub mod value;

pub const IPP_VERSION: u16 = 0x0101;

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
    pub fn from_reader(reader: &mut Read) -> io::Result<IppHeader> {
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
    pub fn write(&self, writer: &mut Write) -> io::Result<usize> {
        writer.write_u16::<BigEndian>(self.version)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}
