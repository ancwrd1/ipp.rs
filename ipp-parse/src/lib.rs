use std::io::{self, Cursor, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Bytes, BytesMut};
use num_traits::FromPrimitive;

pub mod attribute;
pub mod ipp;
pub mod parser;
pub mod value;

pub use crate::{
    attribute::{IppAttribute, IppAttributes},
    ipp::IppVersion,
    parser::IppParser,
    value::IppValue,
};

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
            IppVersion::from_u16(reader.read_u16::<BigEndian>()?)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid IPP version"))?,
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
        writer.write_u16::<BigEndian>(self.version as u16)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_header_ok() {
        let data = &[0x01, 0x01, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

        let header = IppHeader::from_reader(&mut Cursor::new(data));
        assert!(header.is_ok());
        assert_eq!(header.as_ref().unwrap().version, IppVersion::Ipp11);
        assert_eq!(header.as_ref().unwrap().operation_status, 0x1122);
        assert_eq!(header.as_ref().unwrap().request_id, 0x33445566);
    }

    #[test]
    fn test_read_header_error() {
        let data = &[0xff, 0, 0, 0, 0, 0, 0, 0];

        let header = IppHeader::from_reader(&mut Cursor::new(data));
        assert!(header.is_err());
        assert_eq!(header.err().unwrap().to_string(), "Invalid IPP version");
    }

    #[test]
    fn test_write_header() {
        let header = IppHeader::new(IppVersion::Ipp21, 0x1234, 0xaa55aa55);
        let mut buf = Vec::new();
        assert!(header.write(&mut Cursor::new(&mut buf)).is_ok());
        assert_eq!(buf, vec![0x02, 0x01, 0x12, 0x34, 0xaa, 0x55, 0xaa, 0x55]);
    }
}
