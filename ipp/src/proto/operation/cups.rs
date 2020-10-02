//!
//! CUPS-specific IPP operations. For operations which require user authentication the URI may include authority part.
//!

use http::Uri;

use crate::proto::{model::Operation, operation::IppOperation, request::IppRequestResponse};

/// IPP operation CUPS-Get-Printers
#[derive(Default)]
pub struct CupsGetPrinters;

impl CupsGetPrinters {
    /// Create CUPS-Get-Printers operation
    pub fn new() -> CupsGetPrinters {
        CupsGetPrinters::default()
    }
}

impl IppOperation for CupsGetPrinters {
    fn into_ipp_request(self, _uri: Uri) -> IppRequestResponse {
        IppRequestResponse::new(self.version(), Operation::CupsGetPrinters, None)
    }
}

/// IPP operation CUPS-Delete-Printer
#[derive(Default)]
pub struct CupsDeletePrinter;

impl CupsDeletePrinter {
    /// Create CUPS-Get-Printers operation
    pub fn new() -> CupsDeletePrinter {
        CupsDeletePrinter::default()
    }
}

impl IppOperation for CupsDeletePrinter {
    fn into_ipp_request(self, uri: Uri) -> IppRequestResponse {
        IppRequestResponse::new(self.version(), Operation::CupsDeletePrinter, Some(uri))
    }
}
