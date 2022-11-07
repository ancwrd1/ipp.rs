//!
//! IPP payload
//!
use std::io::{self, Read};

#[cfg(feature = "async")]
use {
    futures_util::io::{AllowStdIo, AsyncRead, AsyncReadExt},
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
};

enum PayloadKind {
    #[cfg(feature = "async")]
    Async(Box<dyn AsyncRead + Send + Sync + Unpin>),
    Sync(Box<dyn Read + Send + Sync>),
    Empty,
}

/// IPP payload
pub struct IppPayload {
    inner: PayloadKind,
}

impl IppPayload {
    /// Create empty payload
    pub fn empty() -> Self {
        IppPayload {
            inner: PayloadKind::Empty,
        }
    }

    #[cfg(feature = "async")]
    /// Create an async payload from the AsyncRead object
    pub fn new_async<R>(r: R) -> Self
    where
        R: 'static + AsyncRead + Send + Sync + Unpin,
    {
        IppPayload {
            inner: PayloadKind::Async(Box::new(r)),
        }
    }

    /// Create a sync payload from the Read object
    pub fn new<R>(r: R) -> Self
    where
        R: 'static + Read + Send + Sync,
    {
        IppPayload {
            inner: PayloadKind::Sync(Box::new(r)),
        }
    }
}

impl Default for IppPayload {
    fn default() -> Self {
        Self {
            inner: PayloadKind::Empty,
        }
    }
}

#[cfg(feature = "async")]
impl AsyncRead for IppPayload {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        match self.inner {
            PayloadKind::Async(ref mut inner) => Pin::new(&mut *inner).poll_read(cx, buf),
            PayloadKind::Sync(ref mut inner) => Pin::new(&mut AllowStdIo::new(inner)).poll_read(cx, buf),
            PayloadKind::Empty => Poll::Ready(Ok(0)),
        }
    }
}

impl Read for IppPayload {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner {
            #[cfg(feature = "async")]
            PayloadKind::Async(ref mut inner) => futures_executor::block_on(inner.read(buf)),
            PayloadKind::Sync(ref mut inner) => inner.read(buf),
            PayloadKind::Empty => Ok(0),
        }
    }
}
