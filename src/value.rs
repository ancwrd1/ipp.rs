//!
//! IPP value
//!
use std::io::{Read, Write};
use std::fmt;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use ::{Result, ReadIppExt};
use consts::tag::*;

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
    RangeOfInteger(i32, i32),
    Boolean(bool),
    Keyword(String),
    ListOf(Vec<IppValue>),
    Collection(Vec<IppValue>),
    MimeMediaType(String),
    DateTime(u16, u8, u8, u8, u8, u8, u8, char, u8, u8),
    MemberAttrName(String),
    Resolution(i32, i32, i8),
    Other(u8, Vec<u8>)
}

impl IppValue {
    /// Convert to binary tag
    pub fn to_tag(&self) -> u8 {
        match *self {
            IppValue::Integer(_) => INTEGER,
            IppValue::Enum(_) => ENUM,
            IppValue::RangeOfInteger(_, _) => RANGEOFINTEGER,
            IppValue::Boolean(_) => BOOLEAN,
            IppValue::Keyword(_) => KEYWORD,
            IppValue::OctetString(_) => OCTECTSTRING_UNSPECIFIED,
            IppValue::TextWithoutLanguage(_) => TEXT_WITHOUT_LANGUAGE,
            IppValue::NameWithoutLanguage(_) => NAME_WITHOUT_LANGUAGE,
            IppValue::Charset(_) => CHARSET,
            IppValue::NaturalLanguage(_) => NATURAL_LANGUAGE,
            IppValue::Uri(_) => URI,
            IppValue::MimeMediaType(_) => MIME_MEDIA_TYPE,
            IppValue::ListOf(ref list) => list[0].to_tag(),
            IppValue::Collection(_) => BEG_COLLECTION,
            IppValue::DateTime(_,_,_,_,_,_,_,_,_,_) => DATETIME,
            IppValue::MemberAttrName(_) => MEMBER_ATTR_NAME,
            IppValue::Resolution(_, _, _) => RESOLUTION,
            IppValue::Other(tag, _) => tag
        }
    }

