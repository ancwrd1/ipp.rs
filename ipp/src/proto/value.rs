//!
//! IPP value
//!
use std::{fmt, io, str::FromStr};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use enum_as_inner::EnumAsInner;

use super::{ipp::ValueTag, FromPrimitive as _};

/// IPP value enumeration
#[derive(Clone, Debug, PartialEq, EnumAsInner)]
pub enum IppValue {
    Integer(i32),
    Enum(i32),
    OctetString(String),
    TextWithoutLanguage(String),
    NameWithoutLanguage(String),
    Charset(String),
    NaturalLanguage(String),
    Uri(String),
    RangeOfInteger {
        min: i32,
        max: i32,
    },
    Boolean(bool),
    Keyword(String),
    ListOf(Vec<IppValue>),
    Collection(Vec<IppValue>),
    MimeMediaType(String),
    DateTime {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minutes: u8,
        seconds: u8,
        deciseconds: u8,
        utcdir: char,
        utchours: u8,
        utcmins: u8,
    },
    MemberAttrName(String),
    Resolution {
        crossfeed: i32,
        feed: i32,
        units: i8,
    },
    Other {
        tag: u8,
        data: Bytes,
    },
}

impl IppValue {
    /// Convert to binary tag
    pub fn to_tag(&self) -> ValueTag {
        match *self {
            IppValue::Integer(_) => ValueTag::Integer,
            IppValue::Enum(_) => ValueTag::Enum,
            IppValue::RangeOfInteger { .. } => ValueTag::RangeOfInteger,
            IppValue::Boolean(_) => ValueTag::Boolean,
            IppValue::Keyword(_) => ValueTag::Keyword,
            IppValue::OctetString(_) => ValueTag::OctetStringUnspecified,
            IppValue::TextWithoutLanguage(_) => ValueTag::TextWithoutLanguage,
            IppValue::NameWithoutLanguage(_) => ValueTag::NameWithoutLanguage,
            IppValue::Charset(_) => ValueTag::Charset,
            IppValue::NaturalLanguage(_) => ValueTag::NaturalLanguage,
            IppValue::Uri(_) => ValueTag::Uri,
            IppValue::MimeMediaType(_) => ValueTag::MimeMediaType,
            IppValue::ListOf(ref list) => list[0].to_tag(),
            IppValue::Collection(_) => ValueTag::BegCollection,
            IppValue::DateTime { .. } => ValueTag::DateTime,
            IppValue::MemberAttrName(_) => ValueTag::MemberAttrName,
            IppValue::Resolution { .. } => ValueTag::Resolution,
            IppValue::Other { .. } => ValueTag::Unknown,
        }
    }

