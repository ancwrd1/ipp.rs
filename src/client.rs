//!
//! IPP client
//!
use std::io::BufReader;
use std::time::Duration;
use enum_primitive::FromPrimitive;
use reqwest::{Client, Method,  Body, StatusCode};
use reqwest::header::Headers;
use url::Url;

use ::{IppError, Result};
use request::IppRequestResponse;
use operation::IppOperation;
use attribute::IppAttributeList;
use parser::IppParser;
use consts::statuscode;

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    uri: String
}

impl IppClient {
    /// Create new instance of the client
    pub fn new(uri: &str) -> IppClient {
        IppClient {
            uri: uri.to_string()
        }
    }

    /// send IPP operation
    pub fn send<T: IppOperation>(&self, operation: T) -> Result<IppAttributeList> {
        match self.send_request(operation.to_ipp_request(&self.uri)) {
            Ok(resp) => {
                if resp.header().operation_status > 3 {
                    // IPP error
                    Err(IppError::StatusError(
                        statuscode::StatusCode::from_u16(resp.header().operation_status)
                            .unwrap_or(statuscode::StatusCode::ServerErrorInternalError)))
                } else {
                    Ok(resp.attributes().clone())
                }
            }
            Err(err) => Err(err)
        }
    }

    /// Send request and return response
    pub fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse> {
        match Url::parse(&self.uri) {
            Ok(mut url) => {
                if url.scheme() == "ipp" {
                    url.set_scheme("http").map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;;
                    if  url.port().is_none() {
                        url.set_port(Some(631)).map_err(|_| IppError::RequestError("Invalid URI".to_string()))?;
                    }
                }

                debug!("Request URI: {}", url);

                let mut headers = Headers::new();
                headers.set_raw("Content-Type", "application/ipp");

                let client = Client::builder().gzip(false).timeout(Duration::new(30, 0)).build()?;
                let http_req = client.request(Method::Post, url).headers(headers).body(Body::new(request.into_reader())).build()?;
                let http_resp = client.execute(http_req)?;

                if http_resp.status() == StatusCode::Ok {
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
                        }))
                }
            }
            Err(err) => {
                error!("Invalid URI: {}", self.uri);
                Err(IppError::RequestError(err.to_string()))
            }
        }
    }
}
