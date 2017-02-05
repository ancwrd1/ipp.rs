//!
//! IPP response
//!

use ::{Result, IppHeader};
use parser::IppParser;
use attribute::{IppAttributeList};

/// IPP response is returned by `IppClient`
pub struct IppResponse {
    /// Response header
    header: IppHeader,
    /// Response attributes
    attributes: IppAttributeList
}

impl IppResponse {
    /// Create IppResponse from the parser
    pub fn from_parser(parser: &mut IppParser) -> Result<IppResponse> {
        let res = parser.parse()?;

        Ok(IppResponse { header: res.header().clone(), attributes: res.attributes().clone() })
    }

    /// Get header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get attributes
    pub fn attributes(&self) -> &IppAttributeList {
        &self.attributes
    }
}
