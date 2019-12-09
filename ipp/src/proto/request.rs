//!
//! IPP request
//!
use bytes::{BufMut, Bytes, BytesMut};
use futures::{AsyncRead, AsyncReadExt};
use log::debug;

use super::{
    model::{DelimiterTag, IppVersion, Operation},
    value::*,
    IppAttribute, IppAttributes, IppHeader, IppPayload, StatusCode,
};

/// IPP request/response struct
pub struct IppRequestResponse {
    pub(crate) header: IppHeader,
    pub(crate) attributes: IppAttributes,
    pub(crate) payload: Option<IppPayload>,
}

impl IppRequestResponse {
    /// Create new IPP request for the operation and uri
    pub fn new<S>(version: IppVersion, operation: Operation, uri: Option<S>) -> IppRequestResponse
    where
        S: AsRef<str>,
    {
        let hdr = IppHeader::new(version, operation as u16, 1);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        if let Some(uri) = uri {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(
                    IppAttribute::PRINTER_URI,
                    IppValue::Uri(uri.as_ref().replace("http", "ipp").to_string()),
                ),
            );
        }

        retval
    }

    /// Create response from status and id
    pub fn new_response(version: IppVersion, status: StatusCode, id: u32) -> IppRequestResponse {
        let hdr = IppHeader::new(version, status as u16, id);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        retval
    }

    /// Get IPP header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get mutable IPP header
    pub fn header_mut(&mut self) -> &mut IppHeader {
        &mut self.header
    }

    /// Get attributes
    pub fn attributes(&self) -> &IppAttributes {
        &self.attributes
    }

    /// Get attributes
    pub fn attributes_mut(&mut self) -> &mut IppAttributes {
        &mut self.attributes
    }

    /// Get payload
    pub fn payload(&self) -> Option<&IppPayload> {
        self.payload.as_ref()
    }

    /// Get mutable payload
    pub fn payload_mut(&mut self) -> &mut Option<IppPayload> {
        &mut self.payload
    }

    /// Write request to byte array not including payload
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put(self.header.to_bytes());
        buffer.put(self.attributes.to_bytes());
        buffer.freeze()
    }

    /// Convert request/response into AsyncRead including payload
    pub fn into_reader(self) -> impl AsyncRead + Send + Unpin + 'static {
        let header = self.to_bytes();
        debug!(
            "IPP header size: {}, has payload: {}",
            header.len(),
            self.payload.is_some()
        );

        let payload = self.payload.unwrap_or_else(|| futures::io::empty().into());
        futures::io::Cursor::new(header).chain(payload.into_inner())
    }
}
