//!
//! IPP stream parser
//!
use std::io::{self, Read};

use bytes::Bytes;
use log::{error, trace};

#[cfg(feature = "async")]
use {crate::reader::AsyncIppReader, futures_util::io::AsyncRead};

use crate::{
    attribute::{IppAttribute, IppAttributeGroup, IppAttributes},
    model::{DelimiterTag, ValueTag},
    reader::IppReader,
    request::IppRequestResponse,
    value::IppValue,
    FromPrimitive as _,
};

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

/// Parse error enum
#[derive(Debug, thiserror::Error)]
pub enum IppParseError {
    #[error("Invalid tag: {0}")]
    InvalidTag(u8),

    #[error("Invalid IPP collection")]
    InvalidCollection,

    #[error(transparent)]
    IoError(#[from] io::Error),
}

// create a single value from one-element list, list otherwise
fn list_or_value(mut list: Vec<IppValue>) -> IppValue {
    if list.len() == 1 {
        list.remove(0)
    } else {
        IppValue::Array(list)
    }
}

struct ParserState {
    current_group: Option<IppAttributeGroup>,
    last_name: Option<String>,
    context: Vec<Vec<IppValue>>,
    attributes: IppAttributes,
}

impl ParserState {
    fn new() -> Self {
        ParserState {
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
        trace!("Delimiter tag: {:0x}", tag);

        let tag = DelimiterTag::from_u8(tag).ok_or(IppParseError::InvalidTag(tag))?;
        if tag == DelimiterTag::EndOfAttributes {
            self.add_last_attribute();
        }

        if let Some(group) = self.current_group.take() {
            self.attributes.groups_mut().push(group);
        }

        self.current_group = Some(IppAttributeGroup::new(tag));

        Ok(tag)
    }

    fn parse_value(&mut self, tag: u8, name: String, value: Bytes) -> Result<(), IppParseError> {
        let ipp_value = IppValue::parse(tag, value)?;

        trace!("Value tag: {:0x}: {}: {}", tag, name, ipp_value);

        if !name.is_empty() {
            // single attribute or begin of array
            self.add_last_attribute();
            // store it as a previous attribute
            self.last_name = Some(name);
        }
        if tag == ValueTag::BegCollection as u8 {
            // start new collection in the stack
            trace!("Begin collection");
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
            trace!("End collection");
            match ipp_value {
                IppValue::Other { ref data, .. } if data.is_empty() => {}
                _ => {
                    error!("Invalid end collection attribute");
                    return Err(IppParseError::InvalidCollection);
                }
            }
            if let Some(arr) = self.context.pop() {
                if let Some(val_list) = self.context.last_mut() {
                    let mut map: std::collections::HashMap<String, IppValue> = std::collections::HashMap::new();
                    for idx in (0..arr.len()).step_by(2) {
                        match (arr.get(idx), arr.get(idx + 1)) {
                            (Some(IppValue::MemberAttrName(k)), Some(v)) => {
                                map.insert(k.to_string(), v.clone());
                            },
                            _ => {}
                        }
                    }
                    val_list.push(IppValue::Collection(map));
                }
            }
        } else if let Some(val_list) = self.context.last_mut() {
            // add attribute to the current collection
            val_list.push(ipp_value);
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
/// Asynchronous IPP parser
pub struct AsyncIppParser<R> {
    reader: AsyncIppReader<R>,
    state: ParserState,
}

#[cfg(feature = "async")]
impl<R> AsyncIppParser<R>
where
    R: 'static + AsyncRead + Send + Sync + Unpin,
{
    /// Create IPP parser from AsyncIppReader
    pub fn new<T>(reader: T) -> AsyncIppParser<R>
    where
        T: Into<AsyncIppReader<R>>,
    {
        AsyncIppParser {
            reader: reader.into(),
            state: ParserState::new(),
        }
    }

    async fn parse_value(&mut self, tag: u8) -> Result<(), IppParseError> {
        // value tag
        let name = self.reader.read_name().await?;
        let value = self.reader.read_value().await?;

        self.state.parse_value(tag, name, value)
    }

    /// Parse IPP stream
    pub async fn parse(mut self) -> Result<IppRequestResponse, IppParseError> {
        let header = self.reader.read_header().await?;
        trace!("IPP header: {:?}", header);

        loop {
            match self.reader.read_tag().await? {
                tag @ 0x01..=0x05 => {
                    if self.state.parse_delimiter(tag)? == DelimiterTag::EndOfAttributes {
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
            attributes: self.state.attributes,
            payload: self.reader.into_payload(),
        })
    }
}

/// Synchronous IPP parser
pub struct IppParser<R> {
    reader: IppReader<R>,
    state: ParserState,
}

impl<R> IppParser<R>
where
    R: 'static + Read + Send + Sync,
{
    /// Create IPP parser from IppReader
    pub fn new<T>(reader: T) -> IppParser<R>
    where
        T: Into<IppReader<R>>,
    {
        IppParser {
            reader: reader.into(),
            state: ParserState::new(),
        }
    }

    fn parse_value(&mut self, tag: u8) -> Result<(), IppParseError> {
        // value tag
        let name = self.reader.read_name()?;
        let value = self.reader.read_value()?;

        self.state.parse_value(tag, name, value)
    }

    /// Parse IPP stream
    pub fn parse(mut self) -> Result<IppRequestResponse, IppParseError> {
        let header = self.reader.read_header()?;
        trace!("IPP header: {:?}", header);

        loop {
            match self.reader.read_tag()? {
                tag @ 0x01..=0x05 => {
                    if self.state.parse_delimiter(tag)? == DelimiterTag::EndOfAttributes {
                        break;
                    }
                }
                tag @ 0x10..=0x4a => self.parse_value(tag)?,
                tag => {
                    return Err(IppParseError::InvalidTag(tag));
                }
            }
        }

        Ok(IppRequestResponse {
            header,
            attributes: self.state.attributes,
            payload: self.reader.into_payload(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_parse_no_attributes() {
        let data = &[1, 1, 0, 0, 0, 0, 0, 0, 3];
        let result = AsyncIppParser::new(AsyncIppReader::new(futures_util::io::Cursor::new(data)))
            .parse()
            .await;
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        assert!(res.attributes.groups().is_empty());
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_parse_single_value() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
        ];
        let result = AsyncIppParser::new(AsyncIppReader::new(futures_util::io::Cursor::new(data)))
            .parse()
            .await;
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_parse_array() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78,
            0x21, 0x00, 0x00, 0x00, 0x04, 0x77, 0x65, 0x43, 0x21, 3,
        ];
        let result = AsyncIppParser::new(AsyncIppReader::new(futures_util::io::Cursor::new(data)))
            .parse()
            .await;
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(
            attr.value().as_array(),
            Some(&vec![IppValue::Integer(0x1234_5678), IppValue::Integer(0x7765_4321)])
        );
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_parse_collection() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x4a, 0, 0, 0, 4, b'a', b'b', b'c', b'd',
            0x44, 0, 0, 0, 3, b'k', b'e', b'y', 0x37, 0, 0, 0, 0, 3,
        ];
        let result = AsyncIppParser::new(AsyncIppReader::new(futures_util::io::Cursor::new(data)))
            .parse()
            .await;
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
            attr.value(),
            &IppValue::Collection(hashmap![
                "abcd".to_string() => IppValue::Keyword("key".to_owned())
            ])
        );
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_parse_with_payload() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
            b'f', b'o', b'o',
        ];

        let result = AsyncIppParser::new(AsyncIppReader::new(futures_util::io::Cursor::new(data)))
            .parse()
            .await;
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));

        let mut cursor = futures_util::io::Cursor::new(Vec::new());
        futures_executor::block_on(futures_util::io::copy(res.payload, &mut cursor)).unwrap();
        assert_eq!(cursor.into_inner(), b"foo");
    }

    #[test]
    fn test_parse_no_attributes() {
        let data = &[1, 1, 0, 0, 0, 0, 0, 0, 3];
        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        assert!(res.attributes.groups().is_empty());
    }

    #[test]
    fn test_parse_single_value() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
        ];
        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));
    }

