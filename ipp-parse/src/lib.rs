extern crate byteorder;
extern crate bytes;
extern crate enum_primitive_derive;
extern crate log;
extern crate num_traits;

use std::fmt::{self, Formatter};
use std::io::{self, Cursor, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Bytes, BytesMut};

pub mod attribute;
pub mod ipp;
pub mod parser;
pub mod value;

pub use attribute::{IppAttribute, IppAttributes};
pub use parser::IppParser;
pub use value::IppValue;

/// IPP protocol version
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IppVersion {
    Ipp10,
    Ipp11,
    Ipp20,
    Ipp21,
    Ipp22,
}

impl IppVersion {
    /// Construct IppVersion from reader
    pub fn from_reader(reader: &mut Read) -> io::Result<IppVersion> {
        Ok(match reader.read_u16::<BigEndian>()? {
            0x0100 => IppVersion::Ipp10,
            0x0101 => IppVersion::Ipp11,
            0x0200 => IppVersion::Ipp20,
            0x0201 => IppVersion::Ipp21,
            0x0202 => IppVersion::Ipp22,
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid IPP version: {:0x}", other),
                ))
            }
        })
    }

    pub fn as_u16(&self) -> u16 {
        match self {
            IppVersion::Ipp10 => 0x0100,
            IppVersion::Ipp11 => 0x0101,
            IppVersion::Ipp20 => 0x0200,
            IppVersion::Ipp21 => 0x0201,
            IppVersion::Ipp22 => 0x0202,
        }
    }
}

impl IppWriter for IppVersion {
    /// Write IppVersion into stream as big endian u16 integer
    fn write(&self, writer: &mut Write) -> io::Result<usize> {
        writer.write_u16::<BigEndian>(self.as_u16())?;
        Ok(2)
    }
}

impl fmt::Display for IppVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IppVersion::Ipp10 => write!(f, "1.0"),
            IppVersion::Ipp11 => write!(f, "1.1"),
            IppVersion::Ipp20 => write!(f, "2.0"),
            IppVersion::Ipp21 => write!(f, "2.1"),
            IppVersion::Ipp22 => write!(f, "2.2"),
        }
    }
}

pub trait IppWriter {
    fn write(&self, writer: &mut Write) -> io::Result<usize>;
}

pub trait IppIntoReader: IppWriter {
    fn into_reader(self) -> Box<Read>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        self.write(&mut buf).unwrap();
        Box::new(Cursor::new(buf))
    }
}
impl<R: IppWriter + Sized> IppIntoReader for R {}

/// Trait which adds two methods to Read implementations: `read_string` and `read_bytes`
pub trait IppReadExt: Read {
    fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&self.read_bytes(len)?).to_string())
    }

    fn read_bytes(&mut self, len: usize) -> std::io::Result<Bytes> {
        let mut buf = BytesMut::with_capacity(len);
        buf.resize(len, 0);
        self.read_exact(&mut buf)?;

        Ok(buf.freeze())
    }
}

impl<R: io::Read + ?Sized> IppReadExt for R {}

/// IPP request and response header
#[derive(Clone, Debug)]
pub struct IppHeader {
    /// IPP protocol version
    pub version: IppVersion,
    /// Operation tag for requests, status for responses
    pub operation_status: u16,
    /// ID of the request
    pub request_id: u32,
}

impl IppHeader {
    /// Create IppHeader from the reader
    pub fn from_reader(reader: &mut Read) -> io::Result<IppHeader> {
        let retval = IppHeader::new(
            IppVersion::from_reader(reader)?,
            reader.read_u16::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?,
        );
        Ok(retval)
    }

    /// Create IPP header
    pub fn new(version: IppVersion, status: u16, request_id: u32) -> IppHeader {
        IppHeader {
            version,
            operation_status: status,
            request_id,
        }
    }
}

impl IppWriter for IppHeader {
    /// Write header to a given writer
    fn write(&self, writer: &mut Write) -> io::Result<usize> {
        self.version.write(writer)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}
