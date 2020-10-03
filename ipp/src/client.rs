//!
//! IPP client, selected by `client-isahc` or `client-reqwest` feature
//!
use std::{io, time::Duration};

use http::{uri::InvalidUri, Uri};
use log::debug;
use num_traits::FromPrimitive as _;

#[cfg(feature = "client-isahc")]
mod client_isahc;

#[cfg(all(feature = "client-reqwest", not(feature = "client-isahc")))]
mod client_reqwest;

#[cfg(feature = "client-isahc")]
use client_isahc::{ClientError, IsahcClient as ClientImpl};

#[cfg(all(feature = "client-reqwest", not(feature = "client-isahc")))]
use client_reqwest::{ClientError, ReqwestClient as ClientImpl};

use crate::proto::{
    attribute::IppAttributes, model::StatusCode, operation::IppOperation, parser::IppParseError,
    request::IppRequestResponse,
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
    fn new(uri: Uri) -> Self {
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

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: Uri,
    pub(crate) ignore_tls_errors: bool,
    pub(crate) timeout: Option<Duration>,
}

impl IppClient {
    /// Create IPP client with default options
    pub fn new(uri: Uri) -> Self {
        IppClient {
            uri,
            ignore_tls_errors: false,
            timeout: None,
        }
    }

    /// Create IPP client builder for setting extra options
    pub fn builder(uri: Uri) -> IppClientBuilder {
        IppClientBuilder::new(uri)
    }

    /// Return client URI
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
    /// send IPP operation
    pub async fn send<T>(&self, operation: T) -> Result<IppAttributes, IppError>
    where
        T: IppOperation,
    {
        debug!("Sending IPP operation");

        let resp = self.send_request(operation.into_ipp_request()).await?;

        if resp.header().operation_status > 2 {
            // IPP error
            Err(IppError::StatusError(
                StatusCode::from_u16(resp.header().operation_status).unwrap_or(StatusCode::ServerErrorInternalError),
            ))
        } else {
            Ok(resp.attributes)
        }
    }

    /// Send request and return response
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        ClientImpl(&self).send_request(request).await
    }
}
