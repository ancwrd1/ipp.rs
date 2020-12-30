use std::io;

use futures_util::{io::BufReader, stream::TryStreamExt};
use log::debug;
use reqwest::{Body, ClientBuilder};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{
    client::{IppClient, IppError, CONNECT_TIMEOUT},
    proto::{parser::IppParser, request::IppRequestResponse},
};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), ";reqwest");

pub(super) type ClientError = reqwest::Error;

pub(super) struct ReqwestClient<'a>(pub(super) &'a IppClient);

impl<'a> ReqwestClient<'a> {
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = ClientBuilder::new().connect_timeout(CONNECT_TIMEOUT);

        if let Some(timeout) = self.0.request_timeout {
            debug!("Setting request timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        if self.0.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder = builder
                .danger_accept_invalid_hostnames(true)
                .danger_accept_invalid_certs(true);
        }

        debug!("Sending request to {}", self.0.uri);

        let response = builder
            .user_agent(USER_AGENT)
            .build()?
            .post(&self.0.uri.to_string())
            .header("content-type", "application/ipp")
            .body(Body::wrap_stream(tokio_util::io::ReaderStream::new(
                request.into_reader().compat(),
            )))
            .send()
            .await?;

        debug!("Response status: {}", response.status());

        match response.status().as_u16() {
            200..=202 => IppParser::new(BufReader::new(
                response
                    .bytes_stream()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .into_async_read(),
            ))
            .parse()
            .await
            .map_err(IppError::from),
            other => Err(IppError::RequestError(other)),
        }
    }
}
