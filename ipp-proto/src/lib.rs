use std::io::{self, Read, Write};
use std::pin::Pin;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Bytes, BytesMut};
use futures::task::Context;
use futures::{ready, AsyncRead, Poll, Stream};
use num_traits::FromPrimitive;

pub use crate::{
    attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
    builder::{
        CreateJobBuilder, GetPrinterAttributesBuilder, IppOperationBuilder, PrintJobBuilder, SendDocumentBuilder,
    },
    ipp::{IppVersion, Operation, StatusCode},
    parser::{AsyncIppParser, IppParser, ParseError},
    request::{IppRequestResponse, PayloadKind},
    value::IppValue,
};

pub mod attribute;
pub mod builder;
pub mod ipp;
pub mod operation;
pub mod parser;
pub mod request;
pub mod value;

/// Source for IPP data stream (job file)
pub struct IppJobSource {
    inner: Box<dyn AsyncRead + Send + Sync + Unpin>,
}

impl IppJobSource {
    const CHUNK_SIZE: usize = 32768;
}

impl Stream for IppJobSource {
    type Item = io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = [0u8; IppJobSource::CHUNK_SIZE];

        let result = match ready!(Pin::new(&mut *self.inner).poll_read(cx, &mut buffer)) {
            Err(e) => Some(Err(e)),
            Ok(0) => None,
            Ok(size) => Some(Ok(buffer[0..size].into())),
        };

        Poll::Ready(result)
    }
}

impl<T> From<T> for IppJobSource
where
    T: 'static + AsyncRead + Send + Sync + Unpin,
{
    /// Create job source from AsyncRead
    fn from(r: T) -> Self {
        IppJobSource { inner: Box::new(r) }
    }
}

/// Trait which adds two methods to Read implementations: `read_string` and `read_bytes`
pub(crate) trait IppReadExt: Read {
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
    pub fn from_reader(reader: &mut dyn Read) -> Result<IppHeader, ParseError> {
        let retval = IppHeader::new(
            IppVersion::from_u16(reader.read_u16::<BigEndian>()?).ok_or_else(|| ParseError::InvalidVersion)?,
            reader.read_u16::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?,
        );
        Ok(retval)
    }

    /// Create IPP header
    pub fn new(version: IppVersion, operation_status: u16, request_id: u32) -> IppHeader {
        IppHeader {
            version,
            operation_status,
            request_id,
        }
    }

    /// Get operation_status field as Operation enum. If no match found returns error status code
    pub fn operation(&self) -> Result<Operation, StatusCode> {
        Operation::from_u16(self.operation_status).ok_or(StatusCode::ServerErrorOperationNotSupported)
    }
    /// Write header to a given writer
    pub fn write(&self, writer: &mut dyn Write) -> io::Result<usize> {
        writer.write_u16::<BigEndian>(self.version as u16)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_read_header_ok() {
        let data = &[0x01, 0x01, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

        let header = IppHeader::from_reader(&mut Cursor::new(data));
        assert!(header.is_ok());

        let header = header.ok().unwrap();
        assert_eq!(header.version, IppVersion::Ipp11);
        assert_eq!(header.operation_status, 0x1122);
        assert_eq!(header.request_id, 0x33445566);
    }

    #[test]
    fn test_read_header_error() {
        let data = &[0xff, 0, 0, 0, 0, 0, 0, 0];

        let header = IppHeader::from_reader(&mut Cursor::new(data));
        assert!(header.is_err());
        if let Some(ParseError::InvalidVersion) = header.err() {
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_write_header() {
        let header = IppHeader::new(IppVersion::Ipp21, 0x1234, 0xaa55aa55);
        let mut buf = Vec::new();
        assert!(header.write(&mut Cursor::new(&mut buf)).is_ok());
        assert_eq!(buf, vec![0x02, 0x01, 0x12, 0x34, 0xaa, 0x55, 0xaa, 0x55]);
    }
}
