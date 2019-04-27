//!
//! IPP stream parser
//!
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;

use num_traits::FromPrimitive;

use attribute::{IppAttribute, IppAttributeList};
use consts::statuscode::StatusCode;
use consts::tag::*;
use value::IppValue;
use {IppError, IppHeader, ReadIppExt, Result};

fn list_to_value(mut list: Vec<IppValue>) -> IppValue {
    if list.len() == 1 {
        list.remove(0)
    } else {
        IppValue::ListOf(list)
    }
}

/// IPP parsing result
pub struct IppParseResult {
    header: IppHeader,
    attributes: IppAttributeList,
}

impl IppParseResult {
    /// Create instance of the parsing result
    pub fn new(header: IppHeader, attributes: IppAttributeList) -> IppParseResult {
        IppParseResult { header, attributes }
    }

    /// Get parsed header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get parsed attributes
    pub fn attributes(&self) -> &IppAttributeList {
        &self.attributes
    }
}

/// IPP parser implementation
pub struct IppParser<'a> {
    reader: &'a mut Read,
}

impl<'a> IppParser<'a> {
    /// Create IPP parser using the given Read
    pub fn new(reader: &'a mut Read) -> IppParser<'a> {
        IppParser { reader }
    }

    /// Parse IPP stream
    pub fn parse(&mut self) -> Result<IppParseResult> {
        // last delimiter tag
        let mut delimiter = DelimiterTag::EndOfAttributes;

        // stack of current attributes context. Used with lists and collections
        let mut stack = vec![vec![]];

        // holds the result of parsing
        let mut retval = IppAttributeList::new();

        // name of previous attribute name
        let mut last_name: Option<String> = None;

        // parse IPP header
        let header = IppHeader::from_reader(self.reader)?;
        debug!("IPP reply header: {:?}", header);

        loop {
            let tag = self.reader.read_u8()?;
            if is_delimiter_tag(tag) {
                debug!("Delimiter tag: {:0x}", tag);
                if tag == DelimiterTag::EndOfAttributes as u8 {
                    // end of stream, get last saved collection
                    if let Some(last_name) = last_name {
                        if let Some(val_list) = stack.pop() {
                            retval.add(
                                delimiter,
                                IppAttribute::new(&last_name, list_to_value(val_list)),
                            );
                        }
                    }
                    break;
                } else {
                    // remember delimiter tag
                    delimiter =
                        DelimiterTag::from_u8(tag).ok_or(StatusCode::ClientErrorBadRequest)?;
                }
            } else if is_value_tag(tag) {
                // value tag
                let namelen = self.reader.read_u16::<BigEndian>()?;
                let name = self.reader.read_string(namelen as usize)?;
                let value = IppValue::read(tag, &mut self.reader)?;

                debug!("Value tag: {:0x}: {}: {}", tag, name, value);

                if namelen > 0 {
                    // single attribute or begin of array
                    if let Some(last_name) = last_name {
                        // put the previous attribute into the retval
                        if let Some(val_list) = stack.pop() {
                            retval.add(
                                delimiter,
                                IppAttribute::new(&last_name, list_to_value(val_list)),
                            );
                        }
                        stack.push(vec![]);
                    }
                    // store it as a previous attribute
                    last_name = Some(name);
                }
                if tag == ValueTag::BegCollection as u8 {
                    // start new collection in the stack
                    debug!("Begin collection");
                    stack.push(vec![])
                } else if tag == ValueTag::EndCollection as u8 {
                    // get collection from the stack and add it to the previous element
                    debug!("End collection");
                    if let Some(arr) = stack.pop() {
                        if let Some(val_list) = stack.last_mut() {
                            val_list.push(IppValue::Collection(arr));
                        }
                    }
                } else if let Some(val_list) = stack.last_mut() {
                    // add attribute to the current collection
                    val_list.push(value);
                }
            } else {
                return Err(IppError::TagError(tag));
            }
        }

        Ok(IppParseResult::new(header, retval))
    }
}
