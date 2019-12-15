use bytes::{BufMut, Bytes, BytesMut};
use futures_util::io::AsyncRead;

pub use {
    attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
    builder::{
        CreateJobBuilder, CupsBuilder, GetPrinterAttributesBuilder, IppOperationBuilder, PrintJobBuilder,
        SendDocumentBuilder,
    },
    model::{IppVersion, Operation, StatusCode},
    num_traits::FromPrimitive,
    parser::{IppParseError, IppParser},
    request::IppRequestResponse,
    value::IppValue,
};

pub mod attribute;
pub mod builder;
pub mod model;
pub mod operation;
pub mod parser;
pub mod request;
pub mod value;

/// IPP payload
pub struct IppPayload {
    inner: Box<dyn AsyncRead + Send + Unpin>,
}

impl IppPayload {
    /// Consumes the payload and returns an inner AsyncRead
    pub fn into_inner(self) -> impl AsyncRead + Send {
        self.inner
    }

    /// Create a payload from the AsyncRead instance
    pub fn new<R>(r: R) -> IppPayload
    where
        R: 'static + AsyncRead + Send + Unpin,
    {
        IppPayload { inner: Box::new(r) }
    }
}

impl<T> From<T> for IppPayload
where
    T: 'static + AsyncRead + Send + Unpin,
{
    fn from(r: T) -> Self {
        IppPayload::new(r)
    }
}

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
    /// Create IPP header
    pub fn new(version: IppVersion, operation_status: u16, request_id: u32) -> IppHeader {
        IppHeader {
            version,
            operation_status,
            request_id,
        }
    }

    /// Write header to a given writer
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u16(self.version as u16);
        buffer.put_u16(self.operation_status);
        buffer.put_u32(self.request_id);

        buffer.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_to_bytes() {
        let header = IppHeader::new(IppVersion::Ipp21, 0x1234, 0xaa55_aa55);
        let buf = header.to_bytes();
        assert_eq!(buf, vec![0x02, 0x01, 0x12, 0x34, 0xaa, 0x55, 0xaa, 0x55]);
    }
}
