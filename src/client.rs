//!
//! IPP client
//!
use std::io::{BufWriter, BufReader};

use hyper::client::request::Request;
use hyper::method::Method;
use hyper::{self, Url};
use hyper::status::StatusCode;
use hyper::net::{SslClient, NetworkStream, HttpsConnector, Fresh};
use openssl::ssl::{Ssl, SslContext, SslStream, SslMethod};

use request::IppRequest;
use response::IppResponse;
use ::{IppError, Result};
use attribute::{IppAttributeList};
use parser::IppParser;

// Insecure SSL taken from:
// https://github.com/maximih/hyper_insecure_https_connector
#[derive(Debug, Clone)]
struct InsecureOpensslClient(SslContext);

impl Default for InsecureOpensslClient {
    fn default() -> InsecureOpensslClient {
        InsecureOpensslClient(SslContext::new(SslMethod::Sslv23).unwrap())
    }
}

impl<T: NetworkStream + Send + Clone> SslClient<T> for InsecureOpensslClient {
    type Stream = SslStream<T>;

    fn wrap_client(&self, stream: T, host: &str) -> hyper::Result<Self::Stream> {
        let ssl = Ssl::new(&self.0)?;
        ssl.set_hostname(host)?;
        SslStream::connect(ssl, stream).map_err(From::from)
    }
}

#[cfg(target_os = "macos")]
fn make_request(method: Method, url: Url) -> hyper::Result<Request<Fresh>> {
    Request::new(method, url)
}

#[cfg(not(target_os = "macos"))]
fn make_request(method: Method, url: Url) -> hyper::Result<Request<Fresh>> {
    Request::with_connector(method, url, &HttpsConnector::new(InsecureOpensslClient::default()))
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {}

impl IppClient {
    /// Create new instance of the client
    pub fn new() -> IppClient {
        IppClient {}
    }

    /// Send request and return response
    pub fn send_raw<'a>(&self, request: &'a mut IppRequest<'a>) -> Result<IppResponse> {
        match Url::parse(request.uri()) {
            Ok(url) => {
                // create request and set headers
                let mut http_req_fresh = make_request(Method::Post, url)?;
                http_req_fresh.headers_mut().set_raw("Content-Type", vec![b"application/ipp".to_vec()]);

                // connect and send headers
                let mut http_req_stream = http_req_fresh.start()?;

                // send IPP request using buffered writer.
                // NOTE: unbuffered output will cause issues on many IPP implementations including CUPS
                request.write(&mut BufWriter::new(&mut http_req_stream))?;

                // get the response
                let http_resp = http_req_stream.send()?;

                if http_resp.status == StatusCode::Ok {
                    // HTTP 200 assumes we have IPP response to parse
                    let mut reader = BufReader::new(http_resp);
                    let mut parser = IppParser::new(&mut reader);
                    let resp = IppResponse::from_parser(&mut parser)?;

                    Ok(resp)
                } else {
                    error!("HTTP error: {}", http_resp.status);
                    Err(IppError::RequestError(
                        if let Some(reason) = http_resp.status.canonical_reason() {
                            reason.to_string()
                        } else {
                            format!("{}", http_resp.status)
                        }))
                }
            }
            Err(err) => {
                error!("Invalid URI: {}", request.uri());
                Err(IppError::RequestError(err.to_string()))
            }
        }
    }

    /// Send request and return list of attributes if it succeeds
    pub fn send<'a>(&self, request: &'a mut IppRequest<'a>) -> Result<IppAttributeList> {
        match self.send_raw(request) {
            Ok(resp) => {
                if resp.header().status > 3 {
                    // IPP error
                    Err(IppError::StatusError(resp.header().status))
                } else {
                    Ok(resp.attributes().clone())
                }
            }
            Err(err) => Err(err)
        }
    }
}
