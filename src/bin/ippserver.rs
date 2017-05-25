extern crate hyper;
extern crate ipp;
extern crate enum_primitive;

use hyper::server::{Server, Request, Response, Handler};
use ipp::parser::IppParser;
use ipp::server::*;
use ipp::IppRequestResponse;
use ipp::consts::statuscode::*;
use ipp::consts::tag::*;
use ipp::consts::attribute::*;
use ipp::attribute::IppAttribute;
use ipp::value::IppValue;

struct DummyServer {
    name: String,
}

impl IppServer for DummyServer {
    fn print_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn create_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn cancel_job<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_job_attributes<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_jobs<'a>(&self, _req: &IppRequestResponse) -> IppServerResult<'a> {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_printer_attributes<'a>(&self, req: &IppRequestResponse) -> IppServerResult<'a> {
        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16,
                                                        req.header().request_id);
        resp.set_attribute(DelimiterTag::PrinterAttributes,
                           IppAttribute::new(PRINTER_NAME,
                                             IppValue::NameWithoutLanguage(self.name.clone())));
        Ok(resp)
    }
}

impl Handler for DummyServer {
    fn handle(&self, mut req: Request, res: Response) {
        let mut parser = IppParser::new(&mut req);
        let ippreq = IppRequestResponse::from_parser(&mut parser).unwrap();
        println!("{:?}", ippreq.header());
        println!("{:?}", ippreq.attributes());

        match self.ipp_handle_request(&ippreq) {
            Ok(mut response) => {
                let mut res_streaming = res.start().unwrap();
                response.write(&mut res_streaming).expect("Failed to write response");
            }
            Err(ipp_error) => {
                let mut resp = IppRequestResponse::new_response(ipp_error as u16,
                                                                ippreq.header().request_id);
                let mut res_streaming = res.start().unwrap();
                resp.write(&mut res_streaming).expect("Failed to write response");
            }
        }
    }
}

fn main() {
    let server = DummyServer { name: "foobar".to_string() };
    Server::http("0.0.0.0:8080").unwrap().handle(server).unwrap();
}
