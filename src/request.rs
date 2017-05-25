//!
//! IPP request
//!
use std::io::{self, Read, Write};

use attribute::{IppAttribute, IppAttributeList};
use ::{Result, IPP_VERSION, IppHeader};
use consts::tag::DelimiterTag;
use consts::operation::Operation;
use consts::attribute::{PRINTER_URI, ATTRIBUTES_CHARSET, ATTRIBUTES_NATURAL_LANGUAGE};
use value::IppValue;
use parser::IppParser;

/// IPP request struct
pub struct IppRequestResponse<'a> {
    /// Operation ID
    header: IppHeader,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload to send after IPP-encoded stream (for example Print-Job operation)
    payload: Option<&'a mut Read>
}

pub trait IppRequestTrait {
    fn header(&self) -> &IppHeader;
}

impl<'a> IppRequestTrait for IppRequestResponse<'a> {
    /// Get header
    fn header(&self) -> &IppHeader {
        &self.header
    }
}

impl<'a> IppRequestResponse<'a> {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: Operation, uri: &str) -> IppRequestResponse<'a> {

        let hdr = IppHeader::new(IPP_VERSION, operation as u16, 1);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributeList::new(),
            payload: None };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET,
                              IppValue::Charset("utf-8".to_string())));
        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE,
                              IppValue::NaturalLanguage("en".to_string())));

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(PRINTER_URI,
                              IppValue::Uri(uri.replace("http", "ipp").to_string())));

        retval

    }

    pub fn new_response(status: u16, id: u32) -> IppRequestResponse<'a> {
        let hdr = IppHeader::new(IPP_VERSION, status, id);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributeList::new(),
            payload: None };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET,
                              IppValue::Charset("utf-8".to_string())));
        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE,
                              IppValue::NaturalLanguage("en".to_string())));

        retval
    }

    /// Create IppRequestResponse from the parser
    pub fn from_parser<'b>(parser: &mut IppParser) -> Result<IppRequestResponse<'b>> {
        let res = parser.parse()?;

        Ok(IppRequestResponse {
            header: res.header().clone(),
            attributes: res.attributes().clone(),
            payload: None,
        })
    }

    /// Get header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get attributes
    pub fn attributes(&self) -> &IppAttributeList {
        &self.attributes
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: &'a mut Read) {
        self.payload = Some(payload)
    }

    pub fn set_attribute(&mut self, group: DelimiterTag, attribute: IppAttribute) {
        /// Set attribute
        self.attributes.add(group, attribute);
    }

    /// Serialize request into the binary stream (TCP)
    pub fn write(&'a mut self, writer: &mut Write) -> Result<usize> {
        let mut retval = self.header.write(writer)?;

        retval += self.attributes.write(writer)?;

        debug!("Wrote {} bytes IPP stream", retval);

        if let Some(ref mut payload) = self.payload {
            let size = io::copy(payload, writer)? as usize;
            debug!("Wrote {} bytes payload", size);
            retval += size;
        }

        Ok(retval)
    }
}