    /// Parse value from byte array
    pub fn parse(vtag: u8, mut data: Bytes) -> io::Result<IppValue> {
        let ipptag = match ValueTag::from_u8(vtag) {
            Some(x) => x,
            None => {
                return Ok(IppValue::Other { tag: vtag, data });
            }
        };

        match ipptag {
            ValueTag::Integer => Ok(IppValue::Integer(data.get_i32())),
            ValueTag::Enum => Ok(IppValue::Enum(data.get_i32())),
            ValueTag::OctetStringUnspecified => Ok(IppValue::OctetString(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::TextWithoutLanguage => Ok(IppValue::TextWithoutLanguage(
                String::from_utf8_lossy(&data).into_owned(),
            )),
            ValueTag::NameWithoutLanguage => Ok(IppValue::NameWithoutLanguage(
                String::from_utf8_lossy(&data).into_owned(),
            )),
            ValueTag::Charset => Ok(IppValue::Charset(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::NaturalLanguage => Ok(IppValue::NaturalLanguage(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::Uri => Ok(IppValue::Uri(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::RangeOfInteger => Ok(IppValue::RangeOfInteger {
                min: data.get_i32(),
                max: data.get_i32(),
            }),
            ValueTag::Boolean => Ok(IppValue::Boolean(data.get_u8() != 0)),
            ValueTag::Keyword => Ok(IppValue::Keyword(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::MimeMediaType => Ok(IppValue::MimeMediaType(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::DateTime => Ok(IppValue::DateTime {
                year: data.get_u16(),
                month: data.get_u8(),
                day: data.get_u8(),
                hour: data.get_u8(),
                minutes: data.get_u8(),
                seconds: data.get_u8(),
                deciseconds: data.get_u8(),
                utcdir: data.get_u8() as char,
                utchours: data.get_u8(),
                utcmins: data.get_u8(),
            }),
            ValueTag::MemberAttrName => Ok(IppValue::MemberAttrName(String::from_utf8_lossy(&data).into_owned())),
            ValueTag::Resolution => Ok(IppValue::Resolution {
                crossfeed: data.get_i32(),
                feed: data.get_i32(),
                units: data.get_i8(),
            }),
            _ => Ok(IppValue::Other { tag: vtag, data }),
        }
    }

    /// Write value to byte array
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
                buffer.put_u8(if b { 1 } else { 0 });
            }
            IppValue::Keyword(ref s)
            | IppValue::OctetString(ref s)
            | IppValue::TextWithoutLanguage(ref s)
            | IppValue::NameWithoutLanguage(ref s)
            | IppValue::Charset(ref s)
            | IppValue::NaturalLanguage(ref s)
            | IppValue::Uri(ref s)
            | IppValue::MimeMediaType(ref s)
            | IppValue::MemberAttrName(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::ListOf(ref list) => {
                for (i, item) in list.iter().enumerate() {
                    buffer.put(item.to_bytes());
                    if i < list.len() - 1 {
                        buffer.put_u8(self.to_tag() as u8);
                        buffer.put_u16(0);
                    }
                }
            }
            IppValue::Collection(ref list) => {
                // begin collection: value size is 0
                buffer.put_u16(0);

                for item in list.iter() {
                    // item tag
                    buffer.put_u8(item.to_tag() as u8);
                    // name size is zero, this is a collection
                    buffer.put_u16(0);

                    buffer.put(item.to_bytes());
                }
                // write end collection attribute
                buffer.put_u8(ValueTag::EndCollection as u8);
                buffer.put_u32(0);
            }
            IppValue::DateTime {
                year,
                month,
                day,
                hour,
                minutes,
                seconds,
                deciseconds,
                utcdir,
                utchours,
                utcmins,
            } => {
                buffer.put_u16(year);
                buffer.put_u8(month);
                buffer.put_u8(day);
                buffer.put_u8(hour);
                buffer.put_u8(minutes);
                buffer.put_u8(seconds);
                buffer.put_u8(deciseconds);
                buffer.put_u8(utcdir as u8);
                buffer.put_u8(utchours);
                buffer.put_u8(utcmins);
            }
            IppValue::Resolution { crossfeed, feed, units } => {
                buffer.put_u16(9);
                buffer.put_i32(crossfeed);
                buffer.put_i32(feed);
                buffer.put_u8(units as u8);
            }
            IppValue::Other { ref data, .. } => {
                buffer.put_u16(data.len() as u16);
                buffer.put_slice(&data);
            }
        }
        buffer.freeze()
    }
}

/// Implement Display trait to print the value
impl fmt::Display for IppValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppValue::Integer(i) | IppValue::Enum(i) => write!(f, "{}", i),
            IppValue::RangeOfInteger { min, max } => write!(f, "{}..{}", min, max),
            IppValue::Boolean(b) => write!(f, "{}", if b { "true" } else { "false" }),
            IppValue::Keyword(ref s)
            | IppValue::OctetString(ref s)
            | IppValue::TextWithoutLanguage(ref s)
            | IppValue::NameWithoutLanguage(ref s)
            | IppValue::Charset(ref s)
            | IppValue::NaturalLanguage(ref s)
            | IppValue::Uri(ref s)
            | IppValue::MimeMediaType(ref s)
            | IppValue::MemberAttrName(ref s) => write!(f, "{}", s),
            IppValue::ListOf(ref list) => {
                let s: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            IppValue::Collection(ref list) => {
                let s: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
                write!(f, "<{}>", s.join(", "))
            }
            IppValue::DateTime {
                year,
                month,
                day,
                hour,
                minutes,
                seconds,
                deciseconds,
                utcdir,
                utchours,
                ..
            } => write!(
                f,
                "{}-{}-{},{}:{}:{}.{},{}{}utc",
                year, month, day, hour, minutes, seconds, deciseconds, utcdir as char, utchours
            ),
            IppValue::Resolution { crossfeed, feed, units } => {
                write!(f, "{}x{}{}", crossfeed, feed, if units == 3 { "in" } else { "cm" })
            }

            IppValue::Other { tag, ref data } => write!(f, "{:0x}: {:?}", tag, data),
        }
    }
}

#[derive(Debug)]
pub struct ValueParseError;

impl fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IPP value parse error")
    }
}
impl std::error::Error for ValueParseError {}

impl FromStr for IppValue {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = match s {
            "true" => IppValue::Boolean(true),
            "false" => IppValue::Boolean(false),
            other => {
                if let Ok(iv) = other.parse::<i32>() {
                    IppValue::Integer(iv)
                } else {
                    IppValue::Keyword(other.to_owned())
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

pub struct IppValueIterator<'a> {
    value: &'a IppValue,
    index: usize,
}

impl<'a> Iterator for IppValueIterator<'a> {
    type Item = &'a IppValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.value {
            IppValue::ListOf(ref list) | IppValue::Collection(ref list) => {
                if self.index < list.len() {
                    self.index += 1;
                    Some(&list[self.index - 1])
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
    use crate::proto::{ipp::DelimiterTag, IppAttribute};

    use super::*;

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
        let val = IppValue::ListOf(list.clone());

        for v in val.into_iter().enumerate() {
            assert_eq!(*v.1, list[v.0]);
        }
    }

    #[test]
    fn test_collection_de_serialize() {
        let attr = IppAttribute::new(
            "coll",
            IppValue::Collection(vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)]),
        );
        let buf = attr.to_bytes();

        assert_eq!(
            vec![
                0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x21, 0, 0, 0, 4, 0x11, 0x11, 0x11, 0x11, 0x21, 0, 0, 0, 4,
                0x22, 0x22, 0x22, 0x22, 0x37, 0, 0, 0, 0,
            ],
            buf
        );

        let mut data = vec![1, 1, 0, 0, 0, 0, 0, 0, 4];
        data.extend(buf);
        data.extend(vec![3]);

        let result =
            futures::executor::block_on(crate::proto::parser::IppParser::new(futures::io::Cursor::new(data)).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("coll").unwrap();
        assert_eq!(
            attr.value().as_collection(),
            Some(&vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)])
        );
    }
}
