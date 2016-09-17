//!
//! IPP response
//!

use ::{Result};
use parser::{IppHeader, IppParser};
use attribute::{IppAttributeList};

/// IPP response is returned by IppClient
pub struct IppResponse {
    /// Response header
    header: IppHeader,
    /// Response attributes
    attributes: IppAttributeList
}

impl IppResponse {
    /// Create IppResponse from the parser
    pub fn from_parser(parser: &mut IppParser) -> Result<IppResponse> {
        let res = try!(parser.parse());

        Ok(IppResponse { header: res.header().clone(), attributes: res.attributes().clone() })
    }

    /// Get header
    pub fn header<'a>(&'a self) -> &'a IppHeader {
        &self.header
    }

    /// Get attributes
    pub fn attributes<'a>(&'a self) -> &'a IppAttributeList {
        &self.attributes
    }
}
