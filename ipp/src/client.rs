use std::{fmt, io, time::Duration};

use http::{
    uri::{Authority, InvalidUri},
    Uri,
};
use log::debug;

#[cfg(all(feature = "client-isahc", not(feature = "client-reqwest")))]
use isahc_client::{ClientError, IsahcClient as ClientImpl};

#[cfg(all(not(feature = "client-isahc"), feature = "client-reqwest"))]
use reqwest_client::{ClientError, ReqwestClient as ClientImpl};

use crate::proto::{
    model::{self, DelimiterTag, PrinterState, StatusCode},
    operation::IppOperation,
    request::IppRequestResponse,
    value::ValueParseError,
    FromPrimitive as _, IppAttribute, IppAttributes, IppOperationBuilder, IppParseError,
};

#[cfg(all(feature = "client-isahc", not(feature = "client-reqwest")))]
mod isahc_client;

#[cfg(all(not(feature = "client-isahc"), feature = "client-reqwest"))]
mod reqwest_client;

pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// IPP error
#[derive(Debug)]
pub enum IppError {
    /// HTTP protocol error
    HttpError(http::Error),
    /// Client error
    ClientError(ClientError),
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
    /// Invalid URI
    InvalidUri(InvalidUri),
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
            IppError::InvalidUri(ref e) => write!(f, "{}", e),
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

impl From<http::Error> for IppError {
    fn from(error: http::Error) -> Self {
        IppError::HttpError(error)
    }
}

impl From<ClientError> for IppError {
    fn from(error: ClientError) -> Self {
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

impl From<InvalidUri> for IppError {
    fn from(error: InvalidUri) -> Self {
        IppError::InvalidUri(error)
    }
}

impl std::error::Error for IppError {}

/// Builder to create IPP client
pub struct IppClientBuilder {
    uri: Uri,
    ignore_tls_errors: bool,
    timeout: Option<Duration>,
}

impl IppClientBuilder {
    /// Create a client builder for a given URI
    pub fn new(uri: Uri) -> Self {
        IppClientBuilder {
            uri,
            ignore_tls_errors: false,
            timeout: None,
        }
    }

    /// Enable or disable ignoring of TLS handshake errors. Default is false.
    pub fn ignore_tls_errors(mut self, flag: bool) -> Self {
        self.ignore_tls_errors = flag;
        self
    }

    /// Set network timeout in seconds. Default is 0 (no timeout)
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
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
fn canonicalize_uri(uri: &Uri) -> Uri {
    let mut builder = Uri::builder();
    if let Some(scheme) = uri.scheme_str() {
        builder = builder.scheme(scheme);
    }
    if let Some(authority) = uri.authority() {
        if let Some(port) = authority.port_u16() {
            builder = builder.authority(format!("{}:{}", authority.host(), port).parse::<Authority>().unwrap());
        } else {
            builder = builder.authority(authority.host());
        }
    }
    builder
        .path_and_query(uri.path())
        .build()
        .unwrap_or_else(|_| uri.to_owned())
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: Uri,
    pub(crate) ignore_tls_errors: bool,
    pub(crate) timeout: Option<Duration>,
}

impl IppClient {
    /// Check printer ready status
    pub async fn check_ready(&self) -> Result<(), IppError> {
        debug!("Checking printer status");
        let operation = IppOperationBuilder::get_printer_attributes()
            .attributes(&[IppAttribute::PRINTER_STATE, IppAttribute::PRINTER_STATE_REASONS])
            .build();

        let attrs = self.send(operation).await?;

        let state = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(IppAttribute::PRINTER_STATE))
            .and_then(|attr| attr.value().as_enum())
            .and_then(|v| PrinterState::from_i32(*v));

        if let Some(PrinterState::Stopped) = state {
            debug!("Printer is stopped");
            return Err(IppError::PrinterStopped);
        }

        if let Some(reasons) = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(IppAttribute::PRINTER_STATE_REASONS))
        {
            let keywords = reasons
                .value()
                .into_iter()
                .filter_map(|e| e.as_keyword())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();

            if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                debug!("Printer is in error state: {:?}", keywords);
                return Err(IppError::PrinterStateError(keywords));
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
            .send_request(operation.into_ipp_request(&canonicalize_uri(&self.uri).to_string()))
            .await?;

        if resp.header().operation_status > 2 {
            // IPP error
            Err(IppError::StatusError(
                model::StatusCode::from_u16(resp.header().operation_status)
                    .unwrap_or(model::StatusCode::ServerErrorInternalError),
            ))
        } else {
            Ok(resp.attributes)
        }
    }

    /// Send request and return response
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let imp = ClientImpl {
            uri: self.uri.clone(),
            ignore_tls_errors: self.ignore_tls_errors,
            timeout: self.timeout,
        };
        imp.send_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_device_uri() {
        assert_eq!(
            canonicalize_uri(&"http://user:pass@example.com:631/path?query=val".parse().unwrap()),
            "http://example.com:631/path"
        );
        assert_eq!(
            canonicalize_uri(&"http://example.com/path?query=val".parse().unwrap()),
            "http://example.com/path"
        );
    }
}
