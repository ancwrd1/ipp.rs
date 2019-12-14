//!
//! Attribute-related structs
//!
use std::collections::HashMap;

use bytes::{BufMut, Bytes, BytesMut};

use crate::proto::model::DelimiterTag;

use super::IppValue;

fn is_header_attr(attr: &str) -> bool {
    IppAttribute::HEADER_ATTRS.iter().any(|&at| at == attr)
}

/// `IppAttribute` represents an IPP attribute
#[derive(Clone, Debug)]
pub struct IppAttribute {
    /// Attribute name
    name: String,
    /// Attribute value
    value: IppValue,
}

impl IppAttribute {
    pub const ATTRIBUTES_CHARSET: &'static str = "attributes-charset";
    pub const ATTRIBUTES_NATURAL_LANGUAGE: &'static str = "attributes-natural-language";
    pub const CHARSET_CONFIGURED: &'static str = "charset-configured";
    pub const CHARSET_SUPPORTED: &'static str = "charset-supported";
    pub const COMPRESSION_SUPPORTED: &'static str = "compression-supported";
    pub const DOCUMENT_FORMAT_DEFAULT: &'static str = "document-format-default";
    pub const DOCUMENT_FORMAT_SUPPORTED: &'static str = "document-format-supported";
    pub const GENERATED_NATURAL_LANGUAGE_SUPPORTED: &'static str = "generated-natural-language-supported";
    pub const IPP_VERSIONS_SUPPORTED: &'static str = "ipp-versions-supported";
    pub const NATURAL_LANGUAGE_CONFIGURED: &'static str = "natural-language-configured";
    pub const OPERATIONS_SUPPORTED: &'static str = "operations-supported";
    pub const PDL_OVERRIDE_SUPPORTED: &'static str = "pdl-override-supported";
    pub const PRINTER_IS_ACCEPTING_JOBS: &'static str = "printer-is-accepting-jobs";
    pub const PRINTER_MAKE_AND_MODEL: &'static str = "printer-make-and-model";
    pub const PRINTER_NAME: &'static str = "printer-name";
    pub const PRINTER_STATE: &'static str = "printer-state";
    pub const PRINTER_STATE_MESSAGE: &'static str = "printer-state-message";
    pub const PRINTER_STATE_REASONS: &'static str = "printer-state-reasons";
    pub const PRINTER_UP_TIME: &'static str = "printer-up-time";
    pub const PRINTER_URI: &'static str = "printer-uri";
    pub const PRINTER_URI_SUPPORTED: &'static str = "printer-uri-supported";
    pub const QUEUED_JOB_COUNT: &'static str = "queued-job-count";
    pub const URI_AUTHENTICATION_SUPPORTED: &'static str = "uri-authentication-supported";
    pub const URI_SECURITY_SUPPORTED: &'static str = "uri-security-supported";
    pub const JOB_ID: &'static str = "job-id";
    pub const JOB_NAME: &'static str = "job-name";
    pub const JOB_STATE: &'static str = "job-state";
    pub const JOB_STATE_REASONS: &'static str = "job-state-reasons";
    pub const JOB_URI: &'static str = "job-uri";
    pub const LAST_DOCUMENT: &'static str = "last-document";
    pub const REQUESTING_USER_NAME: &'static str = "requesting-user-name";
    pub const STATUS_MESSAGE: &'static str = "status-message";
    pub const REQUESTED_ATTRIBUTES: &'static str = "requested-attributes";
    pub const SIDES_SUPPORTED: &'static str = "sides-supported";
    pub const OUTPUT_MODE_SUPPORTED: &'static str = "output-mode-supported";
    pub const COLOR_SUPPORTED: &'static str = "color-supported";
    pub const PRINTER_INFO: &'static str = "printer-info";
    pub const PRINTER_LOCATION: &'static str = "printer-location";
    pub const PRINTER_MORE_INFO: &'static str = "printer-more-info";
    pub const PRINTER_RESOLUTION_DEFAULT: &'static str = "printer-resolution-default";
    pub const PRINTER_RESOLUTION_SUPPORTED: &'static str = "printer-resolution-supported";
    pub const COPIES_SUPPORTED: &'static str = "copies-supported";
    pub const COPIES_DEFAULT: &'static str = "copies-default";
    pub const SIDES_DEFAULT: &'static str = "sides-default";
    pub const PRINT_QUALITY_DEFAULT: &'static str = "print-quality-default";
    pub const PRINT_QUALITY_SUPPORTED: &'static str = "print-quality-supported";
    pub const FINISHINGS_DEFAULT: &'static str = "finishings-default";
    pub const FINISHINGS_SUPPORTED: &'static str = "finishings-supported";
    pub const OUTPUT_BIN_DEFAULT: &'static str = "output-bin-default";
    pub const OUTPUT_BIN_SUPPORTED: &'static str = "output-bin-supported";
    pub const ORIENTATION_REQUESTED_DEFAULT: &'static str = "orientation-requested-default";
    pub const ORIENTATION_REQUESTED_SUPPORTED: &'static str = "orientation-requested-supported";
    pub const MEDIA_DEFAULT: &'static str = "media-default";
    pub const MEDIA_SUPPORTED: &'static str = "media-supported";
    pub const PAGES_PER_MINUTE: &'static str = "pages-per-minute";
    pub const COLOR_MODE_SUPPORTED: &'static str = "color-mode-supported";
    pub const PRINT_COLOR_MODE_SUPPORTED: &'static str = "print-color-mode-supported";

