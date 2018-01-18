// IPP server
use num_traits::FromPrimitive;

use request::{IppRequestResponse,IppRequestTrait};
use consts::statuscode::StatusCode;
use consts::operation::Operation;

pub type IppServerResult = Result<IppRequestResponse, StatusCode>;

pub trait IppServer<'b, 'c> {
    type IppRequest : IppRequestTrait;

    fn print_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn print_uri(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn validate_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn create_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn send_document(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn send_uri(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn cancel_job(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn get_job_attributes(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn get_jobs(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn get_printer_attributes(&self, _req: &mut Self::IppRequest) -> IppServerResult;
    fn hold_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn release_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn restart_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn pause_printer(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn resume_printer(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn purge_jobs(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_version(&self) -> u16 {
        0x0101
    }

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
