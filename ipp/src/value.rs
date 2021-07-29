//!
//! IPP value
//!
use std::{convert::Infallible, fmt, io, str::FromStr};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use enum_as_inner::EnumAsInner;

use crate::{model::ValueTag, FromPrimitive as _};

/// IPP attribute values as defined in [RFC 8010](https://tools.ietf.org/html/rfc8010)
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
    UriScheme(String),
    RangeOfInteger {
        min: i32,
        max: i32,
    },
    Boolean(bool),
    Keyword(String),
    Array(Vec<IppValue>),
    Collection(Vec<IppValue>),
    MimeMediaType(String),
    DateTime {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minutes: u8,
        seconds: u8,
        deci_seconds: u8,
        utc_dir: char,
        utc_hours: u8,
        utc_mins: u8,
    },
    MemberAttrName(String),
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
}

impl IppValue {
    /// Convert to binary tag
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
            IppValue::Charset(_) => ValueTag::Charset as u8,
            IppValue::NaturalLanguage(_) => ValueTag::NaturalLanguage as u8,
            IppValue::Uri(_) => ValueTag::Uri as u8,
            IppValue::UriScheme(_) => ValueTag::UriScheme as u8,
            IppValue::MimeMediaType(_) => ValueTag::MimeMediaType as u8,
            IppValue::Array(ref array) => array.get(0).map(|v| v.to_tag()).unwrap_or(ValueTag::Unknown as u8),
            IppValue::Collection(_) => ValueTag::BegCollection as u8,
            IppValue::DateTime { .. } => ValueTag::DateTime as u8,
            IppValue::MemberAttrName(_) => ValueTag::MemberAttrName as u8,
            IppValue::Resolution { .. } => ValueTag::Resolution as u8,
            IppValue::Other { tag, .. } => tag,
            IppValue::NoValue => ValueTag::NoValue as u8,
        }
    }

    /// Parse value from byte array which does not include the value length field
    pub fn parse(value_tag: u8, mut data: Bytes) -> io::Result<IppValue> {
        let ipp_tag = match ValueTag::from_u8(value_tag) {
            Some(x) => x,
            None => {
                return Ok(IppValue::Other { tag: value_tag, data });
            }
        };

        let value = match ipp_tag {
            ValueTag::Integer => IppValue::Integer(data.get_i32()),
            ValueTag::Enum => IppValue::Enum(data.get_i32()),
            ValueTag::OctetStringUnspecified => IppValue::OctetString(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::TextWithoutLanguage => IppValue::TextWithoutLanguage(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::NameWithoutLanguage => IppValue::NameWithoutLanguage(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::Charset => IppValue::Charset(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::NaturalLanguage => IppValue::NaturalLanguage(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::Uri => IppValue::Uri(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::UriScheme => IppValue::UriScheme(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::RangeOfInteger => IppValue::RangeOfInteger {
                min: data.get_i32(),
                max: data.get_i32(),
            },
            ValueTag::Boolean => IppValue::Boolean(data.get_u8() != 0),
            ValueTag::Keyword => IppValue::Keyword(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::MimeMediaType => IppValue::MimeMediaType(String::from_utf8_lossy(&data).into_owned()),
            ValueTag::DateTime => IppValue::DateTime {
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
            },
            ValueTag::MemberAttrName => IppValue::MemberAttrName(String::from_utf8_lossy(&data).into_owned()),
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

    /// Write value to byte array, including leading value length field, excluding value tag
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
            IppValue::Keyword(ref s)
            | IppValue::OctetString(ref s)
            | IppValue::TextWithoutLanguage(ref s)
            | IppValue::NameWithoutLanguage(ref s)
            | IppValue::Charset(ref s)
            | IppValue::NaturalLanguage(ref s)
            | IppValue::Uri(ref s)
            | IppValue::UriScheme(ref s)
            | IppValue::MimeMediaType(ref s)
            | IppValue::MemberAttrName(ref s) => {
                buffer.put_u16(s.len() as u16);
                buffer.put_slice(s.as_bytes());
            }
            IppValue::Array(ref list) => {
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
                deci_seconds,
                utc_dir,
                utc_hours,
                utc_mins,
            } => {
                buffer.put_u16(11);
                buffer.put_u16(year);
                buffer.put_u8(month);
                buffer.put_u8(day);
                buffer.put_u8(hour);
                buffer.put_u8(minutes);
                buffer.put_u8(seconds);
                buffer.put_u8(deci_seconds);
                buffer.put_u8(utc_dir as u8);
                buffer.put_u8(utc_hours);
                buffer.put_u8(utc_mins);
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
            IppValue::Other { ref data, .. } => {
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
            | IppValue::UriScheme(ref s)
            | IppValue::MimeMediaType(ref s)
            | IppValue::MemberAttrName(ref s) => write!(f, "{}", s),
            IppValue::Array(ref array) => {
                let s: Vec<String> = array.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            IppValue::Collection(ref coll) => {
                let s: Vec<String> = coll.iter().map(|v| format!("{}", v)).collect();
                write!(f, "<{}>", s.join(", "))
            }
            IppValue::DateTime {
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
            } => write!(
                f,
                "{}-{}-{},{}:{}:{}.{},{}{}utc",
                year, month, day, hour, minutes, seconds, deci_seconds, utc_dir as char, utc_hours
            ),
            IppValue::Resolution {
                cross_feed,
                feed,
                units,
            } => {
                write!(f, "{}x{}{}", cross_feed, feed, if units == 3 { "in" } else { "cm" })
            }

            IppValue::NoValue => Ok(()),
            IppValue::Other { tag, ref data } => write!(f, "{:0x}: {:?}", tag, data),
        }
    }
}

impl FromStr for IppValue {
    type Err = Infallible;

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
            IppValue::Array(ref array) | IppValue::Collection(ref array) => {
                if self.index < array.len() {
                    self.index += 1;
                    Some(&array[self.index - 1])
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
    use crate::attribute::IppAttribute;
    use crate::model::DelimiterTag;
    use crate::parser::IppParser;
    use crate::reader::IppReader;

    use super::*;

    fn value_check(value: IppValue) {
        let mut b = value.to_bytes();
        b.advance(2); // skip value size
        assert_eq!(IppValue::parse(value.to_tag() as u8, b).unwrap(), value);
    }

    #[test]
    fn test_value_single() {
        value_check(IppValue::Integer(1234));
        value_check(IppValue::Enum(4321));
        value_check(IppValue::OctetString("octet-string".to_owned()));
        value_check(IppValue::TextWithoutLanguage("text-without".to_owned()));
        value_check(IppValue::NameWithoutLanguage("name-without".to_owned()));
        value_check(IppValue::Charset("charset".to_owned()));
        value_check(IppValue::NaturalLanguage("natural".to_owned()));
        value_check(IppValue::Uri("uri".to_owned()));
        value_check(IppValue::UriScheme("urischeme".to_owned()));
        value_check(IppValue::RangeOfInteger { min: -12, max: 45 });
        value_check(IppValue::Boolean(true));
        value_check(IppValue::Boolean(false));
        value_check(IppValue::Keyword("keyword".to_owned()));
        value_check(IppValue::MimeMediaType("mime".to_owned()));
        value_check(IppValue::DateTime {
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
        });
        value_check(IppValue::MemberAttrName("member".to_owned()));
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
            "list",
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
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("list").unwrap();
        assert_eq!(
            attr.value().as_array(),
            Some(&vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)])
        );
    }

    #[test]
    fn test_collection() {
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
        data.push(3);

        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("coll").unwrap();
        assert_eq!(
            attr.value().as_collection(),
            Some(&vec![IppValue::Integer(0x1111_1111), IppValue::Integer(0x2222_2222)])
        );
    }
}
