//!
//! IPP tags

enum_from_primitive! {
#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub enum Tag {
    OperationAttributesTag = 0x01,
    JobAttributesTag = 0x02,
    EndOfAttributesTag = 0x03,
    PrinterAttributesTag = 0x04,
    UnsupportedAttributesTag = 0x05,

    Unsupported = 0x10,
    Unknown = 0x12,
    NoValue = 0x13,
    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,
    OctectStringUnspecified = 0x30,
    DateTime = 0x31,
    Resolution = 0x32,
    RangeOfInteger = 0x33,
    BegCollection = 0x34,
    TextWithLanguage = 0x35,
    NameWithLanguage = 0x36,
    EndCollection = 0x37,
    TextWithoutLanguage = 0x41,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    UriScheme = 0x46,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
    MemberAttrName = 0x4a,
}
}

pub fn is_value_tag(value: u8) -> bool {
    value >= 0x10 && value <= 0x4a
}

pub fn is_delimiter_tag(value: u8) -> bool {
    value >= 0x01 && value <= 0x05
}
