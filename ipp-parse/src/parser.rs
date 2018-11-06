//!
//! IPP stream parser
//!
use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use log::debug;
use num_traits::FromPrimitive;

use ipp::*;
use {IppAttribute, IppAttributes, IppHeader, IppReadExt, IppValue};

// create a single value from one-element list, list otherwise
fn list_or_value(mut list: Vec<IppValue>) -> IppValue {
    if list.len() == 1 {
        list.remove(0)
    } else {
        IppValue::ListOf(list)
    }
}

/// IPP parsing result
pub struct IppParseResult {
    pub header: IppHeader,
    pub attributes: IppAttributes,
}

impl IppParseResult {
    fn new(header: IppHeader, attributes: IppAttributes) -> IppParseResult {
        IppParseResult { header, attributes }
    }
}

/// IPP parser implementation
pub struct IppParser<'a> {
    reader: &'a mut Read,
    // last delimiter tag
    last_delimiter: DelimiterTag,
    // last attribute value
    last_name: Option<String>,
    // stack of current attributes context. Used with lists and collections
    context: Vec<Vec<IppValue>>,
    // holds the result of parsing
    attributes: IppAttributes,
}

impl<'a> IppParser<'a> {
    /// Create IPP parser using the given Read
    pub fn new(reader: &'a mut Read) -> IppParser<'a> {
        IppParser {
            reader,
            last_delimiter: DelimiterTag::EndOfAttributes,
            last_name: None,
            context: vec![vec![]],
            attributes: IppAttributes::new(),
        }
    }

    fn add_attribute(&mut self) {
        if let Some(ref last_name) = self.last_name {
            if let Some(val_list) = self.context.pop() {
                self.attributes.add(
                    self.last_delimiter,
                    IppAttribute::new(&last_name, list_or_value(val_list)),
                );
            }
            self.context.push(vec![]);
        }
    }

    fn parse_delimiter(&mut self, tag: u8) -> io::Result<bool> {
        debug!("Delimiter tag: {:0x}", tag);
        if tag == DelimiterTag::EndOfAttributes as u8 {
            // end of stream, get last saved collection
            self.add_attribute();
            Ok(true)
        } else {
            // remember delimiter tag
            self.last_delimiter = DelimiterTag::from_u8(tag).ok_or(io::Error::new(
                io::ErrorKind::Other,
                format!("Tag error: {}", tag),
            ))?;
            Ok(false)
        }
    }

    fn parse_value(&mut self, tag: u8) -> io::Result<()> {
        // value tag
        let namelen = self.reader.read_u16::<BigEndian>()?;
        let name = self.reader.read_string(namelen as usize)?;
        let value = IppValue::read(tag, &mut self.reader)?;

        debug!("Value tag: {:0x}: {}: {}", tag, name, value);

        if namelen > 0 {
            // single attribute or begin of array
            self.add_attribute();
            // store it as a previous attribute
            self.last_name = Some(name);
        }
        if tag == ValueTag::BegCollection as u8 {
            // start new collection in the stack
            debug!("Begin collection");
            self.context.push(vec![]);
        } else if tag == ValueTag::EndCollection as u8 {
            // get collection from the stack and add it to the previous element
            debug!("End collection");
            if let Some(arr) = self.context.pop() {
                if let Some(val_list) = self.context.last_mut() {
                    val_list.push(IppValue::Collection(arr));
                }
            }
        } else if let Some(val_list) = self.context.last_mut() {
            // add attribute to the current collection
            val_list.push(value);
        }
        Ok(())
    }

    /// Parse IPP stream
    pub fn parse(mut self) -> io::Result<IppParseResult> {
        // parse IPP header
        let header = IppHeader::from_reader(self.reader)?;
        debug!("IPP header: {:?}", header);

        loop {
            match self.reader.read_u8()? {
                tag @ 0x01...0x05 => if self.parse_delimiter(tag)? {
                    break;
                },
                tag @ 0x10...0x4a => self.parse_value(tag)?,
                tag => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Tag error: {}", tag),
                    ))
                }
            }
        }

        Ok(IppParseResult::new(header, self.attributes))
    }
}
