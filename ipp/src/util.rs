//!
//! IPP helper functions
//!
use http::{uri::Authority, Uri};

/// convert `http://username:pwd@host:port/path?query` into `ipp://host:port/path`
pub fn canonicalize_uri(uri: &Uri) -> Uri {
    let mut builder = Uri::builder().scheme("ipp").path_and_query(uri.path());
    if let Some(authority) = uri.authority() {
        if let Some(port) = authority.port_u16() {
            builder = builder.authority(format!("{}:{}", authority.host(), port).parse::<Authority>().unwrap());
        } else {
            builder = builder.authority(authority.host());
        }
    }
    builder.build().unwrap_or_else(|_| uri.to_owned())
}

#[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
pub use client_util::{check_printer_state, get_printer_attributes, print_job};

#[cfg(any(feature = "client-isahc", feature = "client-reqwest"))]
mod client_util {
    use std::{fs::File, path::Path};

    use futures_util::io::AllowStdIo;
    use log::debug;

    use crate::prelude::*;

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

    /// Check printer ready status
    pub async fn check_printer_state(client: &IppClient) -> Result<(), IppError> {
        debug!("Checking printer status");
        let attrs = get_printer_attributes(client).await?;

        let state = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(IppAttribute::PRINTER_STATE))
            .and_then(|attr| attr.value().as_enum())
            .and_then(|v| PrinterState::from_i32(*v));

        if let Some(PrinterState::Stopped) = state {
            debug!("Printer is stopped");
            return Err(IppError::PrinterStopped);
        }

        if let Some(reasons) = attrs
            .groups_of(DelimiterTag::PrinterAttributes)
            .get(0)
            .and_then(|g| g.attributes().get(IppAttribute::PRINTER_STATE_REASONS))
        {
            let keywords = reasons
                .value()
                .into_iter()
                .filter_map(|e| e.as_keyword())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();

            if keywords.iter().any(|k| ERROR_STATES.contains(&&k[..])) {
                debug!("Printer is in error state: {:?}", keywords);
                return Err(IppError::PrinterStateError(keywords));
            }
        }
        Ok(())
    }

    /// Print job
    pub async fn print_job<P>(client: &IppClient, job_path: P) -> Result<IppAttributes, IppError>
    where
        P: AsRef<Path>,
    {
        let payload = IppPayload::new(AllowStdIo::new(File::open(job_path.as_ref())?));
        let operation = IppOperationBuilder::print_job(client.uri().clone(), payload).build();
        client.send(operation).await
    }

    /// Get printer attributes
    pub async fn get_printer_attributes(client: &IppClient) -> Result<IppAttributes, IppError> {
        let operation = IppOperationBuilder::get_printer_attributes(client.uri().clone()).build();
        client.send(operation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_device_uri() {
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