    #[test]
    fn test_parse_array() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78,
            0x21, 0x00, 0x00, 0x00, 0x04, 0x77, 0x65, 0x43, 0x21, 3,
        ];
        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(
            attr.value().as_array(),
            Some(&vec![IppValue::Integer(0x1234_5678), IppValue::Integer(0x7765_4321)])
        );
    }

    #[test]
    fn test_parse_collection() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x4a, 0, 0, 0, 4, b'a', b'b', b'c', b'd',
            0x44, 0, 0, 0, 3, b'k', b'e', b'y', 0x37, 0, 0, 0, 0, 3,
        ];
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
            attr.value(),
            &IppValue::Collection(hashmap![
                "abcd".to_string() => IppValue::Keyword("key".to_owned())
            ])
        );
    }

    #[test]
    fn test_parser_with_payload() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
            b'f', b'o', b'o',
        ];

        let result = IppParser::new(IppReader::new(io::Cursor::new(data))).parse();
        assert!(result.is_ok());

        let mut res = result.ok().unwrap();
        let attrs = res
            .attributes
            .groups_of(DelimiterTag::PrinterAttributes)
            .next()
            .unwrap()
            .attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));

        let mut cursor = io::Cursor::new(Vec::new());
        io::copy(&mut res.payload, &mut cursor).unwrap();
        assert_eq!(cursor.into_inner(), b"foo");
    }
}
