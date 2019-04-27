//!
//! IPP client
//!
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

use num_traits::FromPrimitive;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Certificate, Client, Method};
use url::Url;

use attribute::IppAttributeList;
use consts::statuscode;
use operation::IppOperation;
use parser::IppParser;
use request::IppRequestResponse;
use {IppError, Result};

const TIMEOUT: u64 = 30;

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    uri: String,
    cacerts: Vec<String>,
    verify_hostname: bool,
}

impl IppClient {
    /// Create new instance of the client
    pub fn new(uri: &str) -> IppClient {
        IppClient {
            uri: uri.to_string(),
            cacerts: Vec::new(),
            verify_hostname: true,
        }
    }

    /// Create new instance of the client with a list of root CA certificates
    ///
    /// * `uri` - target printer URI
    /// * `certfiles` - list of certificate file names
    pub fn with_root_certificates<T>(uri: &str, certfiles: &[T]) -> IppClient
    where
        T: AsRef<str>,
    {
        IppClient {
            uri: uri.to_string(),
            cacerts: certfiles
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect::<Vec<String>>(),
            verify_hostname: true,
        }
    }

    /// Enable or disable host name validation for SSL transport. By default it is enabled.
    pub fn set_verify_hostname(&mut self, verify: bool) {
        self.verify_hostname = verify;
    }

    /// send IPP operation
    pub fn send<T: IppOperation>(&self, operation: T) -> Result<IppAttributeList> {
        match self.send_request(operation.to_ipp_request(&self.uri)) {
            Ok(resp) => {
                if resp.header().operation_status > 3 {
                    // IPP error
                    Err(IppError::StatusError(
                        statuscode::StatusCode::from_u16(resp.header().operation_status)
                            .unwrap_or(statuscode::StatusCode::ServerErrorInternalError),
                    ))
                } else {
                    Ok(resp.attributes().clone())
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Send request and return response
    pub fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse> {
        match Url::parse(&self.uri) {
            Ok(mut url) => {
                match url.scheme() {
                    "ipp" => {
                        url.set_scheme("http")
                            .map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;
                        if url.port().is_none() {
                            url.set_port(Some(631))
                                .map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;
                        }
                    }
                    "ipps" => {
                        url.set_scheme("https")
                            .map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;
                        if url.port().is_none() {
                            url.set_port(Some(631))
                                .map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;
                        }
                    }
                    _ => {}
                }

                debug!("Request URI: {}", url);

                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("application/ipp"));

                let mut builder = Client::builder();

                for certfile in &self.cacerts {
                    let mut f = File::open(&certfile)?;
                    let mut buf = Vec::new();
                    let size = f.read_to_end(&mut buf)?;
                    let cacert = match Certificate::from_der(&buf[0..size]) {
                        Ok(cacert) => {
                            debug!("Read DER certificate from {}", certfile);
                            cacert
                        }
                        Err(_) => {
                            let cacert = Certificate::from_pem(&buf[0..size])?;
                            debug!("Read PEM certificate from {}", certfile);
                            cacert
                        }
                    };
                    builder = builder.add_root_certificate(cacert);
                }

                if !self.verify_hostname {
                    debug!("Disabling hostname verification!");
                    builder = builder.danger_accept_invalid_hostnames(true);
                }

                let client = builder
                    .gzip(false)
                    .timeout(Duration::from_secs(TIMEOUT))
                    .build()?;

                let http_req = client
                    .request(Method::POST, url)
                    .headers(headers)
                    .body(Body::new(request.into_reader()))
                    .build()?;
                let http_resp = client.execute(http_req)?;

                if http_resp.status().is_success() {
                    // HTTP 200 assumes we have IPP response to parse
                    let mut reader = BufReader::new(http_resp);
                    let mut parser = IppParser::new(&mut reader);
                    let resp = IppRequestResponse::from_parser(&mut parser)?;

                    Ok(resp)
                } else {
                    error!("HTTP error: {}", http_resp.status());
                    Err(IppError::RequestError(
                        if let Some(reason) = http_resp.status().canonical_reason() {
                            reason.to_string()
                        } else {
                            format!("{}", http_resp.status())
                        },
                    ))
                }
            }
            Err(err) => {
                error!("Invalid URI: {}", self.uri);
                Err(IppError::RequestError(err.to_string()))
            }
        }
    }
}
