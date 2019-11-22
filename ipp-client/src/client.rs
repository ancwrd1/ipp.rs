//!
//! IPP client
//!
use std::{borrow::Cow, time::Duration};

use http::Method;
use isahc::config::{RedirectPolicy, SslOption};
use isahc::prelude::*;
use log::debug;
use num_traits::FromPrimitive;

use ipp_proto::{
    attribute::{PRINTER_STATE, PRINTER_STATE_REASONS},
    ipp::{self, DelimiterTag, PrinterState},
    operation::IppOperation,
    request::IppRequestResponse,
    AsyncIppParser, IppAttributes, IppOperationBuilder,
};

use crate::IppError;
use http::uri::Authority;

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
fn canonicalize_uri(uri: &str) -> Cow<str> {
    match uri.parse::<http::Uri>() {
        Ok(new_uri) => {
            let mut builder = http::Uri::builder();
            if let Some(scheme) = new_uri.scheme_str() {
                builder.scheme(scheme);
            }
            if let Some(authority) = new_uri.authority_part() {
                if let Some(port) = authority.port_u16() {
                    builder.authority(Authority::from_shared(format!("{}:{}", authority.host(), port).into()).unwrap());
                } else {
                    builder.authority(authority.host());
                }
            }
            builder.path_and_query(new_uri.path());
            builder
                .build()
                .map(|u| Cow::Owned(u.to_string()))
                .unwrap_or_else(|_| Cow::Borrowed(uri))
        }
        Err(_) => Cow::Borrowed(uri),
    }
}

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {
    pub(crate) uri: String,
    pub(crate) ignore_tls_errors: bool,
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
            .send_request(operation.into_ipp_request(&canonicalize_uri(&self.uri)))
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
        let mut builder = Request::builder();
        builder.uri(&self.uri);
        builder.connect_timeout(Duration::from_secs(10));
        builder.header("Content-Type", "application/ipp");
        builder.method(Method::POST);
        builder.redirect_policy(RedirectPolicy::Limit(3));

        if self.timeout > 0 {
            debug!("Setting timeout to {}", self.timeout);
            builder.timeout(Duration::from_secs(self.timeout));
        }

        if self.ignore_tls_errors {
            builder.ssl_options(
                SslOption::DANGER_ACCEPT_INVALID_CERTS
                    | SslOption::DANGER_ACCEPT_REVOKED_CERTS
                    | SslOption::DANGER_ACCEPT_INVALID_HOSTS,
            );
        }

        let request = builder.body(Body::reader(request.into_reader()))?;
        let resp = request.send_async().await?;

        let status = resp.status().as_u16();
        if status != 200 {
            return Err(IppError::RequestError(status));
        }

        AsyncIppParser::from(resp.into_body())
            .await
            .map_err(IppError::from)
            .map(IppRequestResponse::from_parse_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_device_uri() {
        assert_eq!(
            canonicalize_uri("http://user:pass@example.com:631/path?query=val"),
            "http://example.com:631/path"
        );
        assert_eq!(
            canonicalize_uri("http://example.com/path?query=val"),
            "http://example.com/path"
        );
    }
}
