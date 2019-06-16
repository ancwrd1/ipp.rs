//!
//! IPP client
//!
use std::{
    fs,
    io::BufReader,
    mem,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::{future::IntoFuture, Future, Stream};
use log::debug;
use num_traits::FromPrimitive;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    r#async::{Client, Decoder},
    Certificate,
};
use url::Url;

use crate::IppError;
use ipp_proto::{ipp, operation::IppOperation, request::IppRequestResponse, IppAttributes, IppParser};
use std::io::Cursor;

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    uri: String,
    cacerts: Vec<PathBuf>,
    verify_hostname: bool,
    verify_certificate: bool,
    timeout: u64,
}

impl IppClient {
    /// Create new instance of the client
    pub fn new(uri: &str) -> IppClient {
        IppClient {
            uri: uri.to_string(),
            cacerts: Vec::new(),
            verify_hostname: true,
            verify_certificate: true,
            timeout: 30,
        }
    }

    /// Enable or disable host name validation for SSL transport. By default it is enabled.
    pub fn set_verify_hostname(&mut self, verify: bool) {
        self.verify_hostname = verify;
    }

    /// Enable or disable server certificate validation for SSL transport. By default it is enabled.
    pub fn set_verify_certificate(&mut self, verify: bool) {
        self.verify_certificate = verify;
    }

    /// Add CA certificate
    pub fn add_root_certificate<P>(&mut self, cacert: P)
    where
        P: AsRef<Path>,
    {
        self.cacerts.push(cacert.as_ref().to_owned());
    }

    /// Set communication timeout in seconds
    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = timeout;
    }

    /// send IPP operation
    pub fn send<T: IppOperation>(&self, operation: T) -> impl Future<Item = IppAttributes, Error = IppError> + Send {
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
    ) -> Box<dyn Future<Item = IppRequestResponse, Error = IppError> + Send> {
        let url = match Url::parse(&self.uri) {
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
                            url.set_port(Some(631)).unwrap();
                        }
                    }
                    _ => {}
                }
                url
            }
            Err(e) => return Box::new(Err(IppError::ParamError(e.to_string())).into_future()),
        };

        debug!("Request URI: {}", url);

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/ipp"));

        let mut builder = Client::builder();

        for cert_file in &self.cacerts {
            let buf = match fs::read(&cert_file) {
                Ok(buf) => buf,
                Err(e) => return Box::new(Err(IppError::from(e)).into_future()),
            };
            let ca_cert = match Certificate::from_der(&buf).or_else(|_| Certificate::from_pem(&buf)) {
                Ok(ca_cert) => ca_cert,
                Err(e) => return Box::new(Err(IppError::from(e)).into_future()),
            };
            builder = builder.add_root_certificate(ca_cert);
        }

        if !self.verify_hostname {
            debug!("Disabling hostname verification!");
            builder = builder.danger_accept_invalid_hostnames(true);
        }

        if !self.verify_certificate {
            debug!("Disabling certificate verification!");
            builder = builder.danger_accept_invalid_certs(true);
        }

        builder = builder.gzip(false).connect_timeout(Duration::from_secs(10));

        if self.timeout > 0 {
            debug!("Setting timeout to {}", self.timeout);
            builder = builder.timeout(Duration::from_secs(self.timeout));
        }

        let fut = builder
            .build()
            .into_future()
            .and_then(|client| client.post(url).headers(headers).body(request.into_stream()).send())
            .and_then(|response| response.error_for_status())
            .and_then(|mut response| {
                let body = mem::replace(response.body_mut(), Decoder::empty());
                body.concat2()
            })
            .map_err(|e| IppError::RequestError(e.to_string()))
            .and_then(|body| {
                let mut reader = BufReader::new(Cursor::new(body));
                let parser = IppParser::new(&mut reader);
                IppRequestResponse::from_parser(parser).map_err(|e| IppError::ParseError(e))
            });

        Box::new(fut)
    }
}
