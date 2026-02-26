//!
//! CUPS-specific IPP operations. For operations which require user authentication the URI may include authority part.
//!

use http::Uri;

use crate::{
    model::Operation, operation::IppOperation, parser::IppParseError, request::IppRequestResponse, value::IppString,
};

/// IPP operation CUPS-Get-Printers
#[derive(Default)]
pub struct CupsGetPrinters;

impl CupsGetPrinters {
    /// Create CUPS-Get-Printers operation
    pub fn new() -> CupsGetPrinters {
        CupsGetPrinters
    }
}

impl IppOperation for CupsGetPrinters {
    fn into_ipp_request(self) -> IppRequestResponse {
        IppRequestResponse::new(self.version(), Operation::CupsGetPrinters, None)
            .expect("cups list printers URI length check missing")
    }
}

/// IPP operation CUPS-Delete-Printer
pub struct CupsDeletePrinter(IppString);

impl CupsDeletePrinter {
    /// Create CUPS-Get-Printers operation
    pub fn new(printer_uri: Uri) -> Result<CupsDeletePrinter, IppParseError> {
        Ok(CupsDeletePrinter(printer_uri.try_into()?))
    }
}

impl IppOperation for CupsDeletePrinter {
    fn into_ipp_request(self) -> IppRequestResponse {
        IppRequestResponse::new_internal(self.version(), Operation::CupsDeletePrinter, Some(self.0))
    }
}
