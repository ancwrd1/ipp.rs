//!
//! IPP stream parser
//!
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};

use ::{Result, IppError, IppHeader, ReadIppExt};
use attribute::{IppAttribute, IppAttributeList};
use value::IppValue;
use consts::tag::*;

/// IPP parsing result
pub struct IppParseResult {
    header: IppHeader,
    attributes: IppAttributeList
}

impl IppParseResult {
    /// Create instance of the parsing result
    pub fn new(header: IppHeader, attributes: IppAttributeList) -> IppParseResult {
        IppParseResult {header: header, attributes: attributes}
    }

    /// Get parsed header
    pub fn header<'a>(&'a self) -> &'a IppHeader {
        &self.header
    }

    /// Get parsed attributes
    pub fn attributes<'a>(&'a self) -> &'a IppAttributeList {
        &self.attributes
    }
}

/// IPP parser implementation
pub struct IppParser<'a> {
    reader: &'a mut Read
}

impl<'a> IppParser<'a> {
    /// Create IPP parser using the given Read
    pub fn new(reader: &'a mut Read) -> IppParser<'a> {
        IppParser { reader: reader }
    }

    /// Parse IPP stream
    pub fn parse(&mut self) -> Result<IppParseResult> {
        let mut delimiter = 0;
        let mut stack: Vec<Vec<IppValue>> = Vec::new();
        let mut retval: IppAttributeList = IppAttributeList::new();
        let mut prev = (0, String::new());

        let header = IppHeader::from_reader(self.reader)?;
        debug!("IPP reply header: {:?}", header);

        stack.push(Vec::new());

        loop {
            let tag = self.reader.read_u8()?;
            if is_delimiter_tag(tag) {
                debug!("Delimiter tag: {:0x}", tag);
                if tag == END_OF_ATTRIBUTES_TAG {
                    let mut val_list = stack.pop().unwrap();
                    let v = if val_list.len() == 1 {val_list.remove(0)} else {IppValue::ListOf(val_list)};
                    retval.add(delimiter, IppAttribute::new(&prev.1, v));
                    break;
                } else {
                    delimiter = tag;
                    continue;
                }
            } else if is_value_tag(tag) {
                // value tag
                let namelen = self.reader.read_u16::<BigEndian>()?;
                let name = self.reader.read_string(namelen as usize)?;
                let value = IppValue::read(tag, &mut self.reader)?;

                debug!("Value tag: {:0x}: {}: {}", tag, name, value);

                if namelen > 0 {
                    // single attribute
                    if prev.0 != 0 {
                        let mut val_list = stack.pop().unwrap();
                        let v = if val_list.len() == 1 {val_list.remove(0)} else {IppValue::ListOf(val_list)};
                        retval.add(delimiter, IppAttribute::new(&prev.1, v));
                        stack.push(Vec::new());
                    }
                    prev = (tag, name);
                }
                match tag {
                    BEG_COLLECTION => {
                        debug!("Begin collection");
                        stack.push(Vec::new())
                    }
                    END_COLLECTION => {
                        debug!("End collection");
                        let arr = stack.pop().unwrap();
                        let mut val_list = stack.last_mut().unwrap();
                        val_list.push(IppValue::Collection(arr));
                    }
                    _ => {
                        stack.last_mut().unwrap().push(value);
                    }
                }
            } else {
                return Err(IppError::TagError(tag))
            }
        }

        Ok(IppParseResult::new(header, retval))
    }
}
