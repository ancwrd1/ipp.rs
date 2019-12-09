use std::{borrow::Cow, fmt, io, time::Duration};

use futures::io::BufReader;
use isahc::{
    config::{RedirectPolicy, SslOption},
    http::{uri::Authority, Method, Uri},
    prelude::*,
};
use log::debug;

use crate::proto::{
    attribute::{PRINTER_STATE, PRINTER_STATE_REASONS},
    model::{self, DelimiterTag, PrinterState, StatusCode},
    operation::IppOperation,
    request::IppRequestResponse,
    value::ValueParseError,
    FromPrimitive as _, IppAttributes, IppOperationBuilder, IppParseError, IppParser,
};

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP protocol error
    HttpError(isahc::http::Error),
    /// Client error
    ClientError(isahc::Error),
    /// HTTP request error
    RequestError(u16),
    /// Network or file I/O error
    IOError(io::Error),
    /// IPP status error
    StatusError(StatusCode),
    /// Printer state error
    PrinterStateError(Vec<String>),
    /// Printer stopped
    PrinterStopped,
    /// Parameter error
    ParamError(String),
    /// Parsing error
    ParseError(IppParseError),
    /// Value parsing error
    ValueParseError(ValueParseError),
    /// Missing attribute in response
    MissingAttribute,
    /// Invalid attribute type
    InvalidAttributeType,
}

impl fmt::Display for IppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppError::HttpError(ref e) => write!(f, "{}", e),
            IppError::ClientError(ref e) => write!(f, "{}", e),
            IppError::RequestError(ref e) => write!(f, "HTTP request failed: {}", e),
            IppError::IOError(ref e) => write!(f, "{}", e),
            IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            IppError::ParamError(ref e) => write!(f, "IPP param error: {}", e),
            IppError::PrinterStateError(ref e) => write!(f, "IPP printer state error: {:?}", e),
            IppError::PrinterStopped => write!(f, "IPP printer stopped"),
            IppError::ParseError(ref e) => write!(f, "{}", e),
            IppError::ValueParseError(ref e) => write!(f, "{}", e),
            IppError::MissingAttribute => write!(f, "Missing attribute in response"),
            IppError::InvalidAttributeType => write!(f, "Invalid attribute type"),
        }
    }
}

impl From<io::Error> for IppError {
    fn from(error: io::Error) -> Self {
        IppError::IOError(error)
    }
}

impl From<StatusCode> for IppError {
    fn from(code: StatusCode) -> Self {
        IppError::StatusError(code)
    }
}

impl From<isahc::http::Error> for IppError {
    fn from(error: isahc::http::Error) -> Self {
        IppError::HttpError(error)
    }
}

impl From<isahc::Error> for IppError {
    fn from(error: isahc::Error) -> Self {
        IppError::ClientError(error)
    }
}

impl From<IppParseError> for IppError {
    fn from(error: IppParseError) -> Self {
        IppError::ParseError(error)
    }
}

impl From<ValueParseError> for IppError {
    fn from(error: ValueParseError) -> Self {
        IppError::ValueParseError(error)
    }
}

impl std::error::Error for IppError {}

/// Builder to create IPP client
pub struct IppClientBuilder {
    uri: String,
    ignore_tls_errors: bool,
    timeout: u64,
}

impl IppClientBuilder {
    /// Create a client builder for a given URI
    pub fn new(uri: &str) -> Self {
        IppClientBuilder {
            uri: uri.to_owned(),
            ignore_tls_errors: false,
            timeout: 0,
        }
    }

    /// Enable or disable ignoring of TLS handshake errors. Default is false.
    pub fn ignore_tls_errors(mut self, flag: bool) -> Self {
        self.ignore_tls_errors = flag;
        self
    }

    /// Set network timeout in seconds. Default is 0 (no timeout)
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the client
    pub fn build(self) -> IppClient {
        IppClient {
            uri: self.uri,
            ignore_tls_errors: self.ignore_tls_errors,
            timeout: self.timeout,
        }
    }
}

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

