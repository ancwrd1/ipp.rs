//!
//! IPP value
//!
#![allow(unused_assignments)]
use std::{borrow::Cow, collections::BTreeMap, fmt, ops::Deref, str::FromStr};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use enum_as_inner::EnumAsInner;
use http::Uri;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FromPrimitive as _, model::ValueTag, parser::IppParseError};

const IPP_STRING_MAX_LENGTH: usize = 1023;

/// A UTF-8 string whose length is bounded by a compile-time maximum (in bytes).
///
/// This type is primarily used to enforce IPP `text(*)`, `name(*)`,
/// `keyword`, and related value length limits defined by the IPP specification.
///
/// The length constraint is measured in UTF-8 encoded bytes,
/// not Unicode scalar values.
///
/// # Type Parameter
/// - `MAX`: Maximum allowed length in bytes.
///
/// # Errors
/// Returns [`IppParseError::InvalidStringLength`] if the input exceeds `MAX`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedString<const MAX: usize = IPP_STRING_MAX_LENGTH> {
    inner: String,
}

/// IPP string value with a maximum length of 1023 bytes
pub type IppString = BoundedString;
/// IPP short string value with a maximum length of 127 bytes
pub type IppShortString = BoundedString<127>;
/// IPP keyword value with a maximum length of 255 bytes
pub type IppKeyword = BoundedString<255>;
/// IPP MIME media type value with a maximum length of 255 bytes
pub type IppMimeMediaType = BoundedString<255>;
/// IPP charset value with a maximum length of 63 bytes
pub type IppCharset = BoundedString<63>;
/// IPP natural language tag with a maximum length of 63 bytes
pub type IppLanguage = BoundedString<63>;
/// IPP name value with a maximum length of 255 bytes
pub type IppName = BoundedString<255>;

impl<const MAX: usize> BoundedString<MAX> {
    /// Attempts to create a bounded string from the given value, returning an error if the string's length exceeds the const generic
    /// defined for the type.
    pub fn new(s: impl Into<String>) -> Result<Self, IppParseError> {
        let s = s.into();
        let len = s.len();

        if len > MAX {
            return Err(IppParseError::InvalidStringLength { len, max: MAX });
        }

        Ok(Self { inner: s })
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() <= MAX
            && let Ok(s) = str::from_utf8(data)
        {
            Some(Self { inner: s.to_owned() })
        } else {
            None
        }
    }

    /// Return the maximum allowed length in bytes
    pub const fn max() -> usize {
        MAX
    }

    /// Return the string as a `str` reference
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Consume the bounded string and return the inner `String`
    pub fn into_inner(self) -> String {
        self.inner
    }

    /// Return the length of the string in bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return true if the string is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Widen the max size of the bounded string.
    /// Infallible because all strings of length <= MAX are valid for any larger MAX2.
    /// if attempting to expand to a smaller `MAX2` the assertion will fail causing a panic.
    pub fn expand<const MAX2: usize>(self) -> BoundedString<MAX2> {
        assert!(MAX2 >= MAX);
        BoundedString::<MAX2> { inner: self.inner }
    }

    /// Attempt to shrink a bounded string to a smaller MAX.
    /// Returns an error if the actual string is too long for the target size.
    pub fn shrink<const MAX2: usize>(self) -> Result<BoundedString<MAX2>, IppParseError> {
        if self.len() > MAX2 {
            return Err(IppParseError::InvalidStringLength {
                len: self.len(),
                max: MAX2,
            });
        }
        Ok(BoundedString::<MAX2> { inner: self.inner })
    }
}

impl<const MAX: usize> From<BoundedString<MAX>> for String {
    fn from(value: BoundedString<MAX>) -> Self {
        value.inner
    }
}

