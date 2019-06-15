//!
//! IPP request
//!
use std::io::{self, Cursor, Write};

use crate::{
    attribute::*,
    ipp::{DelimiterTag, IppVersion, Operation},
    parser::{IppParser, ParseError},
    value::*,
    IppHeader, IppReadStream, IppWriter,
};

use bytes::Bytes;
use futures::Stream;
use log::debug;

/// IPP request/response struct
pub struct IppRequestResponse {
    /// IPP header
    header: IppHeader,
    /// IPP attributes
    attributes: IppAttributes,
    /// Optional payload after IPP-encoded stream (for example binary data for Print-Job operation)
    payload: Option<IppReadStream>,
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
    pub fn new(operation: Operation, uri: &str) -> IppRequestResponse {
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

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(PRINTER_URI, IppValue::Uri(uri.replace("http", "ipp").to_string())),
        );

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

    /// Create IppRequestResponse from the parser
    pub fn from_parser(parser: IppParser) -> Result<IppRequestResponse, ParseError> {
        let res = parser.parse()?;

        Ok(IppRequestResponse {
            header: res.header,
            attributes: res.attributes,
            payload: None,
        })
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
    pub fn payload(&self) -> &Option<IppReadStream> {
        &self.payload
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: IppReadStream) {
        self.payload = Some(payload)
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
            Some(payload) => Box::new(headers.chain(payload)),
            None => Box::new(headers),
        }
    }
}
