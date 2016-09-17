//!
//! IPP client
//!
use hyper::client::request::Request;
use hyper::method::Method;
use hyper::Url;
use hyper::status::StatusCode;
use hyper::header::{ContentType};
use mime::Mime;
use std::io::BufWriter;

use request::IppRequest;
use response::IppResponse;
use ::{IppError, Result};
use attribute::{IppAttributeList};
use parser::IppParser;

/// IPP client
///
/// IPP client is responsible for sending requests to IPP server
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
                let mut http_req_fresh = try!(Request::new(Method::Post, url));
                let mime: Mime = "application/ipp".parse().unwrap();
                http_req_fresh.headers_mut().set(ContentType(mime));
                let mut http_req_stream = try!(http_req_fresh.start());
                try!(request.write(&mut BufWriter::new(&mut http_req_stream)));

                let mut http_resp = try!(http_req_stream.send());
                debug!("HTTP reply headers: {}", http_resp.headers);
                if http_resp.status == StatusCode::Ok {
                    let mut parser = IppParser::new(&mut http_resp);
                    let resp = try!(IppResponse::from_parser(&mut parser));

                    Ok(resp)
                } else {
                    error!("HTTP error: {}", http_resp.status);
                    Err(IppError::RequestError)
                }
            }
            Err(_) => {
                error!("Invalid URI: {}", request.uri());
                Err(IppError::RequestError)
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
