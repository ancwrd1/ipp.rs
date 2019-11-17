//!
//! IPP request
//!
use std::io::{self, Cursor, Write};

use enum_as_inner::EnumAsInner;
use futures::{AsyncRead, AsyncReadExt};
use log::debug;
use tempfile::NamedTempFile;

use crate::{
    attribute::*,
    ipp::{DelimiterTag, IppVersion, Operation},
    parser::IppParseResult,
    value::*,
    IppHeader, IppJobSource, StatusCode,
};

/// Payload type inside the IppRequestResponse
#[derive(EnumAsInner)]
pub enum PayloadKind {
    /// Job source for client side
    JobSource(IppJobSource),
    /// Received data for server side
    ReceivedData(NamedTempFile),
}

/// IPP request/response struct
pub struct IppRequestResponse {
    /// IPP header
    header: IppHeader,
    /// IPP attributes
    attributes: IppAttributes,
    /// Optional payload after IPP-encoded stream (for example binary data for Print-Job operation)
    payload: Option<PayloadKind>,
}

impl IppRequestResponse {
    /// Create new IPP request for the operation and uri
    pub fn new(version: IppVersion, operation: Operation, uri: Option<&str>) -> IppRequestResponse {
        let hdr = IppHeader::new(version, operation as u16, 1);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE, IppValue::NaturalLanguage("en".to_string())),
        );

        if let Some(uri) = uri {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(PRINTER_URI, IppValue::Uri(uri.replace("http", "ipp").to_string())),
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
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE, IppValue::NaturalLanguage("en".to_string())),
        );

        retval
    }

    /// Create IppRequestResponse from parse result
    pub fn from_parse_result(result: IppParseResult) -> IppRequestResponse {
        IppRequestResponse {
            header: result.header,
            attributes: result.attributes,
            payload: result.payload,
        }
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
    pub fn payload(&self) -> &Option<PayloadKind> {
        &self.payload
    }

    /// Get mutable payload
    pub fn payload_mut(&mut self) -> &mut Option<PayloadKind> {
        &mut self.payload
    }

    /// Set payload
    pub fn add_payload(&mut self, payload: IppJobSource) {
        self.payload = Some(PayloadKind::JobSource(payload))
    }

    /// Serialize request into the binary stream (TCP)
    pub fn write(&mut self, writer: &mut dyn Write) -> io::Result<usize> {
        let mut retval = self.header.write(writer)?;

        retval += self.attributes.write(writer)?;

        debug!("Wrote {} bytes IPP stream", retval);

        Ok(retval)
    }

    /// Convert request/response into Stream
    pub fn into_reader(self) -> Box<dyn AsyncRead + Send + Unpin + 'static> {
        let mut header = Cursor::new(Vec::with_capacity(1024));
        let _ = self
            .header
            .write(&mut header)
            .and_then(|_| self.attributes.write(&mut header));

        let cursor = futures::io::Cursor::new(header.into_inner());
        debug!("IPP header size: {}", cursor.get_ref().len());

        match self.payload {
            Some(PayloadKind::JobSource(payload)) => {
                debug!("Adding payload to a reader chain");
                Box::new(cursor.chain(payload.into_reader()))
            }
            _ => Box::new(cursor),
        }
    }
}
