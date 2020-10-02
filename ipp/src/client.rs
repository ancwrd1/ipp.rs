use std::{io, time::Duration};

use http::{
    uri::{Authority, InvalidUri},
    Uri,
};
use log::debug;

#[cfg(feature = "client-isahc")]
mod client_isahc;

#[cfg(all(feature = "client-reqwest", not(feature = "client-isahc")))]
mod client_reqwest;

#[cfg(feature = "client-isahc")]
use client_isahc::{ClientError, IsahcClient as ClientImpl};

#[cfg(all(feature = "client-reqwest", not(feature = "client-isahc")))]
use client_reqwest::{ClientError, ReqwestClient as ClientImpl};

use crate::proto::{
    model::{self, StatusCode},
    operation::IppOperation,
    request::IppRequestResponse,
    FromPrimitive as _, IppAttributes, IppParseError,
};

pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// IPP error
#[derive(Debug, thiserror::Error)]
pub enum IppError {
    #[error(transparent)]
    /// HTTP protocol error
    HttpError(#[from] http::Error),

    #[error(transparent)]
    /// Client error
    ClientError(#[from] ClientError),

    #[error("HTTP request error: {0}")]
    /// HTTP request error
    RequestError(u16),

    #[error(transparent)]
    /// Network or file I/O error
    IOError(#[from] io::Error),

    #[error("IPP status error: {0}")]
    /// IPP status error
    StatusError(StatusCode),

    #[error("Printer state error: {0:?}")]
    /// Printer state error
    PrinterStateError(Vec<String>),

    #[error("Printer stopped")]
    /// Printer stopped
    PrinterStopped,

    #[error("IPP parameter error: {0}")]
    /// Parameter error
    ParamError(String),

    #[error(transparent)]
    /// Parsing error
    ParseError(#[from] IppParseError),

    #[error("Missing attribute in response")]
    /// Missing attribute in response
    MissingAttribute,

    #[error("Invalid attribute type")]
    /// Invalid attribute type
    InvalidAttributeType,

    #[error(transparent)]
    /// Invalid URI
    InvalidUri(#[from] InvalidUri),
}

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
    /// send IPP operation
    pub async fn send<T>(&self, operation: T) -> Result<IppAttributes, IppError>
    where
        T: IppOperation,
    {
        debug!("Sending IPP operation");

        let resp = self
            .send_request(operation.into_ipp_request(canonicalize_uri(&self.uri)))
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
