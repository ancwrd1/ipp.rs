use std::time::Duration;

use futures_util::io::BufReader;
use http::Uri;
use isahc::{
    config::{RedirectPolicy, SslOption},
    http::Method,
    prelude::*,
    Body,
};
use log::debug;

use crate::{
    client::{IppError, CONNECT_TIMEOUT},
    proto::{reader::IppReader, IppParser, IppRequestResponse},
};

pub(super) type ClientError = isahc::Error;

pub(super) struct IsahcClient {
    pub(super) uri: Uri,
    pub(super) timeout: Option<Duration>,
    pub(super) ignore_tls_errors: bool,
}

impl IsahcClient {
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = Request::builder();

        if let Some(timeout) = self.timeout {
            debug!("Setting timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        if self.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder = builder.ssl_options(
                SslOption::DANGER_ACCEPT_INVALID_CERTS
                    | SslOption::DANGER_ACCEPT_REVOKED_CERTS
                    | SslOption::DANGER_ACCEPT_INVALID_HOSTS,
            );
        }

        debug!("Sending request to {}", self.uri);

        let response = builder
            .uri(&self.uri)
            .connect_timeout(CONNECT_TIMEOUT)
            .header("Content-Type", "application/ipp")
            .method(Method::POST)
            .redirect_policy(RedirectPolicy::Limit(10))
            .body(Body::from_reader(request.into_reader()))?
            .send_async()
            .await?;

        debug!("Response status: {}", response.status());

        match response.status().as_u16() {
            200 => IppParser::new(IppReader::new(BufReader::new(response.into_body())))
                .parse()
                .await
                .map_err(IppError::from),
            other => Err(IppError::RequestError(other)),
        }
    }
}
