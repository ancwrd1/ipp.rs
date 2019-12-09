//!
//! CUPS-specific IPP operations
//!

use crate::proto::{ipp::Operation, operation::IppOperation, request::IppRequestResponse};

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
    fn into_ipp_request(self, _uri: &str) -> IppRequestResponse {
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
    fn into_ipp_request(self, uri: &str) -> IppRequestResponse {
        IppRequestResponse::new(self.version(), Operation::CupsDeletePrinter, Some(uri))
    }
}
