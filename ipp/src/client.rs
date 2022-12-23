//!
//! IPP client
//!
use std::{collections::BTreeMap, marker::PhantomData, time::Duration};

use http::Uri;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Builder to create IPP client
pub struct IppClientBuilder<T> {
    uri: Uri,
    ignore_tls_errors: bool,
    request_timeout: Option<Duration>,
    headers: BTreeMap<String, String>,
    _phantom_data: PhantomData<T>,
}

impl<T> IppClientBuilder<T> {
    fn new(uri: Uri) -> Self {
        IppClientBuilder {
            uri,
            ignore_tls_errors: false,
            request_timeout: None,
            headers: BTreeMap::new(),
            _phantom_data: PhantomData::default(),
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

    /// Add basic auth header (RFC 7617)
    pub fn basic_auth<U, P>(mut self, username: U, password: P) -> Self
    where
        U: AsRef<str>,
        P: AsRef<str>,
    {
        let authz = base64::encode(format!("{}:{}", username.as_ref(), password.as_ref()));
        self.headers
            .insert("authorization".to_owned(), format!("Basic {}", authz));
        self
    }
}

#[cfg(feature = "async-client")]
impl IppClientBuilder<non_blocking::AsyncIppClient> {
    /// Build the async client
    pub fn build(self) -> non_blocking::AsyncIppClient {
        non_blocking::AsyncIppClient(self)
    }
}

#[cfg(feature = "client")]
impl IppClientBuilder<blocking::IppClient> {
    /// Build the blocking client
    pub fn build(self) -> blocking::IppClient {
        blocking::IppClient(self)
    }
}

#[cfg(feature = "async-client")]
pub mod non_blocking {
    use std::io;

    use futures_util::{io::BufReader, stream::TryStreamExt};
    use http::Uri;
    use reqwest::{Body, ClientBuilder};
    use tokio_util::compat::FuturesAsyncReadCompatExt;

    use crate::{error::IppError, parser::AsyncIppParser, request::IppRequestResponse};

    use super::{IppClientBuilder, CONNECT_TIMEOUT};

    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), ";reqwest");

    /// Asynchronous IPP client.
    ///
    /// IPP client is responsible for sending requests to IPP server.
    pub struct AsyncIppClient(pub(super) IppClientBuilder<Self>);

    impl AsyncIppClient {
        /// Create IPP client with default options
        pub fn new(uri: Uri) -> Self {
            AsyncIppClient(AsyncIppClient::builder(uri))
        }

        /// Create IPP client builder for setting extra options
        pub fn builder(uri: Uri) -> IppClientBuilder<Self> {
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
                builder = builder.timeout(timeout);
            }

            #[cfg(feature = "async-client-tls")]
            if self.0.ignore_tls_errors {
                builder = builder
                    .danger_accept_invalid_hostnames(true)
                    .danger_accept_invalid_certs(true);
            }

            let mut req_builder = builder.user_agent(USER_AGENT).build()?.post(self.0.uri.to_string());

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
}

#[cfg(feature = "client")]
pub mod blocking {
    use http::Uri;
    use ureq::AgentBuilder;

    use crate::{error::IppError, parser::IppParser, reader::IppReader, request::IppRequestResponse};

    use super::{IppClientBuilder, CONNECT_TIMEOUT};

    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), ";ureq");

    /// Blocking IPP client.
    ///
    /// IPP client is responsible for sending requests to IPP server.
    pub struct IppClient(pub(super) IppClientBuilder<Self>);

    impl IppClient {
        /// Create IPP client with default options
        pub fn new(uri: Uri) -> Self {
            IppClient(IppClient::builder(uri))
        }

        /// Create IPP client builder for setting extra options
        pub fn builder(uri: Uri) -> IppClientBuilder<Self> {
            IppClientBuilder::new(uri)
        }

        /// Return client URI
        pub fn uri(&self) -> &Uri {
            &self.0.uri
        }

        /// Send IPP request to the server
        pub fn send<R>(&self, request: R) -> Result<IppRequestResponse, IppError>
        where
            R: Into<IppRequestResponse>,
        {
            let mut builder = AgentBuilder::new().timeout_connect(CONNECT_TIMEOUT);

            if let Some(timeout) = self.0.request_timeout {
                builder = builder.timeout(timeout);
            }

            #[cfg(feature = "client-tls")]
            {
                let tls_connector = native_tls::TlsConnector::builder()
                    .danger_accept_invalid_hostnames(self.0.ignore_tls_errors)
                    .danger_accept_invalid_certs(self.0.ignore_tls_errors)
                    .build()?;
                builder = builder.tls_connector(std::sync::Arc::new(tls_connector));
            }

            let agent = builder.user_agent(USER_AGENT).build();

            let mut req = agent
                .post(&self.0.uri.to_string())
                .set("content-type", "application/ipp");

            for (k, v) in &self.0.headers {
                req = req.set(k, v);
            }

            let response = req.send(request.into().into_read())?;
            let reader = response.into_reader();
            let parser = IppParser::new(IppReader::new(reader));

            parser.parse().map_err(IppError::from)
        }
    }
}