// converts http://username:pwd@host:port/path?query into http://host:port/path
fn canonicalize_uri(uri: &str) -> Cow<str> {
    match uri.parse::<Uri>() {
        Ok(new_uri) => {
            let mut builder = Uri::builder();
            if let Some(scheme) = new_uri.scheme_str() {
                builder.scheme(scheme);
            }
            if let Some(authority) = new_uri.authority_part() {
                if let Some(port) = authority.port_u16() {
                    builder.authority(Authority::from_shared(format!("{}:{}", authority.host(), port).into()).unwrap());
                } else {
                    builder.authority(authority.host());
                }
            }
            builder.path_and_query(new_uri.path());
            builder
                .build()
                .map(|u| Cow::Owned(u.to_string()))
                .unwrap_or_else(|_| Cow::Borrowed(uri))
        }
        Err(_) => Cow::Borrowed(uri),
    }
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: String,
    pub(crate) ignore_tls_errors: bool,
    pub(crate) timeout: u64,
}

impl IppClient {
    /// Check printer ready status
    pub async fn check_ready(&self) -> Result<(), IppError> {
        debug!("Checking printer status");
        let operation = IppOperationBuilder::get_printer_attributes()
            .attributes(&[PRINTER_STATE, PRINTER_STATE_REASONS])
            .build();

        let attrs = self.send(operation).await?;

        let state = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(PRINTER_STATE))
            .and_then(|attr| attr.value().as_enum())
            .and_then(|v| PrinterState::from_i32(*v));

        if let Some(PrinterState::Stopped) = state {
            debug!("Printer is stopped");
            return Err(IppError::PrinterStopped);
        }

        if let Some(reasons) = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(PRINTER_STATE_REASONS))
        {
            let keywords = reasons
                .value()
                .into_iter()
                .filter_map(|e| e.as_keyword())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();

            if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                debug!("Printer is in error state: {:?}", keywords);
                return Err(IppError::PrinterStateError(keywords.clone()));
            }
        }
        Ok(())
    }

    /// send IPP operation
    pub async fn send<T>(&self, operation: T) -> Result<IppAttributes, IppError>
    where
        T: IppOperation,
    {
        debug!("Sending IPP operation");

        let resp = self
            .send_request(operation.into_ipp_request(&canonicalize_uri(&self.uri)))
            .await?;

        if resp.header().operation_status > 2 {
            // IPP error
            Err(IppError::StatusError(
                model::StatusCode::from_u16(resp.header().operation_status)
                    .unwrap_or(model::StatusCode::ServerErrorInternalError),
            ))
        } else {
            Ok(resp.attributes().clone())
        }
    }

    /// Send request and return response
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = Request::builder();

        if self.timeout > 0 {
            debug!("Setting timeout to {}", self.timeout);
            builder.timeout(Duration::from_secs(self.timeout));
        }

        if self.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder.ssl_options(
                SslOption::DANGER_ACCEPT_INVALID_CERTS
                    | SslOption::DANGER_ACCEPT_REVOKED_CERTS
                    | SslOption::DANGER_ACCEPT_INVALID_HOSTS,
            );
        }

        debug!("Sending request to {}", self.uri);

        let response = builder
            .uri(&self.uri)
            .connect_timeout(Duration::from_secs(10))
            .header("Content-Type", "application/ipp")
            .method(Method::POST)
            .redirect_policy(RedirectPolicy::Limit(32))
            .body(Body::reader(request.into_reader()))?
            .send_async()
            .await?;

        debug!("Response status: {}", response.status());

        match response.status().as_u16() {
            200 => IppParser::new(BufReader::new(response.into_body()))
                .parse()
                .await
                .map_err(IppError::from),
            other => Err(IppError::RequestError(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_device_uri() {
        assert_eq!(
            canonicalize_uri("http://user:pass@example.com:631/path?query=val"),
            "http://example.com:631/path"
        );
        assert_eq!(
            canonicalize_uri("http://example.com/path?query=val"),
            "http://example.com/path"
        );
    }
}
