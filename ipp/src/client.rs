//!
//! IPP client
//!
use std::{collections::BTreeMap, marker::PhantomData, time::Duration};

use base64::Engine;
use http::Uri;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

fn ipp_uri_to_string(uri: &Uri) -> String {
    let (scheme, default_port) = match uri.scheme_str() {
        Some("ipps") => ("https", 443),
        Some("ipp") => ("http", 631),
        _ => return uri.to_string(),
    };

    let authority = match uri.authority() {
        Some(authority) => {
            if authority.port_u16().is_some() {
                authority.to_string()
            } else {
                format!("{}:{}", authority, default_port)
            }
        }
        None => return uri.to_string(),
    };

    let path_and_query = uri.path_and_query().map(|p| p.as_str()).unwrap_or_default();

    format!("{}://{}{}", scheme, authority, path_and_query)
}

/// Builder to create IPP client
pub struct IppClientBuilder<T> {
    uri: Uri,
    ignore_tls_errors: bool,
    request_timeout: Option<Duration>,
    headers: BTreeMap<String, String>,
    ca_certs: Vec<Vec<u8>>,
    _phantom_data: PhantomData<T>,
}

impl<T> IppClientBuilder<T> {
    fn new(uri: Uri) -> Self {
        IppClientBuilder {
            uri,
            ignore_tls_errors: false,
            request_timeout: None,
            headers: BTreeMap::new(),
            ca_certs: Vec::new(),
            _phantom_data: PhantomData,
        }
    }

    /// Enable or disable ignoring of TLS handshake errors. Default is false.
    pub fn ignore_tls_errors(mut self, flag: bool) -> Self {
        self.ignore_tls_errors = flag;
        self
    }

    /// Add custom root certificate in PEM or DER format.
    pub fn ca_cert<D: AsRef<[u8]>>(mut self, data: D) -> Self {
        self.ca_certs.push(data.as_ref().to_owned());
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
        let authz =
            base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()));
        self.headers
            .insert("authorization".to_owned(), format!("Basic {authz}"));
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

    use super::{ipp_uri_to_string, IppClientBuilder, CONNECT_TIMEOUT};

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

            #[cfg(any(feature = "async-client-tls", feature = "async-client-rustls"))]
            {
                if self.0.ignore_tls_errors {
                    builder = builder
                        .danger_accept_invalid_hostnames(true)
                        .danger_accept_invalid_certs(true);
                }
                for data in &self.0.ca_certs {
                    let cert =
                        reqwest::Certificate::from_pem(data).or_else(|_| reqwest::Certificate::from_der(data))?;
                    builder = builder.add_root_certificate(cert);
                }
            }

            #[cfg(feature = "async-client-rustls")]
            {
                builder = builder.use_rustls_tls();
            }

            let mut req_builder = builder
                .user_agent(USER_AGENT)
                .build()?
                .post(ipp_uri_to_string(&self.0.uri));

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

    use super::{ipp_uri_to_string, IppClientBuilder, CONNECT_TIMEOUT};

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
                let mut tls_builder = native_tls::TlsConnector::builder();

                tls_builder
                    .danger_accept_invalid_hostnames(self.0.ignore_tls_errors)
                    .danger_accept_invalid_certs(self.0.ignore_tls_errors);

                for data in &self.0.ca_certs {
                    let cert =
                        native_tls::Certificate::from_pem(data).or_else(|_| native_tls::Certificate::from_der(data))?;
                    tls_builder.add_root_certificate(cert);
                }

                let tls_connector = tls_builder.build()?;
                builder = builder.tls_connector(std::sync::Arc::new(tls_connector));
            }

            #[cfg(feature = "client-rustls")]
            {
                use once_cell::sync::Lazy;
                use rustls::pki_types::pem::PemObject;
                use rustls_native_certs::{load_native_certs, CertificateResult};

                static ROOTS: Lazy<CertificateResult> = Lazy::new(load_native_certs);

                let mut root_store = rustls::RootCertStore::empty();
                root_store.add_parsable_certificates(ROOTS.certs.clone());

                for data in &self.0.ca_certs {
                    let cert = rustls::pki_types::CertificateDer::<'static>::from_pem_slice(data)
                        .unwrap_or_else(|_| rustls::pki_types::CertificateDer::from_slice(data));
                    root_store.add(cert)?;
                }

                let secure_config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                let config = if self.0.ignore_tls_errors {
                    rustls::ClientConfig::builder()
                        .dangerous()
                        .with_custom_certificate_verifier(std::sync::Arc::new(verifiers::NoVerifier(
                            secure_config.crypto_provider().clone(),
                        )))
                        .with_no_client_auth()
                } else {
                    secure_config
                };

                builder = builder.tls_config(std::sync::Arc::new(config));
            }

            let agent = builder.user_agent(USER_AGENT).build();

            let mut req = agent
                .post(&ipp_uri_to_string(&self.0.uri))
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

    #[cfg(feature = "client-rustls")]
    mod verifiers {
        use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
        use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
        use rustls::{crypto::CryptoProvider, DigitallySignedStruct, Error, SignatureScheme};
        use std::sync::Arc;

        #[derive(Debug)]
        pub struct NoVerifier(pub Arc<CryptoProvider>);

        impl ServerCertVerifier for NoVerifier {
            fn verify_server_cert(
                &self,
                _end_entity: &CertificateDer,
                _intermediates: &[CertificateDer],
                _server_name: &ServerName,
                _ocsp_response: &[u8],
                _now: UnixTime,
            ) -> Result<ServerCertVerified, Error> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
                self.0.signature_verification_algorithms.supported_schemes()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ipp_uri_to_string;
    use http::Uri;

    #[test]
    fn test_ipp_uri_no_port() {
        let uri = "ipp://user:pass@host/path?query=1234".parse::<Uri>().unwrap();
        let http_uri = ipp_uri_to_string(&uri);
        assert_eq!(http_uri, "http://user:pass@host:631/path?query=1234");
    }

    #[test]
    fn test_ipp_uri_with_port() {
        let uri = "ipp://user:pass@host:1000".parse::<Uri>().unwrap();
        let http_uri = ipp_uri_to_string(&uri);
        assert_eq!(http_uri, "http://user:pass@host:1000/");
    }

    #[test]
    fn test_ipps_uri_no_port() {
        let uri = "ipps://host".parse::<Uri>().unwrap();
        let http_uri = ipp_uri_to_string(&uri);
        assert_eq!(http_uri, "https://host:443/");
    }

    #[test]
    fn test_ipps_uri_with_port() {
        let uri = "ipps://host:8443".parse::<Uri>().unwrap();
        let http_uri = ipp_uri_to_string(&uri);
        assert_eq!(http_uri, "https://host:8443/");
    }

    #[test]
    fn test_http_uri_no_change() {
        let uri = "http://somehost".parse::<Uri>().unwrap();
        let http_uri = ipp_uri_to_string(&uri);
        assert_eq!(http_uri, uri.to_string());
    }
}
