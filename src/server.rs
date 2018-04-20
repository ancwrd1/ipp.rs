//!
//! Basic definitions for IPP server implementation
//!

use num_traits::FromPrimitive;

use request::{IppRequestResponse,IppRequestTrait};
use consts::statuscode::StatusCode;
use consts::operation::Operation;

pub type IppServerResult = Result<IppRequestResponse, StatusCode>;

/// A trait which defines IPP operations
pub trait IppServer<'b, 'c> {
    type IppRequest : IppRequestTrait;

    /// Print-Job operation
    fn print_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Print-Uri operation
    fn print_uri(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Validate-Job operation
    fn validate_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Cceate-Job operation
    fn create_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Send-Document operation
    fn send_document(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Send-Uri operation
    fn send_uri(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Cancel-Job operation
    fn cancel_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Get-Job-Attributes operation
    fn get_job_attributes(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Get-Jobs operation
    fn get_jobs(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Get-Printer-Attributes operation
    fn get_printer_attributes(&self, _req: &mut Self::IppRequest) -> IppServerResult;

    /// Hold-Job operation
    fn hold_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Release-Job operation
    fn release_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Restart-Job operation
    fn restart_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Pause-Printer operation
    fn pause_printer(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Resume-Printer operation
    fn resume_printer(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Purge-Jobs operation
    fn purge_jobs(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    /// Returns IPP version supported by the server
    fn get_version(&self) -> u16 {
        0x0101
    }

    /// IPP request dispatcher
    fn ipp_handle_request(&self, req: &mut Self::IppRequest) -> IppServerResult {
        if req.header().version != self.get_version() {
            return Err(StatusCode::ServerErrorVersionNotSupported);
        }
        let operation = Operation::from_u16(req.header().operation_status)
            .ok_or(StatusCode::ServerErrorOperationNotSupported)?;

        match operation {
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
        }
    }
}
