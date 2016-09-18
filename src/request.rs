//!
//! IPP request
//!
use std::io::{self, Read, Write};
use byteorder::{BigEndian, WriteBytesExt};

use attribute::{IppAttribute, IppAttributeList};
use ::{Result, IPP_VERSION};
use consts::tag::*;
use consts::attribute::*;
use value::IppValue;

/// IPP request struct
pub struct IppRequest<'a> {
    /// Operation ID
    operation: u16,
    /// IPP server URI
    uri: String,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload to send after IPP-encoded stream (for example Print-Job operation)
    payload: Option<&'a mut Read>
}

impl<'a> IppRequest<'a> {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: u16, uri: &str) -> IppRequest<'a> {
        let mut retval = IppRequest {
            operation: operation,
            uri: uri.to_string(),
            attributes: IppAttributeList::new(),
            payload: None };

        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(ATTRIBUTES_CHARSET,
                              IppValue::Charset("utf-8".to_string())));
        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE,
                              IppValue::NaturalLanguage("en".to_string())));

        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(PRINTER_URI,
                              IppValue::Uri(uri.replace("http", "ipp").to_string())));
        retval
    }

    /// Get uri
    pub fn uri(&self) -> &String {
        &self.uri
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: &'a mut Read) {
        self.payload = Some(payload)
    }

    /// Set attribute
    pub fn set_attribute(&mut self, group: u8, attribute: IppAttribute) {
        self.attributes.add(group, attribute);
    }

    /// Serialize request into the binary stream (TCP)
    pub fn write(&'a mut self, writer: &mut Write) -> Result<usize> {
        let mut retval = 0;
        try!(writer.write_u16::<BigEndian>(IPP_VERSION));
        retval += 2;

        try!(writer.write_u16::<BigEndian>(self.operation));
        retval += 2;

        // request id
        try!(writer.write_u32::<BigEndian>(1));
        retval += 4;

        retval += try!(self.attributes.write(writer));

        debug!("Wrote {} bytes IPP stream", retval);

        match self.payload {
            Some(ref mut payload) => {
                let size = try!(io::copy(payload, writer)) as usize;
                debug!("Wrote {} bytes payload", size);
                retval += size;
            }
            None => {}
        }

        Ok(retval)
    }
}
