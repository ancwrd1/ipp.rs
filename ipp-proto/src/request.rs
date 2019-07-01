//!
//! IPP request
//!
use std::io::{self, Cursor, Write};

use crate::{
    attribute::*,
    ipp::{DelimiterTag, IppVersion, Operation},
    value::*,
    IppHeader, IppReadStream, IppWriter,
};

use crate::parser::IppParseResult;
use bytes::Bytes;
use futures::Stream;
use log::debug;
use tempfile::NamedTempFile;

pub enum PayloadKind {
    Stream(IppReadStream),
    File(NamedTempFile),
}

impl PayloadKind {
    pub fn as_stream(&mut self) -> Option<&mut IppReadStream> {
        match self {
            PayloadKind::Stream(ref mut stream) => Some(stream),
            _ => None,
        }
    }

    pub fn as_file(&mut self) -> Option<&mut NamedTempFile> {
        match self {
            PayloadKind::File(ref mut file) => Some(file),
            _ => None,
        }
    }
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

pub trait IppRequestTrait {
    fn header(&self) -> &IppHeader;
}

impl IppRequestTrait for IppRequestResponse {
    /// Get header
    fn header(&self) -> &IppHeader {
        &self.header
    }
}

impl IppRequestResponse {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: Operation, uri: Option<&str>) -> IppRequestResponse {
        let hdr = IppHeader::new(IppVersion::Ipp11, operation as u16, 1);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE, IppValue::NaturalLanguage("en".to_string())),
        );

        if let Some(uri) = uri {
            retval.set_attribute(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(PRINTER_URI, IppValue::Uri(uri.replace("http", "ipp").to_string())),
            );
        }

        retval
    }

    /// Create response from status and id
    pub fn new_response(status: u16, id: u32) -> IppRequestResponse {
        let hdr = IppHeader::new(IppVersion::Ipp11, status, id);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.set_attribute(
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

    /// Get payload
    pub fn payload(&self) -> &Option<PayloadKind> {
        &self.payload
    }

    /// Get mutable payload
    pub fn payload_mut(&mut self) -> &mut Option<PayloadKind> {
        &mut self.payload
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: IppReadStream) {
        self.payload = Some(PayloadKind::Stream(payload))
    }

    /// Set attribute
    pub fn set_attribute(&mut self, group: DelimiterTag, attribute: IppAttribute) {
        self.attributes.add(group, attribute);
    }

    /// Serialize request into the binary stream (TCP)
    pub fn write(&mut self, writer: &mut Write) -> io::Result<usize> {
        let mut retval = self.header.write(writer)?;

        retval += self.attributes.write(writer)?;

        debug!("Wrote {} bytes IPP stream", retval);

        Ok(retval)
    }

    pub fn into_stream(self) -> Box<dyn Stream<Item = Bytes, Error = io::Error> + Send + 'static> {
        let mut cursor = Cursor::new(Vec::with_capacity(1024));
        let _ = self
            .header
            .write(&mut cursor)
            .and_then(|_| self.attributes.write(&mut cursor));

        let headers = futures::stream::once(Ok(cursor.into_inner().into()));

        match self.payload {
            Some(PayloadKind::Stream(payload)) => Box::new(headers.chain(payload)),
            _ => Box::new(headers),
        }
    }
}
