//!
//! IPP request
//!
use std::io::{self, Read, Write};

use attribute::{IppAttribute, IppAttributeList};
use ::{Result, IPP_VERSION, IppHeader};
use consts::tag::OPERATION_ATTRIBUTES_TAG;
use consts::attribute::{PRINTER_URI, ATTRIBUTES_CHARSET, ATTRIBUTES_NATURAL_LANGUAGE};
use value::IppValue;

/// IPP request struct
pub struct IppRequest<'a> {
    /// Operation ID
    header: IppHeader,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload to send after IPP-encoded stream (for example Print-Job operation)
    payload: Option<&'a mut Read>
}

impl<'a> IppRequest<'a> {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: u16, uri: &str) -> IppRequest<'a> {

        let hdr = IppHeader::new(IPP_VERSION, operation, 1);
        let mut retval = IppRequest {
            header: hdr,
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

    /// Set payload
    pub fn set_payload(&mut self, payload: &'a mut Read) {
        self.payload = Some(payload)
    }

    pub fn set_attribute(&mut self, group: u8, attribute: IppAttribute) {
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
