//!
//! Attribute-related structs
//!
use byteorder::{BigEndian, WriteBytesExt};
use std::collections::HashMap;
use std::io::Write;

use consts::attribute::*;
use consts::tag::*;
use value::IppValue;
use Result;

const HEADER_ATTRS: [&str; 3] = [ATTRIBUTES_CHARSET, ATTRIBUTES_NATURAL_LANGUAGE, PRINTER_URI];

fn is_header_attr(attr: &str) -> bool {
    HEADER_ATTRS.into_iter().any(|&at| at == attr)
}

/// `IppAttribute` represents an IPP attribute
#[derive(Clone, Debug)]
pub struct IppAttribute {
    /// Attribute name
    name: String,
    /// Attribute value
    value: IppValue,
}

impl IppAttribute {
    /// Create new instance of the attribute
    ///
    /// * `name` - Attribute name<br/>
    /// * `value` - Attribute value<br/>
    pub fn new(name: &str, value: IppValue) -> IppAttribute {
        IppAttribute {
            name: name.to_string(),
            value,
        }
    }

    /// Return attribute name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return attribute value
    pub fn value(&self) -> &IppValue {
        &self.value
    }

    /// Serialize attribute into binary stream
    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        let mut retval = 0;

        writer.write_u8(self.value.to_tag() as u8)?;
        retval += 1;

        writer.write_u16::<BigEndian>(self.name.len() as u16)?;
        retval += 2;

        writer.write_all(self.name.as_bytes())?;
        retval += self.name.len();

        retval += self.value.write(writer)?;

        Ok(retval)
    }
}

/// Attribute list indexed by group and name
#[derive(Clone, Default, Debug)]
pub struct IppAttributeList {
    attributes: HashMap<DelimiterTag, HashMap<String, IppAttribute>>,
}

impl IppAttributeList {
    /// Create attribute list
    pub fn new() -> IppAttributeList {
        IppAttributeList::default()
    }

    /// Add attribute to the list
    ///
    /// * `group` - delimiter group<br/>
    /// * `attribute` - attribute to add<br/>
    pub fn add(&mut self, group: DelimiterTag, attribute: IppAttribute) {
        self.attributes.entry(group).or_insert_with(HashMap::new);
        let opt = self.attributes.get_mut(&group).unwrap();
        opt.insert(attribute.name().to_string(), attribute);
    }

    /// Get attribute from the list
    pub fn get(&self, group: DelimiterTag, name: &str) -> Option<&IppAttribute> {
        self.attributes
            .get(&group)
            .and_then(|attrs| attrs.get(name))
    }

    /// Get attribute list for a group
    pub fn get_group(&self, group: DelimiterTag) -> Option<&HashMap<String, IppAttribute>> {
        self.attributes.get(&group)
    }

    /// Get printer attributes
    pub fn get_printer_attributes(&self) -> Option<&HashMap<String, IppAttribute>> {
        self.get_group(DelimiterTag::PrinterAttributes)
    }

    /// Get job attributes
    pub fn get_job_attributes(&self) -> Option<&HashMap<String, IppAttribute>> {
        self.get_group(DelimiterTag::JobAttributes)
    }

    /// Get operation attributes
    pub fn get_operation_attributes(&self) -> Option<&HashMap<String, IppAttribute>> {
        self.get_group(DelimiterTag::OperationAttributes)
    }

    /// Serialize attribute list into binary stream
    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        // first send the header attributes
        writer.write_u8(DelimiterTag::OperationAttributes as u8)?;

        let mut retval = 1;

        for hdr in &HEADER_ATTRS {
            if let Some(attr) = self.get(DelimiterTag::OperationAttributes, hdr) {
                retval += attr.write(writer)?
            }
        }

        // now the rest
        for hdr in &[
            DelimiterTag::OperationAttributes,
            DelimiterTag::JobAttributes,
            DelimiterTag::PrinterAttributes,
        ] {
            let group = *hdr;
            if let Some(attrs) = self.attributes.get(&group) {
                if group != DelimiterTag::OperationAttributes {
                    writer.write_u8(group as u8)?;
                    retval += 1;
                }
                for (_, attr) in attrs.iter().filter(|&(_, v)| {
                    group != DelimiterTag::OperationAttributes || !is_header_attr(v.name())
                }) {
                    retval += attr.write(writer)?;
                }
            }
        }
        writer.write_u8(DelimiterTag::EndOfAttributes as u8)?;
        retval += 1;

        Ok(retval)
    }
}
