//!
//! IPP value
//!
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{Read, Write};

use num_traits::FromPrimitive;

use consts::tag::ValueTag;
use {ReadIppExt, Result};

/// Currently supported IPP values
#[derive(Clone, Debug)]
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
        data: Vec<u8>,
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
            IppValue::OctetString(_) => ValueTag::OctectStringUnspecified,
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

    /// Read value from binary stream
    pub fn read(vtag: u8, reader: &mut Read) -> Result<IppValue> {
        let vsize = reader.read_u16::<BigEndian>()?;

        let ipptag = match ValueTag::from_u8(vtag) {
            Some(x) => x,
            None => {
                return Ok(IppValue::Other {
                    tag: vtag,
                    data: reader.read_vec(vsize as usize)?,
                });
            }
        };

        match ipptag {
            ValueTag::Integer => {
                debug_assert_eq!(vsize, 4);
                Ok(IppValue::Integer(reader.read_i32::<BigEndian>()?))
            }
            ValueTag::Enum => {
                debug_assert_eq!(vsize, 4);
                Ok(IppValue::Enum(reader.read_i32::<BigEndian>()?))
            }
            ValueTag::OctectStringUnspecified => {
                Ok(IppValue::OctetString(reader.read_string(vsize as usize)?))
            }
            ValueTag::TextWithoutLanguage => Ok(IppValue::TextWithoutLanguage(
                reader.read_string(vsize as usize)?,
            )),
            ValueTag::NameWithoutLanguage => Ok(IppValue::NameWithoutLanguage(
                reader.read_string(vsize as usize)?,
            )),
            ValueTag::Charset => Ok(IppValue::Charset(reader.read_string(vsize as usize)?)),
            ValueTag::NaturalLanguage => Ok(IppValue::NaturalLanguage(
                reader.read_string(vsize as usize)?,
            )),
            ValueTag::Uri => Ok(IppValue::Uri(reader.read_string(vsize as usize)?)),
            ValueTag::RangeOfInteger => {
                debug_assert_eq!(vsize, 8);
                Ok(IppValue::RangeOfInteger {
                    min: reader.read_i32::<BigEndian>()?,
                    max: reader.read_i32::<BigEndian>()?,
                })
            }
            ValueTag::Boolean => {
                debug_assert_eq!(vsize, 1);
                Ok(IppValue::Boolean(reader.read_u8()? != 0))
            }
            ValueTag::Keyword => Ok(IppValue::Keyword(reader.read_string(vsize as usize)?)),
            ValueTag::MimeMediaType => {
                Ok(IppValue::MimeMediaType(reader.read_string(vsize as usize)?))
            }
            ValueTag::DateTime => Ok(IppValue::DateTime {
                year: reader.read_u16::<BigEndian>()?,
                month: reader.read_u8()?,
                day: reader.read_u8()?,
                hour: reader.read_u8()?,
                minutes: reader.read_u8()?,
                seconds: reader.read_u8()?,
                deciseconds: reader.read_u8()?,
                utcdir: reader.read_u8()? as char,
                utchours: reader.read_u8()?,
                utcmins: reader.read_u8()?,
            }),
            ValueTag::MemberAttrName => Ok(IppValue::MemberAttrName(
                reader.read_string(vsize as usize)?,
            )),
            ValueTag::Resolution => Ok(IppValue::Resolution {
                crossfeed: reader.read_i32::<BigEndian>()?,
                feed: reader.read_i32::<BigEndian>()?,
                units: reader.read_i8()?,
            }),
            _ => Ok(IppValue::Other {
                tag: vtag,
                data: reader.read_vec(vsize as usize)?,
            }),
        }
    }

    /// Write value to binary stream
    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        match *self {
            IppValue::Integer(i) | IppValue::Enum(i) => {
                writer.write_u16::<BigEndian>(4)?;
                writer.write_i32::<BigEndian>(i)?;
                Ok(6)
            }
            IppValue::RangeOfInteger { min, max } => {
                writer.write_u16::<BigEndian>(8)?;
                writer.write_i32::<BigEndian>(min)?;
                writer.write_i32::<BigEndian>(max)?;
                Ok(10)
            }
            IppValue::Boolean(b) => {
                writer.write_u16::<BigEndian>(1)?;
                writer.write_u8(if b { 1 } else { 0 })?;
                Ok(3)
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
                writer.write_u16::<BigEndian>(s.len() as u16)?;
                writer.write_all(s.as_bytes())?;
                Ok(2 + s.len())
            }
            IppValue::ListOf(ref list) => {
                let mut retval = 0;
                for (i, item) in list.iter().enumerate() {
                    retval += item.write(writer)?;
                    if i < list.len() - 1 {
                        writer.write_u8(self.to_tag() as u8)?;
                        writer.write_u16::<BigEndian>(0)?;
                        retval += 3;
                    }
                }
                Ok(retval)
            }
            IppValue::Collection(ref list) => {
                let mut retval = 0;
                for (i, item) in list.iter().enumerate() {
                    retval += item.write(writer)?;
                    if i < list.len() - 1 {
                        writer.write_u8(self.to_tag() as u8)?;
                        writer.write_u16::<BigEndian>(0)?;
                        retval += 3;
                    }
                }
                writer.write_u8(ValueTag::EndCollection as u8)?;
                retval += 1;
                Ok(retval)
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
                writer.write_u16::<BigEndian>(11)?;

                writer.write_u16::<BigEndian>(year)?;
                writer.write_u8(month)?;
                writer.write_u8(day)?;
                writer.write_u8(hour)?;
                writer.write_u8(minutes)?;
                writer.write_u8(seconds)?;
                writer.write_u8(deciseconds)?;
                writer.write_u8(utcdir as u8)?;
                writer.write_u8(utchours)?;
                writer.write_u8(utcmins)?;

                Ok(13)
            }
            IppValue::Resolution {
                crossfeed,
                feed,
                units,
            } => {
                writer.write_u16::<BigEndian>(9)?;
                writer.write_i32::<BigEndian>(crossfeed)?;
                writer.write_i32::<BigEndian>(feed)?;
                writer.write_i8(units)?;
                Ok(9)
            }
            IppValue::Other { ref data, .. } => {
                writer.write_u16::<BigEndian>(data.len() as u16)?;
                writer.write_all(data)?;
                Ok(2 + data.len())
            }
        }
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
            IppValue::Resolution {
                crossfeed,
                feed,
                units,
            } => write!(
                f,
                "{}x{}{}",
                crossfeed,
                feed,
                if units == 3 { "in" } else { "cm" }
            ),

            IppValue::Other { tag, ref data } => write!(f, "{:0x}: {:?}", tag, data),
        }
    }
}

impl<'a> IntoIterator for &'a IppValue {
    type Item = &'a IppValue;
    type IntoIter = IppValueIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IppValueIntoIterator {
            value: self,
            index: 0,
        }
    }
}

pub struct IppValueIntoIterator<'a> {
    value: &'a IppValue,
    index: usize,
}

impl<'a> Iterator for IppValueIntoIterator<'a> {
    type Item = &'a IppValue;

    fn next(&mut self) -> Option<Self::Item> {
        match *self.value {
            IppValue::ListOf(ref list) | IppValue::Collection(ref list) => {
                if self.index < list.len() {
                    self.index += 1;
                    Some(&list[self.index - 1])
                } else {
                    None
                }
            }
            _ => if self.index == 0 {
                self.index += 1;
                Some(self.value)
            } else {
                None
            },
        }
    }
}
