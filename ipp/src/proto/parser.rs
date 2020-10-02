//!
//! IPP stream parser
//!
use std::io;

use futures_util::io::AsyncRead;
use log::{debug, error};

use crate::proto::reader::IppReader;

use super::{
    model::{DelimiterTag, ValueTag},
    FromPrimitive as _, IppAttribute, IppAttributeGroup, IppAttributes, IppRequestResponse, IppValue,
};

/// Parse error enum
#[derive(Debug, thiserror::Error)]
pub enum IppParseError {
    #[error("Invalid tag: {0}")]
    InvalidTag(u8),

    #[error("Invalid IPP collection")]
    InvalidCollection,

    #[error(transparent)]
    IOError(#[from] io::Error),
}

// create a single value from one-element list, list otherwise
fn list_or_value(mut list: Vec<IppValue>) -> IppValue {
    if list.len() == 1 {
        list.remove(0)
    } else {
        IppValue::Array(list)
    }
}

/// Asynchronous IPP parser
pub struct IppParser<R> {
    reader: IppReader<R>,
    current_group: Option<IppAttributeGroup>,
    last_name: Option<String>,
    context: Vec<Vec<IppValue>>,
    attributes: IppAttributes,
}

impl<R> IppParser<R>
where
    R: 'static + AsyncRead + Send + Sync + Unpin,
{
    /// Create IPP parser from AsyncIppReader
    pub fn new<T>(reader: T) -> IppParser<R>
    where
        T: Into<IppReader<R>>,
    {
        IppParser {
            reader: reader.into(),
            current_group: None,
            last_name: None,
            context: vec![vec![]],
            attributes: IppAttributes::new(),
        }
    }

    fn add_last_attribute(&mut self) {
        if let Some(last_name) = self.last_name.take() {
            if let Some(val_list) = self.context.pop() {
                if let Some(ref mut group) = self.current_group {
                    let attr = IppAttribute::new(&last_name, list_or_value(val_list));
                    group.attributes_mut().insert(last_name, attr);
                }
            }
            self.context.push(vec![]);
        }
    }

    fn parse_delimiter(&mut self, tag: u8) -> Result<DelimiterTag, IppParseError> {
        debug!("Delimiter tag: {:0x}", tag);

        let tag = DelimiterTag::from_u8(tag).ok_or_else(|| IppParseError::InvalidTag(tag))?;
        if tag == DelimiterTag::EndOfAttributes {
            self.add_last_attribute();
        }

        if let Some(group) = self.current_group.take() {
            self.attributes.groups_mut().push(group);
        }

        self.current_group = Some(IppAttributeGroup::new(tag));

        Ok(tag)
    }

    async fn parse_value(&mut self, tag: u8) -> Result<(), IppParseError> {
        // value tag
        let name = self.reader.read_name().await?;
        let value = self.reader.read_value().await?;

        let ipp_value = IppValue::parse(tag, value)?;

        debug!("Value tag: {:0x}: {}: {}", tag, name, ipp_value);

        if !name.is_empty() {
            // single attribute or begin of array
            self.add_last_attribute();
            // store it as a previous attribute
            self.last_name = Some(name);
        }
        if tag == ValueTag::BegCollection as u8 {
            // start new collection in the stack
            debug!("Begin collection");
            match ipp_value {
                IppValue::Other { ref data, .. } if data.is_empty() => {}
                _ => {
                    error!("Invalid begin collection attribute");
                    return Err(IppParseError::InvalidCollection);
                }
            }
            self.context.push(vec![]);
        } else if tag == ValueTag::EndCollection as u8 {
            // get collection from the stack and add it to the previous element
            debug!("End collection");
            match ipp_value {
                IppValue::Other { ref data, .. } if data.is_empty() => {}
                _ => {
                    error!("Invalid end collection attribute");
                    return Err(IppParseError::InvalidCollection);
                }
            }
            if let Some(arr) = self.context.pop() {
                if let Some(val_list) = self.context.last_mut() {
                    val_list.push(IppValue::Collection(arr));
                }
            }
        } else if let Some(val_list) = self.context.last_mut() {
            // add attribute to the current collection
            val_list.push(ipp_value);
        }
        Ok(())
    }

    /// Parse IPP stream
    pub async fn parse(mut self) -> Result<IppRequestResponse, IppParseError> {
        let header = self.reader.read_header().await?;
        debug!("IPP header: {:?}", header);

        loop {
            match self.reader.read_tag().await? {
                tag @ 0x01..=0x05 => {
                    if self.parse_delimiter(tag)? == DelimiterTag::EndOfAttributes {
                        break;
                    }
                }
                tag @ 0x10..=0x4a => self.parse_value(tag).await?,
                tag => {
                    return Err(IppParseError::InvalidTag(tag));
                }
            }
        }

        Ok(IppRequestResponse {
            header,
            attributes: self.attributes,
            payload: self.reader.into_payload(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_attributes() {
        let data = &[1, 1, 0, 0, 0, 0, 0, 0, 3];
        let result =
            futures::executor::block_on(IppParser::new(IppReader::new(futures::io::Cursor::new(data))).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        assert!(res.attributes.groups().is_empty());
    }

    #[test]
    fn test_parse_single_value() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
        ];
        let result =
            futures::executor::block_on(IppParser::new(IppReader::new(futures::io::Cursor::new(data))).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));
    }

    #[test]
    fn test_parse_array() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78,
            0x21, 0x00, 0x00, 0x00, 0x04, 0x77, 0x65, 0x43, 0x21, 3,
        ];
        let result =
            futures::executor::block_on(IppParser::new(IppReader::new(futures::io::Cursor::new(data))).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(
            attr.value().as_array(),
            Some(&vec![IppValue::Integer(0x1234_5678), IppValue::Integer(0x7765_4321)])
        );
    }

    #[test]
    fn test_parse_collection() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x21, 0, 0, 0, 4, 0x12, 0x34, 0x56,
            0x78, 0x44, 0, 0, 0, 3, b'k', b'e', b'y', 0x37, 0, 0, 0, 0, 3,
        ];
        let result =
            futures::executor::block_on(IppParser::new(IppReader::new(futures::io::Cursor::new(data))).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("coll").unwrap();
        assert_eq!(
            attr.value().as_collection(),
            Some(&vec![
                IppValue::Integer(0x1234_5678),
                IppValue::Keyword("key".to_owned())
            ])
        );
    }

    #[test]
    fn test_parser_with_payload() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
            b'f', b'o', b'o',
        ];

        let result =
            futures::executor::block_on(IppParser::new(IppReader::new(futures::io::Cursor::new(data))).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));

        let mut cursor = futures::io::Cursor::new(Vec::new());
        futures::executor::block_on(futures::io::copy(res.payload, &mut cursor)).unwrap();
        assert_eq!(cursor.into_inner(), b"foo");
    }
}
