//!
//! IPP client
//!
use std::collections::BTreeMap;
use std::{io, time::Duration};

use futures_util::{io::BufReader, stream::TryStreamExt};
use http::{uri::InvalidUri, Uri};
use log::debug;
use reqwest::{Body, ClientBuilder};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{
    model::StatusCode,
    parser::{AsyncIppParser, IppParseError},
    request::IppRequestResponse,
};

pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), ";hyper");

/// IPP error
#[derive(Debug, thiserror::Error)]
pub enum IppError {
    #[error(transparent)]
    /// HTTP protocol error
    HttpError(#[from] http::Error),

    #[error(transparent)]
    /// Client error
    ClientError(#[from] reqwest::Error),

    #[error("HTTP request error: {0}")]
    /// HTTP request error
    RequestError(u16),

    #[error(transparent)]
    /// Network or file I/O error
    IoError(#[from] io::Error),

    #[error("IPP status error: {0}")]
    /// IPP status error
    StatusError(StatusCode),

    #[error("Printer state error: {0:?}")]
    /// Printer state error
    PrinterStateError(Vec<String>),

    #[error("Printer stopped")]
    /// Printer stopped
    PrinterStopped,

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
    request_timeout: Option<Duration>,
    headers: BTreeMap<String, String>,
}

impl IppClientBuilder {
    fn new(uri: Uri) -> Self {
        IppClientBuilder {
            uri,
            ignore_tls_errors: false,
            request_timeout: None,
            headers: BTreeMap::new(),
        }
    }

    /// Enable or disable ignoring of TLS handshake errors. Default is false.
    pub fn ignore_tls_errors(mut self, flag: bool) -> Self {
        self.ignore_tls_errors = flag;
        self
    }

    /// Set network request timeout. Default is no timeout.
    pub fn request_timeout(mut self, duration: Duration) -> Self {
        self.request_timeout = Some(duration);
        self
    }

    /// Add custom HTTP header
    pub fn http_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.headers.insert(key.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Build the client
    pub fn build(self) -> IppClient {
        IppClient(self)
    }
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient(IppClientBuilder);

impl IppClient {
    /// Create IPP client with default options
    pub fn new(uri: Uri) -> Self {
        IppClient(IppClient::builder(uri))
    }

    /// Create IPP client builder for setting extra options
    pub fn builder(uri: Uri) -> IppClientBuilder {
        IppClientBuilder::new(uri)
    }

    /// Return client URI
    pub fn uri(&self) -> &Uri {
        &self.0.uri
    }

    /// Send IPP request to the server
    pub async fn send<R>(&self, request: R) -> Result<IppRequestResponse, IppError>
    where
        R: Into<IppRequestResponse>,
    {
        let mut builder = ClientBuilder::new().connect_timeout(CONNECT_TIMEOUT);

        if let Some(timeout) = self.0.request_timeout {
            debug!("Setting request timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        #[cfg(feature = "tls")]
        if self.0.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder = builder
                .danger_accept_invalid_hostnames(true)
                .danger_accept_invalid_certs(true);
        }

        debug!("Sending request to {}", self.0.uri);

        let mut req_builder = builder.user_agent(USER_AGENT).build()?.post(&self.0.uri.to_string());

        for (k, v) in &self.0.headers {
            req_builder = req_builder.header(k, v);
        }

        let response = req_builder
            .header("content-type", "application/ipp")
            .body(Body::wrap_stream(tokio_util::io::ReaderStream::new(
                request.into().into_async_read().compat(),
            )))
            .send()
            .await?;

        debug!("Response status: {}", response.status());

        if response.status().is_success() {
            let parser = AsyncIppParser::new(BufReader::new(
                response
                    .bytes_stream()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .into_async_read(),
            ));
            parser.parse().await.map_err(IppError::from)
        } else {
            Err(IppError::RequestError(response.status().as_u16()))
        }
    }
}
