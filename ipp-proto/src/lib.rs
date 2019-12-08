use bytes::{BufMut, Bytes, BytesMut};
use futures::AsyncRead;
pub use num_traits::FromPrimitive;
use tempfile::NamedTempFile;

pub use crate::{
    attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
    builder::{
        CreateJobBuilder, GetPrinterAttributesBuilder, IppOperationBuilder, PrintJobBuilder, SendDocumentBuilder,
    },
    ipp::{IppVersion, Operation, StatusCode},
    parser::{IppParseError, IppParser},
    request::IppRequestResponse,
    value::IppValue,
};

pub mod attribute;
pub mod builder;
pub mod ipp;
pub mod operation;
pub mod parser;
pub mod request;
pub mod value;

pub(crate) enum PayloadKind {
    Read(Box<dyn AsyncRead + Send + Unpin>),
    TempFile(NamedTempFile),
}

/// Source for IPP data stream (job file)
pub struct IppPayload {
    inner: PayloadKind,
}

impl IppPayload {
    pub fn into_reader(self) -> impl AsyncRead + Send + Unpin {
        match self.inner {
            PayloadKind::Read(read) => read,
            PayloadKind::TempFile(file) => Box::new(futures::io::AllowStdIo::new(file)),
        }
    }

    pub fn from_temp_file(temp_file: NamedTempFile) -> IppPayload {
        IppPayload {
            inner: PayloadKind::TempFile(temp_file),
        }
    }
}

impl<T> From<T> for IppPayload
where
    T: 'static + AsyncRead + Send + Unpin,
{
    /// Create job source from AsyncRead
    fn from(r: T) -> Self {
        IppPayload {
            inner: PayloadKind::Read(Box::new(r)),
        }
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

    /// Get operation_status field as Operation enum. If no match found returns error status code
    pub fn operation(&self) -> Result<Operation, StatusCode> {
        Operation::from_u16(self.operation_status).ok_or(StatusCode::ServerErrorOperationNotSupported)
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
