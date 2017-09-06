extern crate hyper;
extern crate ipp;
#[macro_use]
extern crate lazy_static;

use std::time;
use std::fs::OpenOptions;
use std::sync::atomic;
use std::io;

use hyper::server::{Server, Request, Response, Handler};
use ipp::parser::IppParser;
use ipp::server::*;
use ipp::{IppRequestResponse,IppHeader};
use ipp::request::IppRequestTrait;
use ipp::consts::statuscode::*;
use ipp::consts::tag::*;
use ipp::consts::attribute::*;
use ipp::consts::operation::Operation;
use ipp::attribute::{IppAttribute,IppAttributeList};
use ipp::value::IppValue;

struct DummyServer {
    name: String,
    start_time: time::SystemTime,
    printing: atomic::AtomicBool,
}

impl DummyServer {
    fn get_printer_attribute(&self, attr: &str) -> IppAttribute {
        match attr {
            PRINTER_NAME => {
                IppAttribute::new(attr, IppValue::NameWithoutLanguage(self.name.clone()))
            }
            PRINTER_INFO => {
                IppAttribute::new(attr,
                                  IppValue::TextWithoutLanguage("Project Typemetal".to_string()))
            }
            PRINTER_STATE_MESSAGE => {
                IppAttribute::new(attr,
                                  IppValue::TextWithoutLanguage("Being awesome".to_string()))
            }
            PRINTER_MAKE_AND_MODEL => {
                IppAttribute::new(attr,
                                  IppValue::TextWithoutLanguage("Rust Printer".to_string()))
            }
            IPP_VERSIONS_SUPPORTED => {
                let versions = vec![IppValue::Keyword("1.1".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(versions))
            }
            PRINTER_STATE => {
                let state = if self.printing.load(atomic::Ordering::Relaxed) {
                    self.printing.store(false, atomic::Ordering::Relaxed);
                    IppValue::Enum(PrinterState::Processing as i32)
                } else {
                    IppValue::Enum(PrinterState::Idle as i32)
                };
                IppAttribute::new(attr, state)
            }
            PRINTER_IS_ACCEPTING_JOBS => IppAttribute::new(attr, IppValue::Boolean(true)),
            FINISHINGS_DEFAULT => IppAttribute::new(attr, IppValue::Enum(Finishings::None as i32)),
            FINISHINGS_SUPPORTED => {
                let finishings = vec![IppValue::Enum(Finishings::None as i32)];
                IppAttribute::new(attr, IppValue::ListOf(finishings))
            }
            QUEUED_JOB_COUNT => IppAttribute::new(attr, IppValue::Integer(0)),
            PRINTER_UP_TIME => {
                IppAttribute::new(attr,
                                  IppValue::Integer(self.start_time.elapsed().unwrap().as_secs() as
                                                    i32))
            }
            PDL_OVERRIDE_SUPPORTED => {
                IppAttribute::new(attr, IppValue::Keyword("not-attempted".to_string()))
            }
            CHARSET_CONFIGURED => IppAttribute::new(attr, IppValue::Charset("utf-8".to_string())),
            DOCUMENT_FORMAT_DEFAULT => {
                IppAttribute::new(attr,
                                  IppValue::MimeMediaType("image/pwg-raster".to_string()))
            }
            DOCUMENT_FORMAT_SUPPORTED => {
                let formats = vec![IppValue::MimeMediaType("image/pwg-raster".to_string()),
                                   IppValue::MimeMediaType("image/jpeg".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(formats))
            }
            COMPRESSION_SUPPORTED => {
                let compressions = vec![IppValue::Keyword("none".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(compressions))
            }
            URI_AUTHENTICATION_SUPPORTED => {
                let auths = vec![IppValue::Keyword("none".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(auths))
            }
            NATURAL_LANGUAGE_CONFIGURED => {
                IppAttribute::new(attr, IppValue::NaturalLanguage("en".to_string()))
            }
            GENERATED_NATURAL_LANGUAGE_SUPPORTED => {
                let langs = vec![IppValue::NaturalLanguage("en".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(langs))
            }
            CHARSET_SUPPORTED => {
                let charsets = vec![IppValue::Charset("utf-8".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(charsets))
            }
            OPERATIONS_SUPPORTED => {
                let operations = vec![IppValue::Enum(Operation::PrintJob as i32),
                                      IppValue::Enum(Operation::CreateJob as i32),
                                      IppValue::Enum(Operation::CancelJob as i32),
                                      IppValue::Enum(Operation::GetJobAttributes as i32),
                                      IppValue::Enum(Operation::GetJobs as i32),
                                      IppValue::Enum(Operation::GetPrinterAttributes as i32)];
                IppAttribute::new(attr, IppValue::ListOf(operations))
            }
            PRINTER_STATE_REASONS => {
                let reasons = vec![IppValue::Keyword("none".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(reasons))
            }
            PRINTER_URI_SUPPORTED => {
                let uris = vec![IppValue::Uri("ipp://192.168.1.217".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(uris))
            }
            URI_SECURITY_SUPPORTED => {
                let securities = vec![IppValue::Keyword("none".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(securities))
            }
            _ => panic!("Got an unsupported attribute in get_printer_attribute!"),
        }
    }
}

struct DummyRequest<'a, 'b: 'a> {
    header: IppHeader,
    attributes: IppAttributeList,
    req: Request<'a, 'b>,
}

impl<'a, 'b> IppRequestTrait for DummyRequest<'a, 'b> {
    fn header(&self) -> &IppHeader {
        &self.header
    }
}

impl<'b, 'c: 'b> IppServer<'b, 'c> for DummyServer {
    type IppRequest = DummyRequest<'b, 'c>;

    fn print_job(&self, req: &mut Self::IppRequest) -> IppServerResult {
        println!("Print-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes);
        println!("");
        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16,
                                                        req.header().request_id);

        resp.set_attribute(DelimiterTag::JobAttributes,
                           IppAttribute::new(JOB_URI,
                               IppValue::Uri("ipp://192.168.1.217/jobs/foo".to_string())));
        resp.set_attribute(DelimiterTag::JobAttributes,
                           IppAttribute::new(JOB_ID,
                               IppValue::Integer(1)));
        resp.set_attribute(DelimiterTag::JobAttributes,
                           IppAttribute::new(JOB_STATE,
                               IppValue::Enum(JobState::Processing as i32)));
        resp.set_attribute(DelimiterTag::JobAttributes,
                           IppAttribute::new(JOB_STATE_REASONS,
                               IppValue::Keyword("completed-successfully".to_string())));

        self.printing.store(true, atomic::Ordering::Relaxed);
        let mut file = OpenOptions::new().write(true).create(true).open("printjob.dat").unwrap();
        io::copy(&mut req.req, &mut file).unwrap();
        Ok(resp)
    }

    fn validate_job(&self, req: &mut Self::IppRequest) -> IppServerResult {
        println!("Validate-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes);
        println!("");
        let resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16,
                                                        req.header().request_id);

        Ok(resp)
    }

    fn create_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn cancel_job(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_job_attributes(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_jobs(&self, _req: &mut Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_printer_attributes(&self, req: &mut Self::IppRequest) -> IppServerResult {
        lazy_static! {
            static ref SUPPORTED_ATTRIBUTES : Vec<&'static str> = vec![
                PRINTER_URI_SUPPORTED,
                URI_SECURITY_SUPPORTED,
                URI_AUTHENTICATION_SUPPORTED,
                PRINTER_NAME,
                PRINTER_STATE,
                PRINTER_STATE_REASONS,
                IPP_VERSIONS_SUPPORTED,
                OPERATIONS_SUPPORTED,
                CHARSET_CONFIGURED,
                CHARSET_SUPPORTED,
                NATURAL_LANGUAGE_CONFIGURED,
                GENERATED_NATURAL_LANGUAGE_SUPPORTED,
                DOCUMENT_FORMAT_DEFAULT,
                DOCUMENT_FORMAT_SUPPORTED,
                PRINTER_IS_ACCEPTING_JOBS,
                QUEUED_JOB_COUNT,
                PDL_OVERRIDE_SUPPORTED,
                PRINTER_UP_TIME,
                COMPRESSION_SUPPORTED,
                PRINTER_STATE_MESSAGE,
                PRINTER_MAKE_AND_MODEL,
                FINISHINGS_DEFAULT,
                FINISHINGS_SUPPORTED,
            ];
        }
        let supported_attributes = &SUPPORTED_ATTRIBUTES[..];

        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16,
                                                        req.header().request_id);
        let mut requested_attributes : Vec<&str> = vec![];
        if let Some(attr) = req.attributes.get(DelimiterTag::OperationAttributes, REQUESTED_ATTRIBUTES) {
            match *attr.value() {
                IppValue::Keyword(ref x) => {
                    requested_attributes = vec![x];
                }
                IppValue::ListOf(ref attrs) => {
                    requested_attributes = vec![];
                    for i in attrs {
                        if let IppValue::Keyword(ref keyword) = *i {
                            requested_attributes.push(keyword);
                        } else {
                            return Err(StatusCode::ClientErrorBadRequest);
                        }
                    };
                }
                _ => {
                    return Err(StatusCode::ClientErrorBadRequest);
                }
            }
        };

        let attribute_list = if requested_attributes.is_empty() {
            supported_attributes
        } else {
            &requested_attributes[..]
        };

        for attr in attribute_list {
            if supported_attributes.contains(attr) {
                resp.set_attribute(DelimiterTag::PrinterAttributes,
                                   self.get_printer_attribute(attr));
            } else {
                println!("Unsupported attribute {}", attr);
            }
        }

        Ok(resp)
    }
}

impl Handler for DummyServer {
    fn handle(&self, mut req: Request, res: Response) {
        let ippreq = {
            let mut parser = IppParser::new(&mut req);
            IppRequestResponse::from_parser(&mut parser).unwrap()
        };
        let mut dummy_req = DummyRequest {
            header: ippreq.header().clone(),
            attributes: ippreq.attributes().clone(),
            req: req,
        };

        let mut ippresp = match self.ipp_handle_request(&mut dummy_req) {
            Ok(response) => response,
            Err(ipp_error) =>
                IppRequestResponse::new_response(ipp_error as u16,
                                                 ippreq.header().request_id),
        };
        let mut res_streaming = res.start().unwrap();
        ippresp.write(&mut res_streaming).expect("Failed to write response");
    }
}

fn main() {
    let server = DummyServer {
        name: "foobar".to_string(),
        start_time: time::SystemTime::now(),
        printing: atomic::AtomicBool::new(false),
    };
    Server::http("0.0.0.0:631").unwrap().handle(server).unwrap();
}
