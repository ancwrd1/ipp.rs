//!
//! Attribute-related structs
//!

use std::collections::BTreeMap;

use bytes::{BufMut, Bytes, BytesMut};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    model::DelimiterTag,
    parser::IppParseError,
    value::{IppDateTime, IppName, IppValue},
};

macro_rules! define_attributes {
    ($($name:ident => $value:literal),* $(,)?) => {
        $(pub const $name: &'static str = $value;)*
    };
}

fn is_header_attr(attr: &str) -> bool {
    IppAttribute::HEADER_ATTRS.contains(&attr)
}

/// `IppAttribute` represents an IPP attribute
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct IppAttribute {
    /// Attribute name
    name: IppName,
    /// Attribute value
    value: IppValue,
}

impl IppAttribute {
    define_attributes! {
        ATTRIBUTES_CHARSET => "attributes-charset",
        ATTRIBUTES_NATURAL_LANGUAGE => "attributes-natural-language",
        CHARSET_CONFIGURED => "charset-configured",
        CHARSET_SUPPORTED => "charset-supported",
        COLOR_MODE_SUPPORTED => "color-mode-supported",
        COLOR_SUPPORTED => "color-supported",
        COMPRESSION_SUPPORTED => "compression-supported",
        COPIES => "copies",
        COPIES_DEFAULT => "copies-default",
        COPIES_SUPPORTED => "copies-supported",
        DOCUMENT_FORMAT => "document-format",
        DOCUMENT_FORMAT_DEFAULT => "document-format-default",
        DOCUMENT_FORMAT_PREFERRED => "document-format-preferred",
        DOCUMENT_FORMAT_SUPPORTED => "document-format-supported",
        FINISHINGS => "finishings",
        FINISHINGS_DEFAULT => "finishings-default",
        FINISHINGS_SUPPORTED => "finishings-supported",
        GENERATED_NATURAL_LANGUAGE_SUPPORTED => "generated-natural-language-supported",
        IPP_VERSIONS_SUPPORTED => "ipp-versions-supported",
        JOB_ID => "job-id",
        JOB_NAME => "job-name",
        JOB_STATE => "job-state",
        JOB_STATE_REASONS => "job-state-reasons",
        JOB_URI => "job-uri",
        LAST_DOCUMENT => "last-document",
        MEDIA_COL => "media-col",
        MEDIA_COL_DATABASE => "media-col-database",
        MEDIA_COL_DEFAULT => "media-col-default",
        MEDIA_COL_READY => "media-col-ready",
        MEDIA_COL_SUPPORTED => "media-col-supported",
        MEDIA_DEFAULT => "media-default",
        MEDIA_READY => "media-ready",
        MEDIA_SOURCE_SUPPORTED => "media-source-supported",
        MEDIA_SUPPORTED => "media-supported",
        MEDIA_TYPE_SUPPORTED => "media-type-supported",
        MOPRIA_CERTIFIED => "mopria-certified",
        MULTIPLE_DOCUMENT_HANDLING => "multiple-document-handling",
        MULTIPLE_DOCUMENT_HANDLING_DEFAULT => "multiple-document-handling-default",
        MULTIPLE_DOCUMENT_HANDLING_SUPPORTED => "multiple-document-handling-supported",
        NATURAL_LANGUAGE_CONFIGURED => "natural-language-configured",
        OPERATIONS_SUPPORTED => "operations-supported",
        ORIENTATION_REQUESTED => "orientation-requested",
        ORIENTATION_REQUESTED_DEFAULT => "orientation-requested-default",
        ORIENTATION_REQUESTED_SUPPORTED => "orientation-requested-supported",
        OUTPUT_BIN => "output-bin",
        OUTPUT_BIN_DEFAULT => "output-bin-default",
        OUTPUT_BIN_SUPPORTED => "output-bin-supported",
        OUTPUT_MODE_SUPPORTED => "output-mode-supported",
        PAGES_PER_MINUTE => "pages-per-minute",
        PDL_OVERRIDE_SUPPORTED => "pdl-override-supported",
        PRINTER_DEVICE_ID => "printer-device-id",
        PRINTER_FIRMWARE_NAME => "printer-firmware-name",
        PRINTER_FIRMWARE_STRING_VERSION => "printer-firmware-string-version",
        PRINTER_INFO => "printer-info",
        PRINTER_IS_ACCEPTING_JOBS => "printer-is-accepting-jobs",
        PRINTER_LOCATION => "printer-location",
        PRINTER_MAKE_AND_MODEL => "printer-make-and-model",
        PRINTER_MORE_INFO => "printer-more-info",
        PRINTER_NAME => "printer-name",
        PRINTER_RESOLUTION => "printer-resolution",
        PRINTER_RESOLUTION_DEFAULT => "printer-resolution-default",
        PRINTER_RESOLUTION_SUPPORTED => "printer-resolution-supported",
        PRINTER_STATE => "printer-state",
        PRINTER_STATE_MESSAGE => "printer-state-message",
        PRINTER_STATE_REASONS => "printer-state-reasons",
        PRINTER_UP_TIME => "printer-up-time",
        PRINTER_URI => "printer-uri",
        PRINTER_URI_SUPPORTED => "printer-uri-supported",
        PRINTER_UUID => "printer-uuid",
        PRINT_COLOR_MODE => "print-color-mode",
        PRINT_COLOR_MODE_DEFAULT => "print-color-mode-default",
        PRINT_COLOR_MODE_SUPPORTED => "print-color-mode-supported",
        PRINT_QUALITY => "print-quality",
        PRINT_QUALITY_DEFAULT => "print-quality-default",
        PRINT_QUALITY_SUPPORTED => "print-quality-supported",
        QUEUED_JOB_COUNT => "queued-job-count",
        REQUESTED_ATTRIBUTES => "requested-attributes",
        REQUESTING_USER_NAME => "requesting-user-name",
        SIDES => "sides",
        SIDES_DEFAULT => "sides-default",
        SIDES_SUPPORTED => "sides-supported",
        STATUS_MESSAGE => "status-message",
        URI_AUTHENTICATION_SUPPORTED => "uri-authentication-supported",
        URI_SECURITY_SUPPORTED => "uri-security-supported",
    }

    // Per section 4.1.4. Character Set and Natural Language Operation Attributes
    // The "attributes-charset" and "attributes-natural-language" attributes MUST be the first two attributes
    // in every IPP request and response, as part of the initial Operation Attributes group of the IPP message
    // Per section 4.1.5 Operation targets
    // o  In the case where there is only one operation target attribute
    //    (i.e., either only the "printer-uri" attribute or only the
    //    "job-uri" attribute), that attribute MUST be the third attribute
    //    in the Operation Attributes group.
    // o  In the case where Job operations use two operation target
    //    attributes (i.e., the "printer-uri" and "job-id" attributes), the
    //    "printer-uri" attribute MUST be the third attribute and the
    //    "job-id" attribute MUST be the fourth attribute.
    const HEADER_ATTRS: [&'static str; 4] = [
        IppAttribute::ATTRIBUTES_CHARSET,
        IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE,
        IppAttribute::PRINTER_URI,
        IppAttribute::JOB_ID,
    ];

    /// Create a new instance of the attribute
    ///
    /// * `name` - Attribute name<br/>
    /// * `value` - Attribute value<br/>
    pub fn new(name: IppName, value: IppValue) -> IppAttribute {
        IppAttribute { name, value }
    }

    /// Create a new instance of the attribute
    ///
    /// * `name` - Attribute name<br/>
    /// * `value` - Attribute value<br/>
    pub fn with_name<S>(name: S, value: IppValue) -> Result<IppAttribute, IppParseError>
    where
        S: AsRef<str>,
    {
        Ok(IppAttribute {
            name: name.as_ref().try_into()?,
            value,
        })
    }

    /// Return the attribute name
    pub fn name(&self) -> &IppName {
        &self.name
    }

    /// Return the attribute value
    pub fn value(&self) -> &IppValue {
        &self.value
    }

    /// Consume this attribute and return the value
    pub fn into_value(self) -> IppValue {
        self.value
    }

    /// Write the attribute to a byte array
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct IppAttributeGroup {
    tag: DelimiterTag,
    attributes: Vec<IppAttribute>,
}

impl IppAttributeGroup {
    /// Create a new attribute group of a given type
    pub fn new(tag: DelimiterTag) -> IppAttributeGroup {
        IppAttributeGroup {
            tag,
            attributes: Vec::new(),
        }
    }

    /// Return the group type tag
    pub fn tag(&self) -> DelimiterTag {
        self.tag
    }

    /// Return the list of attributes
    pub fn attributes(&self) -> &[IppAttribute] {
        &self.attributes
    }

    /// Return the mutable list of attributes
    pub fn attributes_mut(&mut self) -> &mut Vec<IppAttribute> {
        &mut self.attributes
    }

    /// Consume this group and return the mutable attributes
    pub fn into_attributes(self) -> Vec<IppAttribute> {
        self.attributes
    }

    pub fn get(&self, name: &str) -> Option<&IppAttribute> {
        self.attributes.iter().find(|attr| attr.name().as_str() == name)
    }
}

impl IntoIterator for IppAttributeGroup {
    type Item = IppAttribute;
    type IntoIter = <Vec<IppAttribute> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_iter()
    }
}

impl<'a> IntoIterator for &'a IppAttributeGroup {
    type Item = &'a IppAttribute;
    type IntoIter = std::slice::Iter<'a, IppAttribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

impl<'a> IntoIterator for &'a mut IppAttributeGroup {
    type Item = &'a mut IppAttribute;
    type IntoIter = std::slice::IterMut<'a, IppAttribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter_mut()
    }
}

/// Attribute list
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct IppAttributes {
    groups: Vec<IppAttributeGroup>,
}

impl IppAttributes {
    /// Create an attribute list
    pub fn new() -> IppAttributes {
        IppAttributes { ..Default::default() }
    }

    /// Get all groups
    pub fn groups(&self) -> &[IppAttributeGroup] {
        &self.groups
    }

    /// Get all mutable groups
    pub fn groups_mut(&mut self) -> &mut Vec<IppAttributeGroup> {
        &mut self.groups
    }

    /// Consume this attribute list and return all attribute groups
    pub fn into_groups(self) -> Vec<IppAttributeGroup> {
        self.groups
    }

    /// Get a list of attribute groups matching a given delimiter tag
    pub fn groups_of(&self, tag: DelimiterTag) -> impl Iterator<Item = &IppAttributeGroup> {
        self.groups.iter().filter(move |g| g.tag == tag)
    }

    /// Add an attribute to a given group
    pub fn add(&mut self, tag: DelimiterTag, attribute: IppAttribute) {
        let group = self.groups_mut().iter_mut().find(|g| g.tag() == tag);
        if let Some(group) = group {
            group.attributes.push(attribute);
        } else {
            let mut new_group = IppAttributeGroup::new(tag);
            new_group.attributes.push(attribute);
            self.groups_mut().push(new_group);
        }
    }

    /// Replace the contents of the first `IppAttributeGroup` if found, otherwise create it
    pub fn set_or_replace(&mut self, tag: DelimiterTag, attributes: Vec<IppAttribute>) {
        let group = self.groups_mut().iter_mut().find(|g| g.tag() == tag);
        if let Some(group) = group {
            group.attributes = attributes;
        } else {
            let mut new_group = IppAttributeGroup::new(tag);
            new_group.attributes = attributes;
            self.groups_mut().push(new_group);
        }
    }

    /// Write the attribute list to a byte array
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        // put the required attributes first as described in section 4.1.4 of RFC8011
        buffer.put_u8(DelimiterTag::OperationAttributes as u8);

        if let Some(group) = self.groups_of(DelimiterTag::OperationAttributes).next() {
            let mut header_slots: [Option<&IppAttribute>; IppAttribute::HEADER_ATTRS.len()] =
                [None; IppAttribute::HEADER_ATTRS.len()];
            for attr in &group.attributes {
                if let Some(idx) = IppAttribute::HEADER_ATTRS
                    .iter()
                    .position(|h| *h == attr.name().as_str())
                {
                    header_slots[idx] = Some(attr);
                }
            }
            for attr in header_slots.into_iter().flatten() {
                buffer.put(attr.to_bytes());
            }

            // then everything else, in original order
            for attr in &group.attributes {
                if !is_header_attr(attr.name()) {
                    buffer.put(attr.to_bytes());
                }
            }
        }

        // now the rest
        for group in self
            .groups()
            .iter()
            .filter(|group| group.tag() != DelimiterTag::OperationAttributes)
        {
            buffer.put_u8(group.tag() as u8);

            for attr in group.attributes() {
                buffer.put(attr.to_bytes());
            }
        }
        buffer.put_u8(DelimiterTag::EndOfAttributes as u8);

        buffer.freeze()
    }
}

/// Util trait to chain `IppAttribute` construction after a faillible or infaillible `IppValue` construction
/// The trait is also implemented on types which have an unambiguous conversion to `IppValue`
///
/// ```
/// let job_id = IppValue::new_integer(1).with_name(IppAttribute::JOB_ID);
/// let printer_uri = IppString::new_uri("ipp://localhost").with_name(IppAttribute::PRINTER_URI);
/// let ipp_array = Vec::new().with_name("some-name");
/// ```
pub trait IppAttrWithName {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError>;
}

impl IppAttrWithName for IppValue {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppName::new(name).map(|name| IppAttribute::new(name, self))
    }
}

impl IppAttrWithName for Result<IppValue, IppParseError> {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        self?.with_name(name)
    }
}

impl IppAttrWithName for Vec<IppValue> {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppValue::Array(self).with_name(name)
    }
}

impl IppAttrWithName for BTreeMap<IppName, IppValue> {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppValue::Collection(self).with_name(name)
    }
}

impl IppAttrWithName for IppDateTime {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppValue::DateTime(self).with_name(name)
    }
}
