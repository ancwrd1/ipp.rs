//!
//! IPP request
//!
use std::io::{self, Read};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use bytes::{BufMut, Bytes, BytesMut};
#[cfg(feature = "async")]
use futures_util::io::{AsyncRead, AsyncReadExt};
use http::Uri;
use log::debug;

use crate::{
    attribute::{IppAttribute, IppAttributes},
    model::{DelimiterTag, IppVersion, Operation, StatusCode},
    payload::IppPayload,
    value::*,
    IppHeader,
};

/// IPP request/response struct
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IppRequestResponse {
    pub(crate) header: IppHeader,
    pub(crate) attributes: IppAttributes,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) payload: IppPayload,
}

impl IppRequestResponse {
    /// Create new IPP request for the operation and uri
    pub fn new(version: IppVersion, operation: Operation, uri: Option<Uri>) -> IppRequestResponse {
        let header = IppHeader::new(version, operation as u16, 1);
        let mut attributes = IppAttributes::new();

        attributes.add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );

        attributes.add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        if let Some(uri) = uri {
            attributes.add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(
                    IppAttribute::PRINTER_URI,
                    IppValue::Uri(crate::util::canonicalize_uri(&uri).to_string()),
                ),
            );
        }

        IppRequestResponse {
            header,
            attributes,
            payload: IppPayload::empty(),
        }
    }

    /// Create response from status and id
    pub fn new_response(version: IppVersion, status: StatusCode, id: u32) -> IppRequestResponse {
        let header = IppHeader::new(version, status as u16, id);
        let mut response = IppRequestResponse {
            header,
            attributes: IppAttributes::new(),
            payload: IppPayload::empty(),
        };

        response.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(IppAttribute::ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        response.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(
                IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
                IppValue::NaturalLanguage("en".to_string()),
            ),
        );

        response
    }

    /// Get IPP header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get mutable IPP header
    pub fn header_mut(&mut self) -> &mut IppHeader {
        &mut self.header
    }

    /// Get attributes
    pub fn attributes(&self) -> &IppAttributes {
        &self.attributes
    }

    /// Get attributes
    pub fn attributes_mut(&mut self) -> &mut IppAttributes {
        &mut self.attributes
    }

    /// Get payload
    pub fn payload(&self) -> &IppPayload {
        &self.payload
    }

    /// Get mutable payload
    pub fn payload_mut(&mut self) -> &mut IppPayload {
        &mut self.payload
    }

    /// Write request to byte array not including payload
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put(self.header.to_bytes());
        buffer.put(self.attributes.to_bytes());
        buffer.freeze()
    }

    #[cfg(feature = "async")]
    /// Convert request/response into AsyncRead including payload
    pub fn into_async_read(self) -> impl AsyncRead + Send + Sync + 'static {
        let header = self.to_bytes();
        debug!("IPP header size: {}", header.len(),);

        futures_util::io::Cursor::new(header).chain(self.payload)
    }

    /// Convert request/response into Read including payload
    pub fn into_read(self) -> impl Read + Send + Sync + 'static {
        let header = self.to_bytes();
        debug!("IPP header size: {}", header.len(),);

        io::Cursor::new(header).chain(self.payload)
    }

    /// Consume request/response and return a payload
    pub fn into_payload(self) -> IppPayload {
        self.payload
    }
}