    /// Read value from binary stream
    pub fn read(vtag: u8, reader: &mut Read) -> Result<IppValue> {
        let vsize = reader.read_u16::<BigEndian>()?;

        match vtag {
            INTEGER => {
                debug_assert_eq!(vsize, 4);
                Ok(IppValue::Integer(reader.read_i32::<BigEndian>()?))
            }
            ENUM => {
                debug_assert_eq!(vsize, 4);
                Ok(IppValue::Enum(reader.read_i32::<BigEndian>()?))
            }
            OCTECTSTRING_UNSPECIFIED => {
                Ok(IppValue::OctetString(reader.read_string(vsize as usize)?))
            }
            TEXT_WITHOUT_LANGUAGE => {
                Ok(IppValue::TextWithoutLanguage(reader.read_string(vsize as usize)?))
            }
            NAME_WITHOUT_LANGUAGE => {
                Ok(IppValue::NameWithoutLanguage(reader.read_string(vsize as usize)?))
            }
            CHARSET => {
                Ok(IppValue::Charset(reader.read_string(vsize as usize)?))
            }
            NATURAL_LANGUAGE => {
                Ok(IppValue::NaturalLanguage(reader.read_string(vsize as usize)?))
            }
            URI => {
                Ok(IppValue::Uri(reader.read_string(vsize as usize)?))
            }
            RANGEOFINTEGER => {
                debug_assert_eq!(vsize, 8);
                Ok(IppValue::RangeOfInteger(reader.read_i32::<BigEndian>()?,
                                             reader.read_i32::<BigEndian>()?))
            }
            BOOLEAN => {
                debug_assert_eq!(vsize, 1);
                Ok(IppValue::Boolean(reader.read_u8()? != 0))
            }
            KEYWORD => {
                Ok(IppValue::Keyword(reader.read_string(vsize as usize)?))
            }
            MIME_MEDIA_TYPE => {
                Ok(IppValue::MimeMediaType(reader.read_string(vsize as usize)?))
            }
            DATETIME => {
                Ok(IppValue::DateTime(
                    reader.read_u16::<BigEndian>()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()? as char,
                    reader.read_u8()?,
                    reader.read_u8()?))
            }
            MEMBER_ATTR_NAME => {
                Ok(IppValue::MemberAttrName(reader.read_string(vsize as usize)?))
            }
            RESOLUTION => {
                Ok(IppValue::Resolution(
                    reader.read_i32::<BigEndian>()?,
                    reader.read_i32::<BigEndian>()?,
                    reader.read_i8()?))
            }
            _ => {
                Ok(IppValue::Other(vtag, reader.read_vec(vsize as usize)?))
            }
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
            IppValue::RangeOfInteger(min, max) => {
                writer.write_u16::<BigEndian>(8)?;
                writer.write_i32::<BigEndian>(min)?;
                writer.write_i32::<BigEndian>(max)?;
                Ok(10)
            }
            IppValue::Boolean(b) => {
                writer.write_u16::<BigEndian>(1)?;
                writer.write_u8(if b {1} else {0})?;
                Ok(3)
            }
            IppValue::Keyword(ref s) | IppValue::OctetString(ref s) |
            IppValue::TextWithoutLanguage(ref s) | IppValue::NameWithoutLanguage(ref s) |
            IppValue::Charset(ref s) | IppValue::NaturalLanguage(ref s) |
            IppValue::Uri(ref s) | IppValue::MimeMediaType(ref s) |
            IppValue::MemberAttrName(ref s) => {
                writer.write_u16::<BigEndian>(s.len() as u16)?;
                writer.write_all(s.as_bytes())?;
                Ok(2 + s.len())
            }
            IppValue::ListOf(ref list) => {
                let mut retval = 0;
                for (i, item) in list.iter().enumerate() {
                    retval += item.write(writer)?;
                    if i < list.len() - 1 {
                        writer.write_u8(self.to_tag())?;
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
                        writer.write_u8(self.to_tag())?;
                        writer.write_u16::<BigEndian>(0)?;
                        retval += 3;
                    }
                }
                writer.write_u8(END_COLLECTION)?;
                retval += 1;
                Ok(retval)
            }
            IppValue::DateTime(year, month, day, hour, minutes, seconds, deciseconds, utcdir, utchours, utcmins) => {
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
            IppValue::Resolution(crossfeed, feed, units) => {
                writer.write_u16::<BigEndian>(9)?;
                writer.write_i32::<BigEndian>(crossfeed)?;
                writer.write_i32::<BigEndian>(feed)?;
                writer.write_i8(units)?;
                Ok(9)
            }
            IppValue::Other(_, ref vec) => {
                writer.write_u16::<BigEndian>(vec.len() as u16)?;
                writer.write_all(vec)?;
                Ok(2 + vec.len())
            }
        }
    }
}

/// Implement Display trait to print the value
impl fmt::Display for IppValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IppValue::Integer(i) | IppValue::Enum(i) => {
                write!(f, "{}", i)
            }
            IppValue::RangeOfInteger(min, max) => {
                write!(f, "{}..{}", min, max)
            }
            IppValue::Boolean(b) => {
                write!(f, "{}", if b {"true"} else {"false"})
            }
            IppValue::Keyword(ref s) | IppValue::OctetString(ref s) |
            IppValue::TextWithoutLanguage(ref s) | IppValue::NameWithoutLanguage(ref s) |
            IppValue::Charset(ref s) | IppValue::NaturalLanguage(ref s) |
            IppValue::Uri(ref s) | IppValue::MimeMediaType(ref s) |
            IppValue::MemberAttrName(ref s) => {
                write!(f, "{}", s)
            }
            IppValue::ListOf(ref list) => {
                let s: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            IppValue::Collection(ref list) => {
                let s: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
                write!(f, "<{}>", s.join(", "))
            }
            IppValue::DateTime(year, month, day, hour, minutes, seconds, deciseconds, utcdir, utchours, _) => {
                write!(f, "{}-{}-{},{}:{}:{}.{},{}{}utc", year, month, day, hour,
                    minutes, seconds, deciseconds, utcdir as char, utchours)
            }
            IppValue::Resolution(crossfeed, feed, units) => {
                write!(f, "{}x{}{}", crossfeed, feed, if units == 3 {"in"} else {"cm"})
            }

            IppValue::Other(tag, ref vec) => {
                write!(f, "{:0x}: {:?}", tag, vec)
            }
        }
    }
}

impl<'a> IntoIterator for &'a IppValue {
    type Item = &'a IppValue;
    type IntoIter = IppValueIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IppValueIntoIterator { value: self, index: 0 }
    }
}

pub struct IppValueIntoIterator<'a> {
    value: &'a IppValue,
    index: usize
}

impl<'a> Iterator for IppValueIntoIterator<'a> {
    type Item = &'a IppValue;
    
    fn next(&mut self) -> Option<Self::Item> {
        match *self.value {
            IppValue::ListOf(ref list) | IppValue::Collection(ref list) =>
                if self.index < list.len() { self.index += 1; Some(&list[self.index - 1]) } else { None },
            _ =>
                if self.index == 0 { self.index += 1; Some(self.value) } else { None }
        }
    }
}
