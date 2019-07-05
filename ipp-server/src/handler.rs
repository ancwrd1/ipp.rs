//!
//! Basic definitions for IPP server implementation
//!
#![allow(unused)]

use num_traits::FromPrimitive;

use ipp_proto::{
    ipp::{Operation, StatusCode},
    request::IppRequestResponse,
    IppVersion,
};

pub type IppServerResult = Result<IppRequestResponse, StatusCode>;

/// A trait which defines IPP operations
pub trait IppRequestHandler {
    /// Print-Job operation
    fn print_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Print-Uri operation
    fn print_uri(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Validate-Job operation
    fn validate_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Cceate-Job operation
    fn create_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Send-Document operation
    fn send_document(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Send-Uri operation
    fn send_uri(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Cancel-Job operation
    fn cancel_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Get-Job-Attributes operation
    fn get_job_attributes(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Get-Jobs operation
    fn get_jobs(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Get-Printer-Attributes operation
    fn get_printer_attributes(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Hold-Job operation
    fn hold_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Release-Job operation
    fn release_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Restart-Job operation
    fn restart_job(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Pause-Printer operation
    fn pause_printer(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Resume-Printer operation
    fn resume_printer(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Purge-Jobs operation
    fn purge_jobs(&self, req: IppRequestResponse) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Returns IPP version supported by the server
    fn version(&self) -> IppVersion {
        IppVersion::Ipp11
    }

    /// IPP request dispatcher
    fn handle_request(&self, req: IppRequestResponse) -> IppServerResult {
        match req.header().operation()? {
            Operation::PrintJob => self.print_job(req),
            Operation::PrintUri => self.print_uri(req),
            Operation::ValidateJob => self.validate_job(req),
            Operation::CreateJob => self.create_job(req),
            Operation::SendDocument => self.send_document(req),
            Operation::SendUri => self.send_uri(req),
            Operation::CancelJob => self.cancel_job(req),
            Operation::GetJobAttributes => self.get_job_attributes(req),
            Operation::GetJobs => self.get_jobs(req),
            Operation::GetPrinterAttributes => self.get_printer_attributes(req),
            Operation::HoldJob => self.hold_job(req),
            Operation::ReleaseJob => self.release_job(req),
            Operation::RestartJob => self.restart_job(req),
            Operation::PausePrinter => self.pause_printer(req),
            Operation::ResumePrinter => self.resume_printer(req),
            Operation::PurgeJobs => self.purge_jobs(req),
            _ => Err(StatusCode::ServerErrorOperationNotSupported),
        }
    }
}