    const HEADER_ATTRS: [&'static str; 3] = [
        IppAttribute::ATTRIBUTES_CHARSET,
        IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
        IppAttribute::PRINTER_URI,
    ];

    /// Create new instance of the attribute
    ///
    /// * `name` - Attribute name<br/>
    /// * `value` - Attribute value<br/>
    pub fn new<S>(name: S, value: IppValue) -> IppAttribute
    where
        S: AsRef<str>,
    {
        IppAttribute {
            name: name.as_ref().to_owned(),
            value,
        }
    }

    /// Return attribute name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return attribute value
    pub fn value(&self) -> &IppValue {
        &self.value
    }

    /// Write attribute to byte array
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.value.to_tag());
        buffer.put_u16(self.name.len() as u16);
        buffer.put_slice(self.name.as_bytes());
        buffer.put(self.value.to_bytes());
        buffer.freeze()
    }
}

/// Attribute group
#[derive(Clone, Debug)]
pub struct IppAttributeGroup {
    tag: DelimiterTag,
    attributes: HashMap<String, IppAttribute>,
}

impl IppAttributeGroup {
    /// Create new attribute group of a given type
    pub fn new(tag: DelimiterTag) -> IppAttributeGroup {
        IppAttributeGroup {
            tag,
            attributes: HashMap::new(),
        }
    }

    /// Return group type tag
    pub fn tag(&self) -> DelimiterTag {
        self.tag
    }

    /// Return read-only attributes
    pub fn attributes(&self) -> &HashMap<String, IppAttribute> {
        &self.attributes
    }

    /// Return mutable attributes
    pub fn attributes_mut(&mut self) -> &mut HashMap<String, IppAttribute> {
        &mut self.attributes
    }
}

/// Attribute list
#[derive(Clone, Debug, Default)]
pub struct IppAttributes {
    groups: Vec<IppAttributeGroup>,
}

impl IppAttributes {
    /// Create attribute list
    pub fn new() -> IppAttributes {
        IppAttributes { ..Default::default() }
    }

    /// Get all groups
    pub fn groups(&self) -> &Vec<IppAttributeGroup> {
        &self.groups
    }

    /// Get all mutable groups
    pub fn groups_mut(&mut self) -> &mut Vec<IppAttributeGroup> {
        &mut self.groups
    }

    /// Get a list of attribute groups matching a given delimiter tag
    pub fn groups_of(&self, tag: DelimiterTag) -> Vec<&IppAttributeGroup> {
        self.groups.iter().filter(|g| g.tag == tag).collect()
    }

    /// Add attribute to a given group
    pub fn add(&mut self, tag: DelimiterTag, attribute: IppAttribute) {
        let mut group = self.groups_mut().iter_mut().find(|g| g.tag() == tag);
        if let Some(ref mut group) = group {
            group.attributes_mut().insert(attribute.name().to_owned(), attribute);
        } else {
            let mut new_group = IppAttributeGroup::new(tag);
            new_group
                .attributes_mut()
                .insert(attribute.name().to_owned(), attribute);
            self.groups_mut().push(new_group);
        }
    }

    /// Write attribute list to byte array
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        // first send the header attributes
        buffer.put_u8(DelimiterTag::OperationAttributes as u8);

        if let Some(group) = self.groups_of(DelimiterTag::OperationAttributes).get(0) {
            for hdr in &IppAttribute::HEADER_ATTRS {
                if let Some(attr) = group.attributes().get(*hdr) {
                    buffer.put(attr.to_bytes());
                }
            }
        }

        // now the rest
        for hdr in &[
            DelimiterTag::OperationAttributes,
            DelimiterTag::JobAttributes,
            DelimiterTag::PrinterAttributes,
        ] {
            if let Some(group) = self.groups_of(*hdr).get(0) {
                if group.tag() != DelimiterTag::OperationAttributes {
                    buffer.put_u8(group.tag() as u8);
                }
                for (_, attr) in group
                    .attributes()
                    .iter()
                    .filter(|&(_, v)| group.tag() != DelimiterTag::OperationAttributes || !is_header_attr(v.name()))
                {
                    buffer.put(attr.to_bytes());
                }
            }
        }
        buffer.put_u8(DelimiterTag::EndOfAttributes as u8);

        buffer.freeze()
    }
}
