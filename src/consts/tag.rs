//!
//! IPP tags
//!
pub const OPERATION_ATTRIBUTES_TAG: u8 = 0x01;
pub const JOB_ATTRIBUTES_TAG: u8 = 0x02;
pub const END_OF_ATTRIBUTES_TAG: u8 = 0x03;
pub const PRINTER_ATTRIBUTES_TAG: u8 = 0x04;
pub const UNSUPPORTED_ATTRIBUTES_TAG: u8 = 0x05;

pub const UNSUPPORTED: u8 = 0x10;
pub const UNKNOWN: u8 = 0x12;
pub const NO_VALUE: u8 = 0x13;
pub const INTEGER: u8 = 0x21;
pub const BOOLEAN: u8 = 0x22;
pub const ENUM: u8 = 0x23;
pub const OCTECTSTRING_UNSPECIFIED: u8 = 0x30;
pub const DATETIME: u8 = 0x31;
pub const RESOLUTION: u8 = 0x32;
pub const RANGEOFINTEGER: u8 = 0x33;
pub const BEG_COLLECTION: u8 = 0x34;
pub const TEXT_WITH_LANGUAGE: u8 = 0x35;
pub const NAME_WITH_LANGUAGE: u8 = 0x36;
pub const END_COLLECTION: u8 = 0x37;
pub const TEXT_WITHOUT_LANGUAGE: u8 = 0x41;
pub const NAME_WITHOUT_LANGUAGE: u8 = 0x42;
pub const KEYWORD: u8 = 0x44;
pub const URI: u8 = 0x45;
pub const URI_SCHEME: u8 = 0x46;
pub const CHARSET: u8 = 0x47;
pub const NATURAL_LANGUAGE: u8 = 0x48;
pub const MIME_MEDIA_TYPE: u8 = 0x49;
pub const MEMBER_ATTR_NAME: u8 = 0x4a;

pub fn is_value_tag(value: u8) -> bool {
    value >= 0x10 && value <= 0x4a
}

pub fn is_delimiter_tag(value: u8) -> bool {
    value >= 0x01 && value <= 0x05
}
