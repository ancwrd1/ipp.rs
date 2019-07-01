#![allow(unused)]

use std::{
    env,
    fs::OpenOptions,
    io::{self, Cursor},
    mem,
    path::PathBuf,
    sync::atomic,
    time,
};

use futures::{future::Future, stream::Stream};
use hyper::{service::service_fn, Body, Chunk, Request, Response, Server};
use lazy_static::lazy_static;
use log::debug;
use tempfile::NamedTempFile;

use ipp_proto::{
    attribute::*,
    ipp::*,
    request::{IppRequestResponse, IppRequestTrait, PayloadKind},
    AsyncIppParser, IppHeader, IppParser, IppValue,
};
use ipp_server::server::*;
use std::time::SystemTime;

struct DummyServer {
    name: String,
    start_time: time::SystemTime,
    printing: atomic::AtomicBool,
    spooler_dir: PathBuf,
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
                    IppValue::Enum(Operation::ValidateJob as i32),
                    IppValue::Enum(Operation::GetPrinterAttributes as i32),
                ];
                IppAttribute::new(attr, IppValue::ListOf(operations))
            }
            PRINTER_STATE_REASONS => IppAttribute::new(attr, IppValue::Keyword("none".to_string())),
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
    payload: Option<PayloadKind>,
}

impl IppRequestTrait for DummyRequest {
    fn header(&self) -> &IppHeader {
        &self.header
    }
}

impl IppRequestHandler for DummyServer {
    type IppRequest = DummyRequest;

    fn print_job(&mut self, mut req: Self::IppRequest) -> IppServerResult {
        println!("Print-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes);
        match req.payload.take() {
            Some(PayloadKind::File(file)) => {
                let new_path = self.spooler_dir.join(format!(
                    "{}.spl",
                    self.start_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ));
                match file.persist(&new_path) {
                    Ok(file) => println!("Saved ipp payload to {}", new_path.display()),
                    Err(e) => println!("Error while saving payload: {}", e),
                }
            }
            _ => println!("No payload!"),
        }

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

    fn get_printer_attributes(&mut self, req: Self::IppRequest) -> IppServerResult {
        static SUPPORTED_ATTRIBUTES: &[&'static str] = &[
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

        let mut resp = IppRequestResponse::new_response(StatusCode::SuccessfulOK as u16, req.header().request_id);
        let mut requested_attributes: Vec<&str> = vec![];
        if let Some(attr) = req
            .attributes
            .groups_of(DelimiterTag::OperationAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(REQUESTED_ATTRIBUTES))
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
            &SUPPORTED_ATTRIBUTES
        } else {
            &requested_attributes[..]
        };

        for attr in attribute_list {
            if SUPPORTED_ATTRIBUTES.contains(attr) {
                resp.set_attribute(DelimiterTag::PrinterAttributes, self.get_printer_attribute(attr));
            } else {
                println!("Unsupported attribute {}", attr);
            }
        }

        Ok(resp)
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} spooler_dir", args[0]);
        std::process::exit(1);
    }

    env_logger::init();

    let addr = ([0, 0, 0, 0], 7631).into();
    let server = Server::bind(&addr).serve(|| {
        service_fn(|mut req: Request<Body>| {
            let mut server = DummyServer {
                name: "foobar".to_string(),
                start_time: time::SystemTime::now(),
                printing: atomic::AtomicBool::new(false),
                spooler_dir: env::args().nth(1).unwrap().into(),
            };

            let body = mem::replace(req.body_mut(), Body::empty());

            let stream: Box<dyn Stream<Item = Chunk, Error = io::Error> + Send> =
                Box::new(body.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())));

            AsyncIppParser::from(stream).map(move |result| {
                debug!("Received response, payload: {}", result.payload.is_some());
                let mut ippreq = IppRequestResponse::from_parse_result(result);

                let dummy_req = DummyRequest {
                    header: ippreq.header().clone(),
                    attributes: ippreq.attributes().clone(),
                    payload: ippreq.payload_mut().take(),
                };

                let mut ippresp = match server.handle_request(dummy_req) {
                    Ok(response) => response,
                    Err(ipp_error) => IppRequestResponse::new_response(ipp_error as u16, ippreq.header().request_id),
                };
                Response::new(Body::wrap_stream(ippresp.into_stream()))
            })
        })
    });

    hyper::rt::run(server.map_err(|e| {
        eprintln!("server error: {}", e);
    }));
}