impl<const MAX: usize> std::borrow::Borrow<str> for BoundedString<MAX> {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl<const MAX: usize> AsRef<str> for BoundedString<MAX> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<const MAX: usize> Deref for BoundedString<MAX> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const MAX: usize> FromStr for BoundedString<MAX> {
    type Err = IppParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<const MAX: usize> TryFrom<&str> for BoundedString<MAX> {
    type Error = IppParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<const MAX: usize> TryFrom<String> for BoundedString<MAX> {
    type Error = IppParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<const MAX: usize> TryFrom<Cow<'_, str>> for BoundedString<MAX> {
    type Error = IppParseError;
    fn try_from(s: Cow<'_, str>) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<const MAX: usize> fmt::Display for BoundedString<MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const MAX: usize> TryFrom<Uri> for BoundedString<MAX> {
    type Error = IppParseError;
    fn try_from(u: Uri) -> Result<Self, Self::Error> {
        u.to_string().try_into()
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> Serialize for BoundedString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> Deserialize<'de> for BoundedString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = String::deserialize(deserializer)?;
        if inner.len() > N {
            Err(serde::de::Error::invalid_length(
                inner.len(),
                &format!("no more than {N} bytes").as_str(),
            ))
        } else {
            Ok(Self { inner })
        }
    }
}

/// Represents an IPP `text(*)` value with length-tiered encoding.
///
/// IPP defines multiple text encodings depending on maximum length:
/// - 0–127 bytes
/// - 128–255 bytes
/// - 256–1023 bytes
///
/// This enum selects the smallest valid representation automatically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IppTextValue {
    Short(IppShortString),
    Medium(BoundedString<255>),
    Long(IppString),
}

impl IppTextValue {
    /// Create an `IppTextValue`, selecting the smallest valid encoding tier for the given string
    pub fn new(s: impl Into<String>) -> Result<Self, IppParseError> {
        let string = s.into();
        let len = string.len();
        match len {
            0..=127 => Ok(Self::Short(IppShortString::new(string)?)),
            128..=255 => Ok(Self::Medium(BoundedString::<255>::new(string)?)),
            256..=IPP_STRING_MAX_LENGTH => Ok(Self::Long(IppString::new(string)?)),
            _ => Err(IppParseError::InvalidStringLength {
                len,
                max: IPP_STRING_MAX_LENGTH,
            }),
        }
    }

    /// Return the length of the string in bytes
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Return true if the string is empty
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    #[must_use]
    /// Returns the same string with a smaller encoding tier if possible
    /// Otherwise returns the original value unchanged.
    pub fn shrink(self) -> Self {
        match self {
            // Unwrap is OK since we check the size
            Self::Medium(inner) if inner.len() <= 127 => Self::Short(inner.shrink().unwrap()),
            Self::Long(inner) if inner.len() <= 127 => Self::Short(inner.shrink().unwrap()),
            Self::Long(inner) if inner.len() <= 255 => Self::Medium(inner.shrink().unwrap()),
            _ => self,
        }
    }
}

impl From<IppShortString> for IppTextValue {
    fn from(value: IppShortString) -> Self {
        Self::Short(value)
    }
}

impl From<BoundedString<255>> for IppTextValue {
    fn from(value: BoundedString<255>) -> Self {
        Self::Medium(value)
    }
}

impl From<IppString> for IppTextValue {
    fn from(value: IppString) -> Self {
        Self::Long(value)
    }
}

impl AsRef<str> for IppTextValue {
    fn as_ref(&self) -> &str {
        match self {
            IppTextValue::Short(s) => s.as_ref(),
            IppTextValue::Medium(s) => s.as_ref(),
            IppTextValue::Long(s) => s.as_ref(),
        }
    }
}

impl Deref for IppTextValue {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl TryFrom<&str> for IppTextValue {
    type Error = IppParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<String> for IppTextValue {
    type Error = IppParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<Cow<'_, str>> for IppTextValue {
    type Error = IppParseError;
    fn try_from(s: Cow<'_, str>) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl fmt::Display for IppTextValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[inline]
fn get_len_string(data: &mut Bytes) -> Result<String, IppParseError> {
    let len = data.get_u16() as usize;
    if let Ok(s) = str::from_utf8(&data[0..len]) {
        let res = s.to_owned();
        data.advance(len);
        Ok(res)
    } else {
        Err(IppParseError::UnsupportedCharset)
    }
}

/// Represents IPP datetime value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IppDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub deci_seconds: u8,
    pub utc_dir: char,
    pub utc_hours: u8,
    pub utc_mins: u8,
}

impl IppDateTime {
    pub fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_u16(self.year);
        bytes.put_u8(self.month);
        bytes.put_u8(self.day);
        bytes.put_u8(self.hour);
        bytes.put_u8(self.minutes);
        bytes.put_u8(self.seconds);
        bytes.put_u8(self.deci_seconds);
        bytes.put_u8(self.utc_dir as u8);
        bytes.put_u8(self.utc_hours);
        bytes.put_u8(self.utc_mins);
        bytes.freeze()
    }
}

impl fmt::Display for IppDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            year,
            month,
            day,
            hour,
            minutes,
            seconds,
            deci_seconds,
            utc_dir,
            utc_hours,
            ..
        } = self;
        write!(
            f,
            "{year}-{month}-{day},{hour}:{minutes}:{seconds}.{deci_seconds},{utc_dir}{utc_hours}utc"
        )
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> From<chrono::DateTime<Tz>> for IppDateTime {
    fn from(value: chrono::DateTime<Tz>) -> Self {
        use chrono::{Datelike, Offset, Timelike};

        let offset = value.offset().fix().local_minus_utc();

        Self {
            year: value.year() as u16,
            month: value.month() as u8,
            day: value.day() as u8,
            hour: value.hour() as u8,
            minutes: value.minute() as u8,
            seconds: value.second() as u8,
            deci_seconds: (value.nanosecond() / 100_000_000) as u8,
            utc_dir: if offset < 0 { '-' } else { '+' },
            utc_hours: (offset.abs() / 3600) as u8,
            utc_mins: (offset.abs() % 3600 / 60) as u8,
        }
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<IppDateTime> for chrono::DateTime<chrono::FixedOffset> {
    type Error = crate::error::IppError;

    fn try_from(value: IppDateTime) -> Result<Self, Self::Error> {
        use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

        let err = || IppParseError::InvalidDateTime;

        let seconds = value.utc_hours as i32 * 3600 + value.utc_mins as i32 * 60;
        let offset = if value.utc_dir == '-' {
            FixedOffset::west_opt(seconds)
        } else if value.utc_dir == '+' {
            FixedOffset::east_opt(seconds)
        } else {
            None
        }
        .ok_or_else(err)?;

        let datetime = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(value.year as _, value.month as _, value.day as _).ok_or_else(err)?,
            NaiveTime::from_hms_milli_opt(
                value.hour as _,
                value.minutes as _,
                value.seconds as _,
                value.deci_seconds as u32 * 100,
            )
            .ok_or_else(err)?,
        );
        Ok(offset.from_local_datetime(&datetime).single().ok_or_else(err)?)
    }
}

/// IPP attribute values as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010)
/// the length for TextWithoutLanguage, TextWithLanguage, and OctetString values is heavily attribute dependent
/// usual values are 127, 255, and 1023 however as these are attribute dependent, a [`IppTextValue`] is used to allow the calling routine to assert expected text length.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumAsInner)]
pub enum IppValue {
    Integer(i32),
    Enum(i32),
    OctetString(Bytes),
    TextWithoutLanguage(IppTextValue),
    NameWithoutLanguage(IppName),
    TextWithLanguage {
        language: IppLanguage,
        text: IppTextValue,
    },
    NameWithLanguage {
        language: IppLanguage,
        name: IppName,
    },
    Charset(IppCharset),
    NaturalLanguage(IppLanguage),
    Uri(IppString),
    UriScheme(IppString),
    RangeOfInteger {
        min: i32,
        max: i32,
    },
    Boolean(bool),
    Keyword(IppKeyword),
    Array(Vec<IppValue>),
    Collection(BTreeMap<IppName, IppValue>),
    MimeMediaType(IppMimeMediaType),
    DateTime(IppDateTime),
    MemberAttrName(IppKeyword),
    Resolution {
        cross_feed: i32,
        feed: i32,
        units: i8,
    },
    NoValue,
    Other {
        tag: u8,
        data: Bytes,
    },
    /// Fallback type for string type values that do not contain valid utf-8 data
    NonUtf8 {
        tag: ValueTag,
        data: Bytes,
    },
}

impl IppValue {
    pub fn new_integer(value: i32) -> Self {
        Self::Integer(value)
    }

    pub fn new_enum(value: i32) -> Self {
        Self::Enum(value)
    }

    pub fn new_octet_string(data: Bytes) -> Self {
        Self::OctetString(data)
    }

    pub fn new_text_without_language(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppTextValue::new(value).map(Self::TextWithoutLanguage)
    }

    pub fn new_name_without_language(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppName::new(value).map(Self::NameWithoutLanguage)
    }

    pub fn new_text_with_language(language: impl Into<String>, text: impl Into<String>) -> Result<Self, IppParseError> {
        Ok(Self::TextWithLanguage {
            language: IppLanguage::new(language)?,
            text: IppTextValue::new(text)?,
        })
    }

    pub fn new_name_with_language(language: impl Into<String>, name: impl Into<String>) -> Result<Self, IppParseError> {
        Ok(Self::NameWithLanguage {
            language: IppLanguage::new(language)?,
            name: IppName::new(name)?,
        })
    }

    pub fn new_charset(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppCharset::new(value).map(Self::Charset)
    }

    pub fn new_natural_language(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppLanguage::new(value).map(Self::NaturalLanguage)
    }

    pub fn new_uri(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppString::new(value).map(Self::Uri)
    }

    pub fn new_uri_scheme(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppString::new(value).map(Self::UriScheme)
    }

    pub fn new_range_of_integer(min: i32, max: i32) -> Self {
        Self::RangeOfInteger { min, max }
    }

    pub fn new_boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub fn new_keyword(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppKeyword::new(value).map(Self::Keyword)
    }

    pub fn new_array() -> Self {
        Self::Array(Vec::new())
    }

    pub fn new_collection() -> Self {
        Self::Collection(BTreeMap::new())
    }

    pub fn new_mime_media_type(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppMimeMediaType::new(value).map(Self::MimeMediaType)
    }

    pub fn new_datetime(datetime: IppDateTime) -> Self {
        Self::DateTime(datetime)
    }

    pub fn new_member_attr_name(value: impl Into<String>) -> Result<Self, IppParseError> {
        IppKeyword::new(value).map(Self::MemberAttrName)
    }

    pub fn new_resolution(cross_feed: i32, feed: i32, units: i8) -> Self {
        Self::Resolution {
            cross_feed,
            feed,
            units,
        }
    }

    pub fn new_no_value() -> Self {
        Self::NoValue
    }

    pub fn new_other(tag: u8, data: Bytes) -> Self {
        Self::Other { tag, data }
    }

    pub fn new_non_utf8(tag: ValueTag, data: Bytes) -> Self {
        Self::NonUtf8 { tag, data }
    }

    /// Convert to a binary tag
    pub fn to_tag(&self) -> u8 {
        match *self {
            IppValue::Integer(_) => ValueTag::Integer as u8,
            IppValue::Enum(_) => ValueTag::Enum as u8,
            IppValue::RangeOfInteger { .. } => ValueTag::RangeOfInteger as u8,
            IppValue::Boolean(_) => ValueTag::Boolean as u8,
            IppValue::Keyword(_) => ValueTag::Keyword as u8,
            IppValue::OctetString(_) => ValueTag::OctetStringUnspecified as u8,
            IppValue::TextWithoutLanguage(_) => ValueTag::TextWithoutLanguage as u8,
            IppValue::NameWithoutLanguage(_) => ValueTag::NameWithoutLanguage as u8,
            IppValue::TextWithLanguage { .. } => ValueTag::TextWithLanguage as u8,
            IppValue::NameWithLanguage { .. } => ValueTag::NameWithLanguage as u8,
            IppValue::Charset(_) => ValueTag::Charset as u8,
            IppValue::NaturalLanguage(_) => ValueTag::NaturalLanguage as u8,
            IppValue::Uri(_) => ValueTag::Uri as u8,
            IppValue::UriScheme(_) => ValueTag::UriScheme as u8,
            IppValue::MimeMediaType(_) => ValueTag::MimeMediaType as u8,
            IppValue::Array(ref array) => array.first().map(|v| v.to_tag()).unwrap_or(ValueTag::Unknown as u8),
            IppValue::Collection(_) => ValueTag::BegCollection as u8,
            IppValue::DateTime { .. } => ValueTag::DateTime as u8,
            IppValue::MemberAttrName(_) => ValueTag::MemberAttrName as u8,
            IppValue::Resolution { .. } => ValueTag::Resolution as u8,
            IppValue::Other { tag, .. } => tag,
            IppValue::NoValue => ValueTag::NoValue as u8,
            IppValue::NonUtf8 { tag, .. } => tag as u8,
        }
    }

    fn from_bounded_utf8<const MAX: usize, VType>(
        into: VType,
        tag: ValueTag,
        data: Bytes,
    ) -> Result<Self, IppParseError>
    where
        VType: FnOnce(BoundedString<MAX>) -> Self,
    {
        if data.len() > MAX {
            Err(IppParseError::InvalidStringLength {
                len: data.len(),
                max: MAX,
            })
        } else {
            Ok(BoundedString::<MAX>::from_bytes(&data).map_or_else(|| IppValue::NonUtf8 { tag, data }, into))
        }
    }

    fn from_utf8<VType>(into: VType, tag: ValueTag, data: Bytes) -> Result<Self, IppParseError>
    where
        VType: FnOnce(IppTextValue) -> Self,
    {
        if data.len() > IPP_STRING_MAX_LENGTH {
            Err(IppParseError::InvalidStringLength {
                len: data.len(),
                max: IPP_STRING_MAX_LENGTH,
            })
        } else {
            Ok(BoundedString::from_bytes(&data)
                .map(IppTextValue::Long)
                .map(IppTextValue::shrink)
                .map_or_else(|| IppValue::NonUtf8 { tag, data }, into))
        }
    }

    /// Parse value from a byte array which does not include the value length field
    pub fn parse(value_tag: u8, mut data: Bytes) -> Result<IppValue, IppParseError> {
        let ipp_tag = match ValueTag::from_u8(value_tag) {
            Some(x) => x,
            None => {
                return Ok(IppValue::Other { tag: value_tag, data });
            }
        };

        let value = match ipp_tag {
            ValueTag::Integer => IppValue::Integer(data.get_i32()),
            ValueTag::Enum => IppValue::Enum(data.get_i32()),
            ValueTag::OctetStringUnspecified => IppValue::OctetString(data),
            ValueTag::TextWithoutLanguage => Self::from_utf8(IppValue::TextWithoutLanguage, ipp_tag, data)?,
            ValueTag::NameWithoutLanguage => Self::from_bounded_utf8(IppValue::NameWithoutLanguage, ipp_tag, data)?,
            ValueTag::TextWithLanguage => IppValue::TextWithLanguage {
                language: get_len_string(&mut data)?.try_into()?,
                text: get_len_string(&mut data)?.try_into()?,
            },
            ValueTag::NameWithLanguage => IppValue::NameWithLanguage {
                language: get_len_string(&mut data)?.try_into()?,
                name: get_len_string(&mut data)?.try_into()?,
            },
            ValueTag::Charset => Self::from_bounded_utf8(IppValue::Charset, ipp_tag, data)?,
            ValueTag::NaturalLanguage => Self::from_bounded_utf8(IppValue::NaturalLanguage, ipp_tag, data)?,
            ValueTag::Uri => Self::from_bounded_utf8(IppValue::Uri, ipp_tag, data)?,
            ValueTag::UriScheme => Self::from_bounded_utf8(IppValue::UriScheme, ipp_tag, data)?,
            ValueTag::RangeOfInteger => IppValue::RangeOfInteger {
                min: data.get_i32(),
                max: data.get_i32(),
            },
            ValueTag::Boolean => IppValue::Boolean(data.get_u8() != 0),
            ValueTag::Keyword => Self::from_bounded_utf8(IppValue::Keyword, ipp_tag, data)?,
            ValueTag::MimeMediaType => Self::from_bounded_utf8(IppValue::MimeMediaType, ipp_tag, data)?,
            ValueTag::DateTime => IppValue::DateTime(IppDateTime {
                year: data.get_u16(),
                month: data.get_u8(),
                day: data.get_u8(),
                hour: data.get_u8(),
                minutes: data.get_u8(),
                seconds: data.get_u8(),
                deci_seconds: data.get_u8(),
                utc_dir: data.get_u8() as char,
                utc_hours: data.get_u8(),
                utc_mins: data.get_u8(),
            }),
            ValueTag::MemberAttrName => Self::from_bounded_utf8(IppValue::MemberAttrName, ipp_tag, data)?,
            ValueTag::Resolution => IppValue::Resolution {
                cross_feed: data.get_i32(),
                feed: data.get_i32(),
                units: data.get_i8(),
            },
            ValueTag::NoValue => IppValue::NoValue,
            _ => IppValue::Other { tag: value_tag, data },
        };
        Ok(value)
    }

    /// Write the value to a byte array, including the leading value length field, excluding the value tag
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        match *self {
            IppValue::Integer(i) | IppValue::Enum(i) => {
                buffer.put_u16(4);
                buffer.put_i32(i);
            }
            IppValue::RangeOfInteger { min, max } => {
                buffer.put_u16(8);
                buffer.put_i32(min);
                buffer.put_i32(max);
            }
            IppValue::Boolean(b) => {
                buffer.put_u16(1);
                buffer.put_u8(b as u8);
            }
            IppValue::Keyword(ref s) | IppValue::NameWithoutLanguage(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::OctetString(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s);
            }
            IppValue::TextWithoutLanguage(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }

            IppValue::Charset(ref s) | IppValue::NaturalLanguage(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::Uri(ref s) | IppValue::UriScheme(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::MimeMediaType(ref s) | IppValue::MemberAttrName(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::TextWithLanguage { ref language, ref text } => {
                buffer.put_u16((language.len() + text.len() + 4) as u16);
                buffer.put_u16(language.len() as u16);
                buffer.put_slice(language.as_bytes());
                buffer.put_u16(text.len() as u16);
                buffer.put_slice(text.as_bytes());
            }
            IppValue::NameWithLanguage { ref language, ref name } => {
                buffer.put_u16((language.len() + name.len() + 4) as u16);
                buffer.put_u16(language.len() as u16);
                buffer.put_slice(language.as_bytes());
                buffer.put_u16(name.len() as u16);
                buffer.put_slice(name.as_bytes());
            }
            IppValue::Array(ref list) => {
                for (i, item) in list.iter().enumerate() {
                    buffer.put(item.to_bytes());
                    if i < list.len() - 1 {
                        buffer.put_u8(self.to_tag());
                        buffer.put_u16(0);
                    }
                }
            }
            IppValue::Collection(ref list) => {
                // begin collection: value size is 0
                buffer.put_u16(0);

                for item in list.iter() {
                    let atr_name: IppValue = IppValue::MemberAttrName(item.0.clone());
                    // item tag
                    buffer.put_u8(atr_name.to_tag());
                    // name size is zero, this is a collection
                    buffer.put_u16(0);

                    buffer.put(atr_name.to_bytes());

                    // item tag
                    buffer.put_u8(item.1.to_tag());
                    // name size is zero, this is a collection
                    buffer.put_u16(0);

                    buffer.put(item.1.to_bytes());
                }
                // write end collection attribute
                buffer.put_u8(ValueTag::EndCollection as u8);
                buffer.put_u32(0);
            }
            IppValue::DateTime(ref datetime) => {
                buffer.put_u16(11);
                buffer.put(datetime.to_bytes())
            }
            IppValue::Resolution {
                cross_feed,
                feed,
                units,
            } => {
                buffer.put_u16(9);
                buffer.put_i32(cross_feed);
                buffer.put_i32(feed);
                buffer.put_u8(units as u8);
            }
            IppValue::NoValue => buffer.put_u16(0),
            IppValue::Other { ref data, .. } | IppValue::NonUtf8 { ref data, .. } => {
                buffer.put_u16(data.len() as u16);
                buffer.put_slice(data);
            }
        }
        buffer.freeze()
    }
}

/// Implement Display trait to print the value
impl fmt::Display for IppValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppValue::Integer(i) | IppValue::Enum(i) => write!(f, "{i}"),
            IppValue::RangeOfInteger { min, max } => write!(f, "{min}..{max}"),
            IppValue::Boolean(b) => write!(f, "{}", if b { "true" } else { "false" }),
            IppValue::Keyword(ref s) | IppValue::NameWithoutLanguage(ref s) => {
                write!(f, "{s}")
            }
            IppValue::TextWithoutLanguage(ref s) => {
                write!(f, "{s}")
            }
            IppValue::Charset(ref s) | IppValue::NaturalLanguage(ref s) => {
                write!(f, "{s}")
            }
            IppValue::Uri(ref s) | IppValue::UriScheme(ref s) => {
                write!(f, "{s}")
            }
            IppValue::MimeMediaType(ref s) | IppValue::MemberAttrName(ref s) => write!(f, "{s}"),
            IppValue::TextWithLanguage { ref language, ref text } => write!(f, "{language}:{text}"),
            IppValue::NameWithLanguage { ref language, ref name } => write!(f, "{language}:{name}"),
            IppValue::Array(ref array) => {
                let s: Vec<String> = array.iter().map(|v| format!("{v}")).collect();
                write!(f, "[{}]", s.join(", "))
            }
            IppValue::Collection(ref coll) => {
                let s: Vec<String> = coll.iter().map(|(k, v)| format!("{k}={v}")).collect();
                write!(f, "<{}>", s.join(", "))
            }
            IppValue::DateTime(ref datetime) => datetime.fmt(f),
            IppValue::Resolution {
                cross_feed,
                feed,
                units,
            } => {
                write!(f, "{cross_feed}x{feed}{}", if units == 3 { "in" } else { "cm" })
            }

            IppValue::NoValue => Ok(()),
            IppValue::OctetString(ref data) => write!(f, "{:0x}: {data:?}", ValueTag::OctetStringUnspecified as u8),
            IppValue::Other { tag, ref data } => write!(f, "{tag:0x}: {data:?}"),
            IppValue::NonUtf8 { tag, ref data } => write!(f, "{:0x}: {}", tag as u8, data.escape_ascii()),
        }
    }
}

impl FromStr for IppValue {
    type Err = IppParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = match s {
            "true" => IppValue::Boolean(true),
            "false" => IppValue::Boolean(false),
            other => {
                if let Ok(iv) = other.parse::<i32>() {
                    IppValue::Integer(iv)
                } else {
                    IppValue::Keyword(other.try_into()?)
                }
            }
        };
        Ok(value)
    }
}

impl<'a> IntoIterator for &'a IppValue {
    type Item = &'a IppValue;
    type IntoIter = IppValueIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IppValueIterator { value: self, index: 0 }
    }
}

/// Iterator over [`IppValue`] items, yielding individual elements from arrays and collections
pub struct IppValueIterator<'a> {
    value: &'a IppValue,
    index: usize,
}

impl<'a> Iterator for IppValueIterator<'a> {
    type Item = &'a IppValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.value {
            IppValue::Array(array) => {
                if self.index < array.len() {
                    self.index += 1;
                    Some(&array[self.index - 1])
                } else {
                    None
                }
            }
            IppValue::Collection(map) => {
                if let Some(entry) = map.iter().nth(self.index) {
                    self.index += 1;
                    Some(entry.1)
                } else {
                    None
                }
            }
            _ => {
                if self.index == 0 {
                    self.index += 1;
                    Some(self.value)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, io};

    use super::*;
    use crate::{attribute::IppAttribute, model::DelimiterTag, parser::IppParser, reader::IppReader};

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_datetime_roundtrip() {
        use chrono::{DateTime, FixedOffset};

        let original = IppDateTime {
            year: 2020,
            month: 2,
            day: 13,
            hour: 12,
            minutes: 34,
            seconds: 22,
            deci_seconds: 9,
            utc_dir: '+',
            utc_hours: 5,
            utc_mins: 30,
        };

        let dt: DateTime<FixedOffset> = original.clone().try_into().unwrap();
        assert_eq!(dt.to_string(), "2020-02-13 12:34:22.900 +05:30");

        let back: IppDateTime = dt.into();
        assert_eq!(back, original);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_datetime_negative_offset() {
        use chrono::{DateTime, FixedOffset};

        let original = IppDateTime {
            year: 1999,
            month: 12,
            day: 31,
            hour: 23,
            minutes: 59,
            seconds: 59,
            deci_seconds: 8,
            utc_dir: '-',
            utc_hours: 8,
            utc_mins: 0,
        };

        let dt: DateTime<FixedOffset> = original.clone().try_into().unwrap();
        assert_eq!(dt.to_string(), "1999-12-31 23:59:59.800 -08:00");

        let back: IppDateTime = dt.into();
        assert_eq!(back, original);
    }

    fn value_check(value: IppValue) {
        let mut b = value.to_bytes();
        b.advance(2); // skip value size
        assert_eq!(IppValue::parse(value.to_tag(), b).unwrap(), value);

        #[cfg(feature = "serde")]
        {
            let json = serde_json::to_string(&value).unwrap();
            let from_json: IppValue = serde_json::from_str(&json).unwrap();
            assert_eq!(value, from_json);
        }
    }

    /*
    // this is a test that deliberately fails to compile as it uses a trait conditional evaluation to check the type of non equal const generics
    #[test]
    fn should_fail_to_compile() {
        let ipp_name: BoundedString = IppAttribute::ATTRIBUTES_CHARSET.into();
    }*/

    #[test]
    fn test_value_single() {
        value_check(IppValue::Integer(1234));
        value_check(IppValue::Enum(4321));
        value_check(IppValue::OctetString("octet-string".into()));
        value_check(IppValue::TextWithoutLanguage(
            "text-without".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::NameWithoutLanguage(
            "name-without".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::TextWithLanguage {
            language: "en".try_into().expect("failed to create IPP text value"),
            text: "text-with".try_into().expect("failed to create IPP text value"),
        });
        value_check(IppValue::NameWithLanguage {
            language: "en".try_into().expect("failed to create IPP text value"),
            name: "name-with".try_into().expect("failed to create IPP text value"),
        });
        value_check(IppValue::Charset(
            "charset".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::NaturalLanguage(
            "natural".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::Uri(
            "uri".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::UriScheme(
            "urischeme".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::RangeOfInteger { min: -12, max: 45 });
        value_check(IppValue::Boolean(true));
        value_check(IppValue::Boolean(false));
        value_check(IppValue::Keyword(
            "keyword".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::MimeMediaType(
            "mime".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::DateTime(IppDateTime {
            year: 2020,
            month: 2,
            day: 13,
            hour: 12,
            minutes: 34,
            seconds: 22,
            deci_seconds: 1,
            utc_dir: 'c',
            utc_hours: 1,
            utc_mins: 30,
        }));
        value_check(IppValue::MemberAttrName(
            "member".try_into().expect("failed to create IPP text value"),
        ));
        value_check(IppValue::Resolution {
            cross_feed: 800,
            feed: 600,
            units: 2,
        });
        value_check(IppValue::NoValue);
        value_check(IppValue::Other {
            tag: 123,
            data: "foo".into(),
        });
    }

    #[test]
    fn test_value_iterator_single() {
        let val = IppValue::Integer(1234);

        for v in &val {
            assert_eq!(*v, val);
        }
    }

    #[test]
    fn test_value_iterator_multiple() {
        let list = vec![IppValue::Integer(1234), IppValue::Integer(5678)];
        let val = IppValue::Array(list.clone());

        for v in val.into_iter().enumerate() {
            assert_eq!(*v.1, list[v.0]);
        }
    }

    #[test]
    fn test_array() {
        let attr = IppAttribute::new(
            "list".try_into().unwrap(),
            IppValue::Array(vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)]),
        );
        let buf = attr.to_bytes().to_vec();

        assert_eq!(
            buf,
            vec![
                0x21, 0, 4, b'l', b'i', b's', b't', 0, 4, 0x11, 0x11, 0x11, 0x11, 0x21, 0, 0, 0, 4, 0x22, 0x22, 0x22,
                0x22
            ],
        );

        let mut data = vec![1, 1, 0, 0, 0, 0, 0, 0, 4];
        data.extend(buf);
        data.push(3);

        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let group = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap();
        let attr = group.get("list").unwrap();
        assert_eq!(
            attr.value().as_array(),
            Some(&vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)])
        );
    }

    #[test]
    fn test_collection() {
        let attr = IppAttribute::new(
            "coll".try_into().unwrap(),
            IppValue::Collection(BTreeMap::from([(
                "abcd".try_into().unwrap(),
                IppValue::Integer(0x2222_2222),
            )])),
        );
        let buf = attr.to_bytes();

        assert_eq!(
            vec![
                0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x4a, 0, 0, 0, 4, b'a', b'b', b'c', b'd', 0x21, 0, 0, 0, 4,
                0x22, 0x22, 0x22, 0x22, 0x37, 0, 0, 0, 0,
            ],
            buf
        );

        let mut data = vec![1, 1, 0, 0, 0, 0, 0, 0, 4];
        data.extend(buf);
        data.push(3);

        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let group = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap();
        let attr = group.get("coll").unwrap();
        assert_eq!(
            attr.value(),
            &IppValue::Collection(BTreeMap::from([(
                "abcd".try_into().unwrap(),
                IppValue::Integer(0x2222_2222)
            )]))
        );
    }
}
