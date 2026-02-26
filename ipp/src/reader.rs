//!
//! IPP reader
//!
use std::io::{self, Read};

use bytes::Bytes;

#[cfg(feature = "async")]
use futures_util::io::{AsyncRead, AsyncReadExt};

use crate::{IppHeader, model::IppVersion, parser::IppParseError, payload::IppPayload, value::IppName};

#[cfg(feature = "async")]
/// Asynchronous IPP reader contains a set of methods to read from IPP data stream
pub struct AsyncIppReader<R> {
    inner: R,
}

#[cfg(feature = "async")]
impl<R> AsyncIppReader<R>
where
    R: AsyncRead + Send + Sync + Unpin,
{
    /// Create IppReader from AsyncRead instance
    pub fn new(inner: R) -> Self {
        AsyncIppReader { inner }
    }

    async fn read_bytes(&mut self, len: usize) -> io::Result<Bytes> {
        let mut buf = vec![0; len];
        self.inner.read_exact(&mut buf).await?;
        Ok(buf.into())
    }

    async fn read_string(&mut self, len: usize) -> io::Result<String> {
        self.read_bytes(len)
            .await
            .map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    async fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.inner.read_exact(&mut buf).await?;
        Ok(u16::from_be_bytes(buf))
    }

    async fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf).await?;
        Ok(buf[0])
    }

    async fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.inner.read_exact(&mut buf).await?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read tag
    pub async fn read_tag(&mut self) -> io::Result<u8> {
        self.read_u8().await
    }

    /// Read IPP name from [len; name] element
    pub async fn read_name(&mut self) -> Result<IppName, IppParseError> {
        let name_len = self.read_u16().await?;
        self.read_string(name_len as usize).await?.try_into()
    }

    /// Read IPP value from [len; value] element
    pub async fn read_value(&mut self) -> io::Result<Bytes> {
        let value_len = self.read_u16().await?;
        self.read_bytes(value_len as usize).await
    }

    /// Read IPP header
    pub async fn read_header(&mut self) -> io::Result<IppHeader> {
        let version = IppVersion(self.read_u16().await?);
        let operation_status = self.read_u16().await?;
        let request_id = self.read_u32().await?;

        Ok(IppHeader::new(version, operation_status, request_id))
    }

    /// Release the underlying reader
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Convert the remaining inner stream into IppPayload
    pub fn into_payload(self) -> IppPayload
    where
        R: 'static,
    {
        IppPayload::new_async(self.inner)
    }
}

#[cfg(feature = "async")]
impl<R> From<R> for AsyncIppReader<R>
where
    R: AsyncRead + Send + Sync + Unpin,
{
    fn from(r: R) -> Self {
        AsyncIppReader::new(r)
    }
}

/// Synchronous IPP reader contains a set of methods to read from IPP data stream
pub struct IppReader<R> {
    inner: R,
}

impl<R> IppReader<R>
where
    R: Read + Send + Sync,
{
    /// Create IppReader from Read instance
    pub fn new(inner: R) -> Self {
        IppReader { inner }
    }

    fn read_bytes(&mut self, len: usize) -> io::Result<Bytes> {
        let mut buf = vec![0; len];
        self.inner.read_exact(&mut buf)?;
        Ok(buf.into())
    }

    fn read_string(&mut self, len: usize) -> io::Result<String> {
        self.read_bytes(len).map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read tag
    pub fn read_tag(&mut self) -> io::Result<u8> {
        self.read_u8()
    }

    /// Read IPP name from [len; name] element
    pub fn read_name(&mut self) -> Result<IppName, IppParseError> {
        let name_len = self.read_u16()?;
        self.read_string(name_len as usize)?.try_into()
    }

    /// Read IPP value from [len; value] element
    pub fn read_value(&mut self) -> io::Result<Bytes> {
        let value_len = self.read_u16()?;
        self.read_bytes(value_len as usize)
    }

    /// Read IPP header
    pub fn read_header(&mut self) -> io::Result<IppHeader> {
        let version = IppVersion(self.read_u16()?);
        let operation_status = self.read_u16()?;
        let request_id = self.read_u32()?;

        Ok(IppHeader::new(version, operation_status, request_id))
    }

    /// Release the underlying reader
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Convert the remaining inner stream into IppPayload
    pub fn into_payload(self) -> IppPayload
    where
        R: 'static,
    {
        IppPayload::new(self.inner)
    }
}

impl<R> From<R> for IppReader<R>
where
    R: Read + Send + Sync,
{
    fn from(r: R) -> Self {
        IppReader::new(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StatusCode;

    #[test]
    fn test_read_name() {
        let data = io::Cursor::new(vec![0x00, 0x04, b't', b'e', b's', b't']);
        let mut reader = IppReader::new(data);
        let name = reader.read_name().unwrap();
        assert_eq!(name, "test".try_into().unwrap());
    }

    #[test]
    fn test_read_value() {
        let data = io::Cursor::new(vec![0x00, 0x04, b't', b'e', b's', b't']);
        let mut reader = IppReader::new(data);
        let value = reader.read_value().unwrap();
        assert_eq!(value.as_ref(), b"test");
    }

    #[test]
    fn test_read_borrowed_value() {
        let data = vec![0x00, 0x04, b't', b'e', b's', b't'];
        let data = io::Cursor::new(&data);
        let mut reader = IppReader::new(data);
        let value = reader.read_value().unwrap();
        assert_eq!(value.as_ref(), b"test");
    }

    #[test]
    fn test_read_header() {
        let data = io::Cursor::new(vec![0x01, 0x01, 0x04, 0x01, 0x11, 0x22, 0x33, 0x44]);
        let mut reader = IppReader::new(data);
        let header = reader.read_header().unwrap();
        assert_eq!(header.version, IppVersion::v1_1());
        assert_eq!(header.operation_or_status, 0x401);
        assert_eq!(header.request_id, 0x11223344);
        assert_eq!(header.status_code(), StatusCode::ClientErrorForbidden);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_read_name() {
        let data = futures_util::io::Cursor::new(vec![0x00, 0x04, b't', b'e', b's', b't']);
        let mut reader = AsyncIppReader::new(data);
        let name = reader.read_name().await.unwrap();
        assert_eq!(name, "test".try_into().unwrap());
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_read_value() {
        let data = futures_util::io::Cursor::new(vec![0x00, 0x04, b't', b'e', b's', b't']);
        let mut reader = AsyncIppReader::new(data);
        let value = reader.read_value().await.unwrap();
        assert_eq!(value.as_ref(), b"test");
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_read_header() {
        let data = futures_util::io::Cursor::new(vec![0x01, 0x01, 0x04, 0x01, 0x11, 0x22, 0x33, 0x44]);
        let mut reader = AsyncIppReader::new(data);
        let header = reader.read_header().await.unwrap();
        assert_eq!(header.version, IppVersion::v1_1());
        assert_eq!(header.operation_or_status, 0x401);
        assert_eq!(header.request_id, 0x11223344);
        assert_eq!(header.status_code(), StatusCode::ClientErrorForbidden);
    }
}
