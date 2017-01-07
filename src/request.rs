//!
//! IPP request
//!
use std::io::{self, Read, Write};

use attribute::{IppAttribute, IppAttributeList};
use ::{Result, IPP_VERSION, IppHeader};

/// IPP request struct
pub struct IppRequest<'a> {
    /// Operation ID
    operation: u16,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload to send after IPP-encoded stream (for example Print-Job operation)
    payload: Option<&'a mut Read>
}

impl<'a> IppRequest<'a> {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: u16) -> IppRequest<'a> {
        IppRequest {
            operation: operation,
            attributes: IppAttributeList::new(),
            payload: None }
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
        let hdr = IppHeader::new(IPP_VERSION, self.operation, 1);
        let mut retval = hdr.write(writer)?;

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
