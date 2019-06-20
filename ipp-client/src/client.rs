//!
//! IPP client
//!
use std::{
    fs,
    io::{BufReader, Cursor},
    mem,
    path::PathBuf,
    time::Duration,
};

use futures::{future::IntoFuture, Future, Stream};
use log::debug;
use num_traits::FromPrimitive;
use reqwest::{
    r#async::{Client, Decoder},
    Certificate,
};
use url::Url;

use crate::IppError;
use ipp_proto::{
    attribute::{PRINTER_STATE, PRINTER_STATE_REASONS},
    ipp::{self, DelimiterTag, PrinterState},
    operation::IppOperation,
    request::IppRequestResponse,
    IppAttributes, IppOperationBuilder, IppParser, IppValue,
};

const ERROR_STATES: &[&str] = &[
    "media-jam",
    "toner-empty",
    "spool-area-full",
    "cover-open",
    "door-open",
    "input-tray-missing",
    "output-tray-missing",
    "marker-supply-empty",
    "paused",
    "shutdown",
];

fn parse_uri(uri: String) -> impl Future<Item = Url, Error = IppError> {
    futures::lazy(move || match Url::parse(&uri) {
        Ok(mut url) => {
            match url.scheme() {
                "ipp" => {
                    url.set_scheme("http").unwrap();
                    if url.port().is_none() {
                        url.set_port(Some(631)).unwrap();
                    }
                }
                "ipps" => {
                    url.set_scheme("https").unwrap();
                    if url.port().is_none() {
                        url.set_port(Some(443)).unwrap();
                    }
                }
                _ => {}
            }
            Ok(url)
        }
        Err(e) => Err(IppError::ParamError(e.to_string())),
    })
}

fn parse_certs(certs: Vec<PathBuf>) -> impl Future<Item = Vec<Certificate>, Error = IppError> {
    futures::lazy(move || {
        let mut result = Vec::new();

        for cert_file in certs {
            let buf = match fs::read(&cert_file) {
                Ok(buf) => buf,
                Err(e) => return Err(IppError::from(e)),
            };
            let ca_cert = match Certificate::from_der(&buf).or_else(|_| Certificate::from_pem(&buf)) {
                Ok(ca_cert) => ca_cert,
                Err(e) => return Err(IppError::from(e)),
            };
            result.push(ca_cert);
        }
        Ok(result)
    })
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: String,
    pub(crate) ca_certs: Vec<PathBuf>,
    pub(crate) verify_hostname: bool,
    pub(crate) verify_certificate: bool,
    pub(crate) timeout: u64,
}

impl IppClient {
    /// Check printer ready status
    pub fn check_ready(&self) -> impl Future<Item = (), Error = IppError> {
        debug!("Checking printer status");
        let operation = IppOperationBuilder::get_printer_attributes()
            .attributes(&[PRINTER_STATE, PRINTER_STATE_REASONS])
            .build();

        self.send(operation).and_then(|attrs| {
            if let Some(a) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE) {
                if let IppValue::Enum(ref e) = *a.value() {
                    if let Some(state) = PrinterState::from_i32(*e) {
                        if state == PrinterState::Stopped {
                            debug!("Printer is stopped");
                            return Err(IppError::PrinterStopped);
                        }
                    }
                }
            }

            if let Some(reasons) = attrs.get(DelimiterTag::PrinterAttributes, PRINTER_STATE_REASONS) {
                let keywords = match *reasons.value() {
                    IppValue::ListOf(ref v) => v
                        .iter()
                        .filter_map(|e| {
                            if let IppValue::Keyword(ref k) = *e {
                                Some(k.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    IppValue::Keyword(ref v) => vec![v.clone()],
                    _ => Vec::new(),
                };
                if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                    debug!("Printer is in error state: {:?}", keywords);
                    return Err(IppError::PrinterStateError(keywords.clone()));
                }
            }
            Ok(())
        })
    }

    /// send IPP operation
    pub fn send<T>(&self, operation: T) -> impl Future<Item = IppAttributes, Error = IppError>
    where
        T: IppOperation,
    {
        debug!("Sending IPP operation");
        self.send_request(operation.into_ipp_request(&self.uri))
            .and_then(|resp| {
                if resp.header().operation_status > 2 {
                    // IPP error
                    Err(IppError::StatusError(
                        ipp::StatusCode::from_u16(resp.header().operation_status)
                            .unwrap_or(ipp::StatusCode::ServerErrorInternalError),
                    ))
                } else {
                    Ok(resp.attributes().clone())
                }
            })
    }

    /// Send request and return response
    pub fn send_request(
        &self,
        request: IppRequestResponse,
    ) -> impl Future<Item = IppRequestResponse, Error = IppError> + Send {
        // Some printers don't support gzip
        let mut builder = Client::builder().gzip(false).connect_timeout(Duration::from_secs(10));

        if !self.verify_hostname {
            debug!("Disabling hostname verification!");
            builder = builder.danger_accept_invalid_hostnames(true);
        }

        if !self.verify_certificate {
            debug!("Disabling certificate verification!");
            builder = builder.danger_accept_invalid_certs(true);
        }

        if self.timeout > 0 {
            debug!("Setting timeout to {}", self.timeout);
            builder = builder.timeout(Duration::from_secs(self.timeout));
        }

        let uri = self.uri.clone();
        let ca_certs = self.ca_certs.clone();

        parse_uri(uri).and_then(|url| {
            parse_certs(ca_certs).and_then(|certs| {
                debug!("Request URI: {}", url);

                for ca_cert in certs {
                    builder = builder.add_root_certificate(ca_cert);
                }

                builder
                    .build()
                    .into_future()
                    .and_then(|client| {
                        client
                            .post(url)
                            .header("Content-Type", "application/ipp")
                            .body(request.into_stream())
                            .send()
                    })
                    .and_then(|response| response.error_for_status())
                    .and_then(|mut response| {
                        let body = mem::replace(response.body_mut(), Decoder::empty());
                        body.concat2()
                    })
                    .map_err(IppError::HttpError)
                    .and_then(|body| {
                        let mut reader = BufReader::new(Cursor::new(body));
                        let parser = IppParser::new(&mut reader);
                        IppRequestResponse::from_parser(parser).map_err(IppError::ParseError)
                    })
            })
        })
    }
}
