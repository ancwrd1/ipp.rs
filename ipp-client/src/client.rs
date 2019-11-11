//!
//! IPP client
//!
use std::{
    borrow::Cow,
    fs, io,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{ready, Future, Stream};
use log::debug;
use num_traits::FromPrimitive;
use reqwest::{Body, Certificate, Client, Response, Url};

use ipp_proto::{
    attribute::{PRINTER_STATE, PRINTER_STATE_REASONS},
    ipp::{self, DelimiterTag, PrinterState},
    operation::IppOperation,
    request::IppRequestResponse,
    AsyncIppParser, IppAttributes, IppOperationBuilder,
};

use crate::IppError;

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

fn parse_uri(uri: String) -> Result<Url, IppError> {
    match Url::parse(&uri) {
        Ok(mut url) => {
            match url.scheme() {
                "ipp" => {
                    url.set_scheme("http").unwrap();
                    if url.port().is_none() {
                        url.set_port(Some(631)).unwrap();
                    }
                }
                "ipps" => {
                    url.set_scheme("https").unwrap();
                    if url.port().is_none() {
                        url.set_port(Some(443)).unwrap();
                    }
                }
                _ => {}
            }
            Ok(url)
        }
        Err(e) => Err(IppError::ParamError(e.to_string())),
    }
}

fn to_device_uri(uri: &str) -> Cow<str> {
    match Url::parse(&uri) {
        Ok(ref mut url) if !url.username().is_empty() => {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            Cow::Owned(url.to_string())
        }
        _ => Cow::Borrowed(uri),
    }
}

fn parse_certs(certs: Vec<PathBuf>) -> Result<Vec<Certificate>, IppError> {
    let mut result = Vec::new();

    for cert_file in certs {
        let buf = match fs::read(&cert_file) {
            Ok(buf) => buf,
            Err(e) => return Err(IppError::from(e)),
        };
        let ca_cert = match Certificate::from_der(&buf).or_else(|_| Certificate::from_pem(&buf)) {
            Ok(ca_cert) => ca_cert,
            Err(e) => return Err(IppError::from(e)),
        };
        result.push(ca_cert);
    }
    Ok(result)
}

struct ResponseStream(Response);

impl Stream for ResponseStream {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut chunk = Box::pin(self.0.chunk());
        match ready!(chunk.as_mut().poll(cx)) {
            Ok(None) => Poll::Ready(None),
            Ok(Some(bytes)) => Poll::Ready(Some(Ok(bytes.to_vec()))),
            Err(e) => Poll::Ready(Some(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))),
        }
    }
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: String,
    pub(crate) ca_certs: Vec<PathBuf>,
    pub(crate) verify_hostname: bool,
    pub(crate) verify_certificate: bool,
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
            .send_request(operation.into_ipp_request(&to_device_uri(&self.uri)))
            .await?;

        if resp.header().operation_status > 2 {
            // IPP error
            Err(IppError::StatusError(
                ipp::StatusCode::from_u16(resp.header().operation_status)
                    .unwrap_or(ipp::StatusCode::ServerErrorInternalError),
            ))
        } else {
            Ok(resp.attributes().clone())
        }
    }

    /// Send request and return response
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        // Some printers don't support gzip
        let mut builder = Client::builder().connect_timeout(Duration::from_secs(10));

        if !self.verify_hostname {
            debug!("Disabling hostname verification!");
            builder = builder.danger_accept_invalid_hostnames(true);
        }

        if !self.verify_certificate {
            debug!("Disabling certificate verification!");
            builder = builder.danger_accept_invalid_certs(true);
        }

        if self.timeout > 0 {
            debug!("Setting timeout to {}", self.timeout);
            builder = builder.timeout(Duration::from_secs(self.timeout));
        }

        let uri = self.uri.clone();
        let ca_certs = self.ca_certs.clone();

        let url = parse_uri(uri)?;
        let certs = parse_certs(ca_certs)?;

        builder = certs
            .into_iter()
            .fold(builder, |builder, ca_cert| builder.add_root_certificate(ca_cert));

        let client = builder.build()?;

        let mut builder = client
            .post(url.clone())
            .header("Content-Type", "application/ipp")
            .body(Body::wrap_stream(request.into_stream()));

        if !url.username().is_empty() {
            debug!("Setting basic auth: {} ****", url.username());
            builder = builder.basic_auth(
                url.username(),
                url.password()
                    .map(|p| percent_encoding::percent_decode(p.as_bytes()).decode_utf8().unwrap()),
            );
        }

        let response = builder.send().await?.error_for_status().map_err(IppError::HttpError)?;

        let stream = ResponseStream(response);

        AsyncIppParser::from(stream)
            .await
            .map_err(IppError::from)
            .map(IppRequestResponse::from_parse_result)
    }
}
