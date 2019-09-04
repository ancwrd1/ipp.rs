//!
//! IPP stream parser
//!
use std::{
    fmt,
    io::{self, Read},
};

use byteorder::{BigEndian, ReadBytesExt};
use futures::{try_ready, Async, Future, Poll, Stream};
use log::{debug, error};
use num_traits::FromPrimitive;

use crate::{ipp::*, IppAttribute, IppAttributeGroup, IppAttributes, IppHeader, IppReadExt, IppValue, PayloadKind};

/// Parse error enum
#[derive(Debug)]
pub enum ParseError {
    InvalidTag(u8),
    InvalidVersion,
    InvalidCollection,
    Incomplete,
    IOError(io::Error),
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::UnexpectedEof => ParseError::Incomplete,
            _ => ParseError::IOError(error),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidTag(tag) => write!(f, "Invalid tag: {}", tag),
            ParseError::InvalidVersion => write!(f, "Invalid IPP protocol version"),
            ParseError::InvalidCollection => write!(f, "Invalid IPP collection"),
            ParseError::Incomplete => write!(f, "Incomplete IPP payload"),
            ParseError::IOError(err) => write!(f, "{}", err.to_string()),
        }
    }
}

impl std::error::Error for ParseError {}

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
    pub payload: Option<PayloadKind>,
}

impl IppParseResult {
    fn new(header: IppHeader, attributes: IppAttributes) -> IppParseResult {
        IppParseResult {
            header,
            attributes,
            payload: None,
        }
    }
}

/// IPP parser implementation
pub struct IppParser<'a> {
    reader: &'a mut dyn Read,
    current_group: Option<IppAttributeGroup>,
    last_name: Option<String>,
    context: Vec<Vec<IppValue>>,
    attributes: IppAttributes,
}

