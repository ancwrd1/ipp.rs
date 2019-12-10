//!
//! IPP stream parser
//!
use std::{fmt, io};

use bytes::{Buf, Bytes};
use futures_util::io::{AsyncRead, AsyncReadExt};
use log::{debug, error};

use super::{
    model::{DelimiterTag, IppVersion, ValueTag},
    FromPrimitive as _, IppAttribute, IppAttributeGroup, IppAttributes, IppHeader, IppPayload, IppRequestResponse,
    IppValue,
};

/// Parse error enum
#[derive(Debug)]
pub enum IppParseError {
    InvalidTag(u8),
    InvalidVersion,
    InvalidCollection,
    Incomplete,
    IOError(io::Error),
}

impl From<io::Error> for IppParseError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::UnexpectedEof => IppParseError::Incomplete,
            _ => IppParseError::IOError(error),
        }
    }
}

impl fmt::Display for IppParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IppParseError::InvalidTag(tag) => write!(f, "Invalid tag: {}", tag),
            IppParseError::InvalidVersion => write!(f, "Invalid IPP protocol version"),
            IppParseError::InvalidCollection => write!(f, "Invalid IPP collection"),
            IppParseError::Incomplete => write!(f, "Incomplete IPP payload"),
            IppParseError::IOError(err) => write!(f, "{}", err.to_string()),
        }
    }
}

impl std::error::Error for IppParseError {}

// create a single value from one-element list, list otherwise
fn list_or_value(mut list: Vec<IppValue>) -> IppValue {
    if list.len() == 1 {
        list.remove(0)
    } else {
        IppValue::Array(list)
    }
}

/// Asynchronous IPP parser
pub struct IppParser {
    reader: Box<dyn AsyncRead + Send + Unpin>,
    current_group: Option<IppAttributeGroup>,
    last_name: Option<String>,
    context: Vec<Vec<IppValue>>,
    attributes: IppAttributes,
    payload: Option<IppPayload>,
}

impl IppParser {
    /// Create IPP parser from AsyncRead
    pub fn new<R>(reader: R) -> IppParser
    where
        R: AsyncRead + Send + Unpin + 'static,
    {
        IppParser {
            reader: Box::new(reader),
            current_group: None,
            last_name: None,
            context: vec![vec![]],
            attributes: IppAttributes::new(),
            payload: None,
        }
    }

    fn add_last_attribute(&mut self) {
        if let Some(ref last_name) = self.last_name {
            if let Some(val_list) = self.context.pop() {
                if let Some(ref mut group) = self.current_group {
                    group.attributes_mut().insert(
                        last_name.clone(),
                        IppAttribute::new(&last_name, list_or_value(val_list)),
                    );
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

    async fn read_bytes(&mut self, len: usize) -> Result<Bytes, IppParseError> {
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf).await?;
        Ok(buf.into())
    }

    async fn read_string(&mut self, len: usize) -> Result<String, IppParseError> {
        self.read_bytes(len)
            .await
            .map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    async fn read_u16(&mut self) -> Result<u16, IppParseError> {
        self.read_bytes(2).await.map(|mut b| b.get_u16())
    }

    async fn read_u8(&mut self) -> Result<u8, IppParseError> {
        self.read_bytes(1).await.map(|mut b| b.get_u8())
    }

    async fn read_u32(&mut self) -> Result<u32, IppParseError> {
        self.read_bytes(4).await.map(|mut b| b.get_u32())
    }

    async fn parse_value(&mut self, tag: u8) -> Result<(), IppParseError> {
        // value tag
        let name_len = self.read_u16().await?;
        let name = self.read_string(name_len as usize).await?;
        let value_len = self.read_u16().await?;
        let value = self.read_bytes(value_len as usize).await?;

        let ipp_value = IppValue::parse(tag, value)?;

        debug!("Value tag: {:0x}: {}: {}", tag, name, ipp_value);

        if name_len > 0 {
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
        let version = IppVersion::from_u16(self.read_u16().await?).ok_or_else(|| IppParseError::InvalidVersion)?;
        let operation_status = self.read_u16().await?;
        let request_id = self.read_u32().await?;

        let header = IppHeader::new(version, operation_status, request_id);
        debug!("IPP header: {:?}", header);

        loop {
            match self.read_u8().await? {
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

        let mut buf = [0u8; 32768];
        let size = self.reader.read(&mut buf).await?;
        if size > 0 {
            debug!("Payload detected");
            let cursor = futures_util::io::Cursor::new(buf[..size].to_vec());
            self.payload = Some(cursor.chain(self.reader).into());
        }

        Ok(IppRequestResponse {
            header,
            attributes: self.attributes,
            payload: self.payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_attributes() {
        let data = &[1, 1, 0, 0, 0, 0, 0, 0, 3];
        let result = futures::executor::block_on(IppParser::new(futures::io::Cursor::new(data)).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        assert!(res.attributes.groups().is_empty());
    }

    #[test]
    fn test_parse_single_value() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
        ];
        let result = futures::executor::block_on(IppParser::new(futures::io::Cursor::new(data)).parse());
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
        let result = futures::executor::block_on(IppParser::new(futures::io::Cursor::new(data)).parse());
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
        let result = futures::executor::block_on(IppParser::new(futures::io::Cursor::new(data)).parse());
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

        let result = futures::executor::block_on(IppParser::new(futures::io::Cursor::new(data)).parse());
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x1234_5678));

        match res.payload {
            Some(payload) => {
                let mut cursor = futures::io::Cursor::new(Vec::new());
                futures::executor::block_on(futures::io::copy(payload.into_inner(), &mut cursor)).unwrap();
                assert_eq!(cursor.into_inner(), b"foo");
            }
            _ => panic!("Wrong payload!"),
        }
    }
}
