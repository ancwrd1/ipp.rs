#![allow(unused)]

use std::{
    env,
    fs::OpenOptions,
    io::{self, Cursor},
    mem,
    path::PathBuf,
    sync::{atomic, Arc},
    time,
};

use futures::{future::Future, stream::Stream};
use hyper::{service::service_fn, Body, Chunk, Request, Response, Server};
use log::debug;
use tempfile::NamedTempFile;

use ipp_proto::{
    attribute::*,
    ipp::*,
    request::{IppRequestResponse, PayloadKind},
    AsyncIppParser, IppHeader, IppParser, IppValue,
};
use ipp_server::{handler::*, server::IppServerBuilder};
use lazy_static::lazy_static;

struct TestServer {
    name: String,
    start_time: time::SystemTime,
    printing: atomic::AtomicBool,
    spooler_dir: PathBuf,
}

impl TestServer {
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

impl IppRequestHandler for TestServer {
    fn print_job(&self, mut req: IppRequestResponse) -> IppServerResult {
        println!("Print-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes());

        match req.payload_mut().take() {
            Some(PayloadKind::TempFile(file)) => {
                let new_path = self.spooler_dir.join(format!(
                    "{}.spl",
                    self.start_time
                        .duration_since(time::SystemTime::UNIX_EPOCH)
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

        let mut resp =
            IppRequestResponse::new_response(self.version(), StatusCode::SuccessfulOK, req.header().request_id);

        resp.attributes_mut().add(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_URI, IppValue::Uri("ipp://192.168.1.217/jobs/foo".to_string())),
        );
        resp.attributes_mut().add(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_ID, IppValue::Integer(1)),
        );
        resp.attributes_mut().add(
            DelimiterTag::JobAttributes,
            IppAttribute::new(JOB_STATE, IppValue::Enum(JobState::Processing as i32)),
        );
        resp.attributes_mut().add(
            DelimiterTag::JobAttributes,
            IppAttribute::new(
                JOB_STATE_REASONS,
                IppValue::Keyword("completed-successfully".to_string()),
            ),
        );

        self.printing.store(true, atomic::Ordering::Relaxed);

        Ok(resp)
    }

    fn validate_job(&self, req: IppRequestResponse) -> IppServerResult {
        println!("Validate-Job");
        println!("{:?}", req.header());
        println!("{:?}", req.attributes());

        let resp = IppRequestResponse::new_response(self.version(), StatusCode::SuccessfulOK, req.header().request_id);

        Ok(resp)
    }

    fn get_printer_attributes(&self, req: IppRequestResponse) -> IppServerResult {
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

        let mut resp =
            IppRequestResponse::new_response(self.version(), StatusCode::SuccessfulOK, req.header().request_id);

        let mut requested_attributes: Vec<&str> = vec![];

        if let Some(attr) = req
            .attributes()
            .groups_of(DelimiterTag::OperationAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(REQUESTED_ATTRIBUTES))
        {
            match *attr.value() {
                IppValue::Keyword(ref k) => {
                    requested_attributes.push(k);
                }
                IppValue::ListOf(ref attrs) => {
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
            requested_attributes.as_slice()
        };

        for attr in attribute_list {
            if SUPPORTED_ATTRIBUTES.contains(attr) {
                resp.attributes_mut()
                    .add(DelimiterTag::PrinterAttributes, self.get_printer_attribute(attr));
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

    let _ = std::fs::create_dir_all(&args[1]);

    let test_server = TestServer {
        name: "IPP server example".to_string(),
        start_time: time::SystemTime::now(),
        printing: atomic::AtomicBool::new(false),
        spooler_dir: env::args().nth(1).unwrap().into(),
    };

    let fut = IppServerBuilder::new(([0, 0, 0, 0], 7631))
        .handler(Arc::new(test_server))
        .build()
        .map_err(|e| {
            eprintln!("ERROR: {:?}", e);
        });

    hyper::rt::run(fut);
}