impl<'a> IppParser<'a> {
    /// Create IPP parser using the given Read
    pub fn new(reader: &'a mut dyn Read) -> IppParser<'a> {
        IppParser {
            reader,
            current_group: None,
            last_name: None,
            context: vec![vec![]],
            attributes: IppAttributes::new(),
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

    fn parse_delimiter(&mut self, tag: u8) -> Result<DelimiterTag, ParseError> {
        debug!("Delimiter tag: {:0x}", tag);

        let tag = DelimiterTag::from_u8(tag).ok_or_else(|| ParseError::InvalidTag(tag))?;
        if tag == DelimiterTag::EndOfAttributes {
            self.add_last_attribute();
        }

        if let Some(group) = self.current_group.take() {
            self.attributes.groups_mut().push(group);
        }

        self.current_group = Some(IppAttributeGroup::new(tag));

        Ok(tag)
    }

    fn parse_value(&mut self, tag: u8) -> Result<(), ParseError> {
        // value tag
        let namelen = self.reader.read_u16::<BigEndian>()?;
        let name = self.reader.read_string(namelen as usize)?;
        let value = IppValue::read(tag, &mut self.reader)?;

        debug!("Value tag: {:0x}: {}: {}", tag, name, value);

        if namelen > 0 {
            // single attribute or begin of array
            self.add_last_attribute();
            // store it as a previous attribute
            self.last_name = Some(name);
        }
        if tag == ValueTag::BegCollection as u8 {
            // start new collection in the stack
            debug!("Begin collection");
            match value {
                IppValue::Other { ref data, .. } if data.is_empty() => {}
                _ => {
                    error!("Invalid begin collection attribute");
                    return Err(ParseError::InvalidCollection);
                }
            }
            self.context.push(vec![]);
        } else if tag == ValueTag::EndCollection as u8 {
            // get collection from the stack and add it to the previous element
            debug!("End collection");
            match value {
                IppValue::Other { ref data, .. } if data.is_empty() => {}
                _ => {
                    error!("Invalid end collection attribute");
                    return Err(ParseError::InvalidCollection);
                }
            }
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
    pub fn parse(mut self) -> Result<IppParseResult, ParseError> {
        let header = IppHeader::from_reader(self.reader)?;
        debug!("IPP header: {:?}", header);

        loop {
            match self.reader.read_u8()? {
                tag @ 0x01..=0x05 => {
                    if self.parse_delimiter(tag)? == DelimiterTag::EndOfAttributes {
                        break;
                    }
                }
                tag @ 0x10..=0x4a => self.parse_value(tag)?,
                tag => {
                    return Err(ParseError::InvalidTag(tag));
                }
            }
        }

        Ok(IppParseResult::new(header, self.attributes))
    }
}

enum AsyncParseState {
    Headers(Vec<u8>),
    Payload(IppParseResult),
}

/// Asynchronous IPP parser using Streams
pub struct AsyncIppParser<I, E> {
    state: AsyncParseState,
    stream: Box<dyn Stream<Item = I, Error = E> + Send>,
}

impl<I, E> Future for AsyncIppParser<I, E>
where
    I: AsRef<[u8]>,
    ParseError: From<E>,
{
    type Item = IppParseResult;
    type Error = ParseError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Some(item) = try_ready!(self.stream.poll()) {
            match self.state {
                AsyncParseState::Headers(ref mut buffer) => {
                    buffer.extend_from_slice(item.as_ref());
                    let length = buffer.len() as u64;

                    let mut reader = io::Cursor::new(buffer);
                    let parser = IppParser::new(&mut reader);

                    match parser.parse() {
                        Ok(mut result) => {
                            debug!("Parse ok, proceeding to payload state");
                            if reader.position() < length {
                                debug!("Adding residual payload from this chunk");
                                let mut temp = tempfile::NamedTempFile::new()?;
                                io::copy(&mut reader, &mut temp)?;
                                result.payload = Some(PayloadKind::ReceivedData(temp));
                            }
                            self.state = AsyncParseState::Payload(result);
                        }
                        Err(ParseError::Incomplete) => {
                            debug!("Incomplete request, awaiting for more data");
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                AsyncParseState::Payload(ref mut result) => {
                    let mut reader = io::Cursor::new(&item);
                    match result.payload {
                        Some(PayloadKind::ReceivedData(ref mut file)) => {
                            debug!(
                                "Payload chunk received, appending to existing file: {}",
                                file.path().display()
                            );
                            io::copy(&mut reader, file)?;
                        }
                        None => {
                            let mut temp = tempfile::NamedTempFile::new()?;
                            debug!("Payload chunk received, creating new file: {}", temp.path().display());
                            io::copy(&mut reader, &mut temp)?;
                            result.payload = Some(PayloadKind::ReceivedData(temp));
                        }
                        _ => panic!("Should not happen"),
                    }
                }
            }
        }

        match self.state {
            AsyncParseState::Headers(_) => Err(ParseError::Incomplete),
            AsyncParseState::Payload(ref mut result) => {
                debug!("Parsing finished, payload: {}", result.payload.is_some());
                Ok(Async::Ready(IppParseResult {
                    header: result.header.clone(),
                    attributes: result.attributes.clone(),
                    payload: result.payload.take(),
                }))
            }
        }
    }
}

impl<I, E> From<Box<dyn Stream<Item = I, Error = E> + Send>> for AsyncIppParser<I, E> {
    /// Construct asynchronous parser from the stream
    fn from(s: Box<dyn Stream<Item = I, Error = E> + Send>) -> AsyncIppParser<I, E> {
        AsyncIppParser {
            state: AsyncParseState::Headers(Vec::new()),
            stream: s,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_parse_no_attributes() {
        let data = &[1, 1, 0, 0, 0, 0, 0, 0, 3];
        let result = IppParser::new(&mut Cursor::new(data)).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        assert!(res.attributes.groups().is_empty());
    }

    #[test]
    fn test_parse_single_value() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
        ];
        let result = IppParser::new(&mut Cursor::new(data)).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x12345678));
    }

    #[test]
    fn test_parse_list() {
        let data = &[
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78,
            0x21, 0x00, 0x00, 0x00, 0x04, 0x77, 0x65, 0x43, 0x21, 3,
        ];
        let result = IppParser::new(&mut Cursor::new(data)).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(
            attr.value().as_listof(),
            Some(&vec![IppValue::Integer(0x12345678), IppValue::Integer(0x77654321)])
        );
    }

    #[test]
    fn test_parse_collection() {
        let data = vec![
            1, 1, 0, 0, 0, 0, 0, 0, 4, 0x34, 0, 4, b'c', b'o', b'l', b'l', 0, 0, 0x21, 0, 0, 0, 4, 0x12, 0x34, 0x56,
            0x78, 0x44, 0, 0, 0, 3, b'k', b'e', b'y', 0x37, 0, 0, 0, 0, 3,
        ];
        let result = IppParser::new(&mut Cursor::new(data)).parse();
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("coll").unwrap();
        assert_eq!(
            attr.value().as_collection(),
            Some(&vec![
                IppValue::Integer(0x12345678),
                IppValue::Keyword("key".to_owned())
            ])
        );
    }

    #[test]
    fn test_async_parser_with_payload() {
        // split IPP into arbitrary chunks
        let data = vec![
            vec![1, 1, 0],
            vec![0, 0, 0, 0, 0, 4],
            vec![
                0x21, 0x00, 0x04, b't', b'e', b's', b't', 0x00, 0x04, 0x12, 0x34, 0x56, 0x78, 3,
            ],
            vec![b'f'],
            vec![b'o', b'o'],
        ];

        let source: Box<dyn Stream<Item = Vec<u8>, Error = io::Error> + Send> =
            Box::new(futures::stream::iter_ok::<_, io::Error>(data));

        let parser = AsyncIppParser::from(source);

        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(parser);
        assert!(result.is_ok());

        let res = result.ok().unwrap();
        let attrs = res.attributes.groups_of(DelimiterTag::PrinterAttributes)[0].attributes();
        let attr = attrs.get("test").unwrap();
        assert_eq!(attr.value().as_integer(), Some(&0x12345678));

        match res.payload {
            Some(PayloadKind::ReceivedData(f)) => {
                let foo = std::fs::read_to_string(f.path()).unwrap();
                assert_eq!(foo, "foo");
            }
            _ => panic!("Wrong payload!"),
        }
    }

}
