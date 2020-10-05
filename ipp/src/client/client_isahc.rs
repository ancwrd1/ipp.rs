use futures_util::io::BufReader;
use http::Method;
use isahc::{
    config::{RedirectPolicy, SslOption},
    prelude::*,
    Body,
};
use log::debug;

use crate::{
    client::{IppClient, IppError, CONNECT_TIMEOUT},
    proto::{parser::IppParser, request::IppRequestResponse},
};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), ";isahc");

pub(super) type ClientError = isahc::Error;

pub(super) struct IsahcClient<'a>(pub(super) &'a IppClient);

impl<'a> IsahcClient<'a> {
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = Request::builder();

        if let Some(timeout) = self.0.request_timeout {
            debug!("Setting request timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        if self.0.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder = builder.ssl_options(
                SslOption::DANGER_ACCEPT_INVALID_CERTS
                    | SslOption::DANGER_ACCEPT_REVOKED_CERTS
                    | SslOption::DANGER_ACCEPT_INVALID_HOSTS,
            );
        }

        debug!("Sending request to {}", self.0.uri);

        let request = builder
            .uri(&self.0.uri)
            // disable accept-encoding header because it breaks some IPP implementations (older Xerox)
            .automatic_decompression(false)
            .connect_timeout(CONNECT_TIMEOUT)
            .header("content-type", "application/ipp")
            .header("user-agent", USER_AGENT)
            .method(Method::POST)
            .redirect_policy(RedirectPolicy::Limit(10))
            .body(Body::from_reader(request.into_reader()))?;

        let response = request.send_async().await?;

        debug!("Response status: {}", response.status());

        match response.status().as_u16() {
            200..=202 => IppParser::new(BufReader::new(response.into_body()))
                .parse()
                .await
                .map_err(IppError::from),
            other => Err(IppError::RequestError(other)),
        }
    }
}
