//!
//! IPP reader
//!
use std::io;

use bytes::Bytes;
use futures_util::io::{AsyncRead, AsyncReadExt};

use crate::proto::{IppHeader, IppPayload, IppVersion};

/// IPP reader contains a set of methods to read from IPP data stream
pub struct IppReader<R> {
    inner: R,
}

impl<R> IppReader<R>
where
    R: 'static + AsyncRead + Send + Sync + Unpin,
{
    /// Create IppReader from AsyncRead instance
    pub fn new(inner: R) -> Self {
        IppReader { inner }
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
    pub async fn read_name(&mut self) -> io::Result<String> {
        let name_len = self.read_u16().await?;
        self.read_string(name_len as usize).await
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

    /// Convert the remaining inner stream into IppPayload
    pub fn into_payload(self) -> IppPayload {
        IppPayload::new(self.inner)
    }
}

impl<R> From<R> for IppReader<R>
where
    R: 'static + AsyncRead + Send + Sync + Unpin,
{
    fn from(r: R) -> Self {
        IppReader::new(r)
    }
}
