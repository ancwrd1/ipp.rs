//!
//! IPP request
//!
use std::io::{self, Read, Write};

use log::debug;

use ippparse::attribute::{
    IppAttribute, IppAttributeList, ATTRIBUTES_CHARSET, ATTRIBUTES_NATURAL_LANGUAGE, PRINTER_URI,
};
use ippparse::ipp::{DelimiterTag, Operation};
use ippparse::parser::IppParser;
use ippparse::value::IppValue;
use ippparse::{IppHeader, IppVersion};

/// IPP request/response struct
pub struct IppRequestResponse {
    /// IPP header
    header: IppHeader,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload after IPP-encoded stream (for example binary data for Print-Job operation)
    payload: Option<Box<Read>>,
}

/// Helper class to combine IPP data and payload
pub struct IppReadAdapter {
    data: Box<Read>,
    payload: Option<Box<Read>>,
}

unsafe impl Send for IppReadAdapter {}

impl Read for IppReadAdapter {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.data.read(buf)? {
            0 => if let Some(ref mut payload) = self.payload {
                payload.read(buf)
            } else {
                Ok(0)
            },
            rc => Ok(rc),
        }
    }
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
            attributes: IppAttributeList::new(),
            payload: None,
        };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                PRINTER_URI,
                IppValue::Uri(uri.replace("http", "ipp").to_string()),
            ),
        );

        retval
    }

    /// Create response from status and id
    pub fn new_response(status: u16, id: u32) -> IppRequestResponse {
        let hdr = IppHeader::new(IppVersion::Ipp11, status, id);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributeList::new(),
            payload: None,
        };

        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.set_attribute(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        retval
    }

    /// Create IppRequestResponse from the parser
    pub fn from_parser(parser: &mut IppParser) -> io::Result<IppRequestResponse> {
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
    pub fn attributes(&self) -> &IppAttributeList {
        &self.attributes
    }

    /// Get payload
    pub fn payload(&self) -> &Option<Box<Read>> {
        &self.payload
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: Box<Read>) {
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

        if let Some(ref mut payload) = self.payload {
            let size = io::copy(payload, writer)? as usize;
            debug!("Wrote {} bytes payload", size);
            retval += size;
        }

        Ok(retval)
    }

    /// Convert request into reader
    pub fn into_reader(self) -> impl Read {
        IppReadAdapter {
            data: Box::new(
                self.header
                    .into_reader()
                    .chain(self.attributes.into_reader()),
            ),
            payload: self.payload,
        }
    }
}
