// IPP server
use enum_primitive::FromPrimitive;

use request::IppRequestResponse;
use consts::statuscode::StatusCode;
use consts::operation::Operation;

pub type IppServerResult<'a> = Result<IppRequestResponse<'a>, StatusCode>;

pub trait IppServer {
    fn print_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn print_uri<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn validate_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn create_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn send_document<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn send_uri<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn cancel_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn get_job_attributes<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn get_jobs<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn get_printer_attributes<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a>;
    fn hold_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn release_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn restart_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn pause_printer<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn resume_printer<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }
    fn purge_jobs<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn ipp_handle_request<'a>(&self, req: &IppRequestResponse) -> IppServerResult<'a> {
        if req.header().version != 0x0101 {
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
