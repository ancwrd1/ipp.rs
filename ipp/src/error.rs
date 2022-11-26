//!
//! IPP error
//!
use std::io;

use http::uri::InvalidUri;

use crate::{model::StatusCode, parser::IppParseError};

/// IPP error
#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum IppError {
    #[error(transparent)]
    /// HTTP protocol error
    HttpError(#[from] http::Error),

    #[error(transparent)]
    #[cfg(feature = "async-client")]
    /// Client error
    AsyncClientError(#[from] reqwest::Error),

    #[error("HTTP request error: {0}")]
    /// HTTP request error
    RequestError(u16),

    #[error(transparent)]
    /// Network or file I/O error
    IoError(#[from] io::Error),

    #[error("IPP status error: {0}")]
    /// IPP status error
    StatusError(StatusCode),

    #[error("Printer not ready")]
    PrinterNotReady,

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

    #[error(transparent)]
    #[cfg(feature = "client")]
    /// Client error
    ClientError(#[from] ureq::Error),

    #[error(transparent)]
    #[cfg(feature = "async-client-tls")]
    /// TLS error
    TlsError(#[from] native_tls::Error),
}
