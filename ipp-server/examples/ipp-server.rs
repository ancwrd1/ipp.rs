#![allow(unused)]

use std::{
    fs::OpenOptions,
    io::{self, Cursor},
    sync::atomic,
    time,
};

use futures::{future::Future, stream::Stream};
use hyper::{service::service_fn, Body, Chunk, Request, Response, Server};
use lazy_static::lazy_static;

use ipp_parse::{attribute::*, ipp::*, IppHeader, IppParser, IppValue};
use ipp_proto::request::{IppRequestResponse, IppRequestTrait};
use ipp_server::server::*;

struct DummyServer {
    name: String,
    start_time: time::SystemTime,
    printing: atomic::AtomicBool,
}

impl DummyServer {
    fn get_printer_attribute(&self, attr: &str) -> IppAttribute {
        match attr {
            PRINTER_NAME => IppAttribute::new(attr, IppValue::NameWithoutLanguage(self.name.clone())),
            PRINTER_INFO => IppAttribute::new(attr, IppValue::TextWithoutLanguage("Project Typemetal".to_string())),
            PRINTER_STATE_MESSAGE => {
                IppAttribute::new(attr, IppValue::TextWithoutLanguage("Being awesome".to_string()))
            }
            PRINTER_MAKE_AND_MODEL => {
                IppAttribute::new(attr, IppValue::TextWithoutLanguage("Rust Printer".to_string()))
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
            PRINTER_UP_TIME => IppAttribute::new(
                attr,
                IppValue::Integer(self.start_time.elapsed().unwrap().as_secs() as i32),
            ),
            PDL_OVERRIDE_SUPPORTED => IppAttribute::new(attr, IppValue::Keyword("not-attempted".to_string())),
            CHARSET_CONFIGURED => IppAttribute::new(attr, IppValue::Charset("utf-8".to_string())),
            DOCUMENT_FORMAT_DEFAULT => IppAttribute::new(attr, IppValue::MimeMediaType("image/pwg-raster".to_string())),
            DOCUMENT_FORMAT_SUPPORTED => {
                let formats = vec![
                    IppValue::MimeMediaType("image/pwg-raster".to_string()),
                    IppValue::MimeMediaType("image/jpeg".to_string()),
                ];
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
            NATURAL_LANGUAGE_CONFIGURED => IppAttribute::new(attr, IppValue::NaturalLanguage("en".to_string())),
            GENERATED_NATURAL_LANGUAGE_SUPPORTED => {
                let langs = vec![IppValue::NaturalLanguage("en".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(langs))
            }
            CHARSET_SUPPORTED => {
                let charsets = vec![IppValue::Charset("utf-8".to_string())];
                IppAttribute::new(attr, IppValue::ListOf(charsets))
            }
            OPERATIONS_SUPPORTED => {
                let operations = vec![
                    IppValue::Enum(Operation::PrintJob as i32),
                    IppValue::Enum(Operation::CreateJob as i32),
                    IppValue::Enum(Operation::CancelJob as i32),
                    IppValue::Enum(Operation::GetJobAttributes as i32),
                    IppValue::Enum(Operation::GetJobs as i32),
                    IppValue::Enum(Operation::GetPrinterAttributes as i32),
                ];
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

struct DummyRequest {
    header: IppHeader,
    attributes: IppAttributes,
    cursor: Cursor<Chunk>,
}

impl IppRequestTrait for DummyRequest {
    fn header(&self) -> &IppHeader {
        &self.header
    }
}

impl IppServer for DummyServer {
    type IppRequest = DummyRequest;

    fn print_job(&mut self, mut req: Self::IppRequest) -> IppServerResult {
        println!("Print-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes);
        println!();
        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16, req.header().request_id);

        resp.set_attribute(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_URI, IppValue::Uri("ipp://192.168.1.217/jobs/foo".to_string())),
        );
        resp.set_attribute(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_ID, IppValue::Integer(1)),
        );
        resp.set_attribute(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_STATE, IppValue::Enum(JobState::Processing as i32)),
        );
        resp.set_attribute(
            DelimiterTag::JobAttributes,
            IppAttribute::new(
                JOB_STATE_REASONS,
                IppValue::Keyword("completed-successfully".to_string()),
            ),
        );

        self.printing.store(true, atomic::Ordering::Relaxed);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("printjob.dat")
            .unwrap();
        io::copy(&mut req.cursor, &mut file).unwrap();
        Ok(resp)
    }

    fn validate_job(&mut self, req: Self::IppRequest) -> IppServerResult {
        println!("Validate-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes);
        println!();
        let resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16, req.header().request_id);

        Ok(resp)
    }

    fn create_job(&mut self, req: Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn cancel_job(&mut self, req: Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_job_attributes(&mut self, req: Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_jobs(&mut self, req: Self::IppRequest) -> IppServerResult {
        Err(StatusCode::ServerErrorOperationNotSupported)
    }

    fn get_printer_attributes(&mut self, req: Self::IppRequest) -> IppServerResult {
        lazy_static! {
            static ref SUPPORTED_ATTRIBUTES: Vec<&'static str> = vec![
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

        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16, req.header().request_id);
        let mut requested_attributes: Vec<&str> = vec![];
        if let Some(attr) = req
            .attributes
            .get(DelimiterTag::OperationAttributes, REQUESTED_ATTRIBUTES)
        {
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
                    }
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
                resp.set_attribute(DelimiterTag::PrinterAttributes, self.get_printer_attribute(attr));
            } else {
                println!("Unsupported attribute {}", attr);
            }
        }

        Ok(resp)
    }
}

fn main() {
    let addr = ([0, 0, 0, 0], 8631).into();
    let server = Server::bind(&addr).serve(|| {
        service_fn(|req: Request<Body>| {
            let mut server = DummyServer {
                name: "foobar".to_string(),
                start_time: time::SystemTime::now(),
                printing: atomic::AtomicBool::new(false),
            };
            req.into_body().concat2().map(move |c| {
                let mut cursor = Cursor::new(c);
                let ippreq = {
                    let mut parser = IppParser::new(&mut cursor);
                    IppRequestResponse::from_parser(parser).unwrap()
                };
                let dummy_req = DummyRequest {
                    header: ippreq.header().clone(),
                    attributes: ippreq.attributes().clone(),
                    cursor,
                };

                let mut ippresp = match server.handle_request(dummy_req) {
                    Ok(response) => response,
                    Err(ipp_error) => IppRequestResponse::new_response(ipp_error as u16, ippreq.header().request_id),
                };
                let mut buf: Vec<u8> = vec![];
                let _ = ippresp.write(&mut buf);
                Response::new(Body::from(buf))
            })
        })
    });

    hyper::rt::run(server.map_err(|e| {
        eprintln!("server error: {}", e);
    }));
}
