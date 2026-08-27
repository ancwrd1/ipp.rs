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
    value::{BoundedStrLiteral, IppDateTime, IppName, IppValue},
};

macro_rules! define_attributes {
    ($($name:ident => $value:literal),* $(,)?) => {
        $(pub const $name: IppAttributeName = IppAttributeName::const_new($value);)*
    };
}

fn is_header_attr(attr: &str) -> bool {
    IppAttribute::HEADER_ATTRS.contains(&attr)
}

pub type IppAttributeName = BoundedStrLiteral<255>;

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
    // Attributes defined in RFC 8011
    define_attributes! {
        ATTRIBUTES_CHARSET => "attributes-charset",
        ATTRIBUTES_NATURAL_LANGUAGE => "attributes-natural-language",
        CHARSET_CONFIGURED => "charset-configured",
        CHARSET_SUPPORTED => "charset-supported",
        COLOR_SUPPORTED => "color-supported",
        COMPRESSION => "compression",
        COMPRESSION_SUPPORTED => "compression-supported",
        COPIES => "copies",
        COPIES_DEFAULT => "copies-default",
        COPIES_SUPPORTED => "copies-supported",
        COVER_BACK => "cover-back",
        COVER_FRONT => "cover-front",
        DATE_TIME_AT_COMPLETED => "date-time-at-completed",
        DATE_TIME_AT_CREATION => "date-time-at-creation",
        DATE_TIME_AT_PROCESSING => "date-time-at-processing",
        DETAILED_STATUS_MESSAGE => "detailed-status-message",
        DOCUMENT_ACCESS_ERROR => "document-access-error",
        DOCUMENT_FORMAT => "document-format",
        DOCUMENT_FORMAT_DEFAULT => "document-format-default",
        DOCUMENT_FORMAT_SUPPORTED => "document-format-supported",
        DOCUMENT_NAME => "document-name",
        DOCUMENT_NATURAL_LANGUAGE => "document-natural-language",
        DOCUMENT_URI => "document-uri",
        FINISHINGS => "finishings",
        FINISHINGS_DEFAULT => "finishings-default",
        FINISHINGS_SUPPORTED => "finishings-supported",
        GENERATED_NATURAL_LANGUAGE_SUPPORTED => "generated-natural-language-supported",
        IPP_ATTRIBUTE_FIDELITY => "ipp-attribute-fidelity",
        IPP_VERSIONS_SUPPORTED => "ipp-versions-supported",
        JOB_DETAILED_STATUS_MESSAGES => "job-detailed-status-messages",
        JOB_DOCUMENT_ACCESS_ERRORS => "job-document-access-errors",
        JOB_HOLD_UNTIL => "job-hold-until",
        JOB_HOLD_UNTIL_DEFAULT => "job-hold-until-default",
        JOB_HOLD_UNTIL_SUPPORTED => "job-hold-until-supported",
        JOB_ID => "job-id",
        JOB_IMPRESSIONS => "job-impressions",
        JOB_IMPRESSIONS_COMPLETED => "job-impressions-completed",
        JOB_IMPRESSIONS_SUPPORTED => "job-impressions-supported",
        JOB_K_OCTETS => "job-k-octets",
        JOB_K_OCTETS_PROCESSED => "job-k-octets-processed",
        JOB_K_OCTETS_SUPPORTED => "job-k-octets-supported",
        JOB_MEDIA_SHEETS => "job-media-sheets",
        JOB_MEDIA_SHEETS_COMPLETED => "job-media-sheets-completed",
        JOB_MEDIA_SHEETS_SUPPORTED => "job-media-sheets-supported",
        JOB_MESSAGE_FROM_OPERATOR => "job-message-from-operator",
        JOB_MORE_INFO => "job-more-info",
        JOB_NAME => "job-name",
        JOB_ORIGINATING_USER_NAME => "job-originating-user-name",
        JOB_PRINTER_UP_TIME => "job-printer-up-time",
        JOB_PRINTER_URI => "job-printer-uri",
        JOB_PRIORITY => "job-priority",
        JOB_PRIORITY_DEFAULT => "job-priority-default",
        JOB_PRIORITY_SUPPORTED => "job-priority-supported",
        JOB_SHEETS => "job-sheets",
        JOB_SHEETS_DEFAULT => "job-sheets-default",
        JOB_SHEETS_SUPPORTED => "job-sheets-supported",
        JOB_STATE => "job-state",
        JOB_STATE_MESSAGE => "job-state-message",
        JOB_STATE_REASONS => "job-state-reasons",
        JOB_URI => "job-uri",
        LAST_DOCUMENT => "last-document",
        LIMIT => "limit",
        MEDIA => "media",
        MEDIA_DEFAULT => "media-default",
        MEDIA_READY => "media-ready",
        MEDIA_SUPPORTED => "media-supported",
        MESSAGE => "message",
        MULTIPLE_DOCUMENT_HANDLING => "multiple-document-handling",
        MULTIPLE_DOCUMENT_HANDLING_DEFAULT => "multiple-document-handling-default",
        MULTIPLE_DOCUMENT_HANDLING_SUPPORTED => "multiple-document-handling-supported",
        MULTIPLE_DOCUMENT_JOBS_SUPPORTED => "multiple-document-jobs-supported",
        MULTIPLE_OPERATION_TIME_OUT => "multiple-operation-time-out",
        MY_JOBS => "my-jobs",
        NATURAL_LANGUAGE_CONFIGURED => "natural-language-configured",
        NUMBER_OF_DOCUMENTS => "number-of-documents",
        NUMBER_OF_INTERVENING_JOBS => "number-of-intervening-jobs",
        NUMBER_UP => "number-up",
        NUMBER_UP_DEFAULT => "number-up-default",
        NUMBER_UP_SUPPORTED => "number-up-supported",
        OPERATIONS_SUPPORTED => "operations-supported",
        ORIENTATION_REQUESTED => "orientation-requested",
        ORIENTATION_REQUESTED_DEFAULT => "orientation-requested-default",
        ORIENTATION_REQUESTED_SUPPORTED => "orientation-requested-supported",
        OUTPUT_DEVICE_ASSIGNED => "output-device-assigned",
        PAGE_RANGES => "page-ranges",
        PAGE_RANGES_SUPPORTED => "page-ranges-supported",
        PAGES_PER_MINUTE => "pages-per-minute",
        PAGES_PER_MINUTE_COLOR => "pages-per-minute-color",
        PDL_OVERRIDE_SUPPORTED => "pdl-override-supported",
        PRINT_QUALITY => "print-quality",
        PRINT_QUALITY_DEFAULT => "print-quality-default",
        PRINT_QUALITY_SUPPORTED => "print-quality-supported",
        PRINTER_CURRENT_TIME => "printer-current-time",
        PRINTER_DRIVER_INSTALLER => "printer-driver-installer",
        PRINTER_INFO => "printer-info",
        PRINTER_IS_ACCEPTING_JOBS => "printer-is-accepting-jobs",
        PRINTER_LOCATION => "printer-location",
        PRINTER_MAKE_AND_MODEL => "printer-make-and-model",
        PRINTER_MESSAGE_FROM_OPERATOR => "printer-message-from-operator",
        PRINTER_MORE_INFO => "printer-more-info",
        PRINTER_MORE_INFO_MANUFACTURER => "printer-more-info-manufacturer",
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
        QUEUED_JOB_COUNT => "queued-job-count",
        REFERENCE_URI_SCHEMES_SUPPORTED => "reference-uri-schemes-supported",
        REQUESTED_ATTRIBUTES => "requested-attributes",
        REQUESTING_USER_NAME => "requesting-user-name",
        SEPARATOR_SHEETS => "separator-sheets",
        SIDES => "sides",
        SIDES_DEFAULT => "sides-default",
        SIDES_SUPPORTED => "sides-supported",
        STATUS_MESSAGE => "status-message",
        TIME_AT_COMPLETED => "time-at-completed",
        TIME_AT_CREATION => "time-at-creation",
        TIME_AT_PROCESSING => "time-at-processing",
        URI_AUTHENTICATION_SUPPORTED => "uri-authentication-supported",
        URI_SECURITY_SUPPORTED => "uri-security-supported",
        WHICH_JOBS => "which-jobs"
    }

    // Special attribute groups defined in 4.2.5 and 4.3.4 of RFC 8011
    // can be used in "get-printer-attributes" or "get-job-attributes" operations to obtain several
    // attributes at once
    define_attributes! {
        ALL => "all",
        JOB_DESCRIPTION => "job-description",
        JOB_TEMPLATE => "job-template",
        PRINTER_DESCRIPTION => "printer-description",
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
        IppAttribute::ATTRIBUTES_CHARSET.inner,
        IppAttribute::ATTRIBUTES_NATURAL_LANGUAGE.inner,
        IppAttribute::PRINTER_URI.inner,
        IppAttribute::JOB_ID.inner,
    ];

    /// Create a new instance of the attribute
    ///
    /// * `name` - Attribute name<br/>
    /// * `value` - Attribute value<br/>
    pub fn new(name: impl Into<IppName>, value: IppValue) -> IppAttribute {
        IppAttribute {
            name: name.into(),
            value,
        }
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

    /// Get a list of mutable attribute groups matching a given delimiter tag
    pub fn groups_of_mut(&mut self, tag: DelimiterTag) -> impl Iterator<Item = &mut IppAttributeGroup> {
        self.groups.iter_mut().filter(move |g| g.tag == tag)
    }

    /// Get the first group that matches the delimiter
    pub fn first_of(&self, tag: DelimiterTag) -> Option<&IppAttributeGroup> {
        self.groups_of(tag).next()
    }

    /// Get a mutable access to the first group that matches the delimiter
    pub fn first_of_mut(&mut self, tag: DelimiterTag) -> Option<&mut IppAttributeGroup> {
        self.groups_of_mut(tag).next()
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
/// use ipp::attribute::*;
/// use ipp::value::*;
/// let job_id = IppValue::new_integer(1).with_name(IppAttribute::JOB_ID);
/// let printer_uri = IppValue::new_uri("ipp://localhost").with_name(IppAttribute::PRINTER_URI);
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

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> IppAttrWithName for chrono::DateTime<Tz> {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppDateTime::from(self).with_name(name)
    }
}

#[cfg(feature = "jiff")]
impl IppAttrWithName for jiff::Zoned {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppDateTime::from(self).with_name(name)
    }
}

#[cfg(feature = "jiff")]
impl IppAttrWithName for &jiff::Zoned {
    fn with_name<S: Into<String>>(self, name: S) -> Result<IppAttribute, IppParseError> {
        IppDateTime::from(self).with_name(name)
    }
}
