//!
//! IPP helper functions
//!
use http::Uri;
use num_traits::FromPrimitive;

use crate::{
    attribute::IppAttribute,
    error::IppError,
    model::{DelimiterTag, PrinterState},
    prelude::IppRequestResponse,
    value::IppName,
};

/// convert `http://username:pwd@host:port/path?query` into `ipp://host:port/path`
pub fn canonicalize_uri(uri: &Uri) -> Uri {
    let mut builder = Uri::builder().scheme("ipp").path_and_query(uri.path());
    if let Some(authority) = uri.authority() {
        if let Some(port) = authority.port_u16() {
            builder = builder.authority(format!("{}:{}", authority.host(), port).as_str());
        } else {
            builder = builder.authority(authority.host());
        }
    }
    builder.build().unwrap_or_else(|_| uri.to_owned())
}

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

/// Check if the printer is ready for printing
///
/// * `response` - IPP response to check
pub fn is_printer_ready(response: &IppRequestResponse) -> Result<bool, IppError> {
    let status = response.header().status_code();
    if !status.is_success() {
        return Err(IppError::StatusError(status));
    }

    let printer_state_attr_name: IppName = IppAttribute::PRINTER_STATE.try_into().unwrap();
    let printer_state_reasons_name: IppName = IppAttribute::PRINTER_STATE_REASONS.try_into().unwrap();
    let state = response
        .attributes()
        .groups_of(DelimiterTag::PrinterAttributes)
        .next()
        .and_then(|g| g.attributes().get(&printer_state_attr_name))
        .and_then(|attr| attr.value().as_enum())
        .and_then(|v| PrinterState::from_i32(*v));

    if let Some(PrinterState::Stopped) = state {
        return Ok(false);
    }

    if let Some(reasons) = response
        .attributes()
        .groups_of(DelimiterTag::PrinterAttributes)
        .next()
        .and_then(|g| g.attributes().get(&printer_state_reasons_name))
    {
        let keywords = reasons
            .value()
            .into_iter()
            .filter_map(|e| e.as_keyword())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_uri() {
        assert_eq!(
            canonicalize_uri(&"http://user:pass@example.com:631/path?query=val".parse().unwrap()),
            "ipp://example.com:631/path"
        );
        assert_eq!(
            canonicalize_uri(&"http://example.com/path?query=val".parse().unwrap()),
            "ipp://example.com/path"
        );
    }
}
