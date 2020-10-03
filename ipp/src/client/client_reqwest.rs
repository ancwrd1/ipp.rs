use futures_util::io::BufReader;
use log::debug;
use reqwest::{Body, ClientBuilder};

use crate::{
    client::{IppClient, IppError, CONNECT_TIMEOUT},
    proto::{parser::IppParser, request::IppRequestResponse},
};

pub(crate) const USER_AGENT: &str = concat!("ipp.rs/", env!("CARGO_PKG_VERSION"), ";reqwest");

pub(super) type ClientError = reqwest::Error;

pub(super) struct ReqwestClient<'a>(pub(super) &'a IppClient);

impl<'a> ReqwestClient<'a> {
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = ClientBuilder::new().connect_timeout(CONNECT_TIMEOUT);

        if let Some(timeout) = self.0.timeout {
            debug!("Setting timeout to {:?}", timeout);
            builder = builder.timeout(timeout);
        }

        if self.0.ignore_tls_errors {
            debug!("Setting dangerous TLS options");
            builder = builder
                .danger_accept_invalid_hostnames(true)
                .danger_accept_invalid_certs(true);
        }

        debug!("Sending request to {}", self.0.uri);

        let response = builder
            .user_agent(USER_AGENT)
            .build()?
            .post(&self.0.uri.to_string())
            .header("content-type", "application/ipp")
            .body(Body::wrap_stream(util::ReaderStream::new(request.into_reader())))
            .send()
            .await?;

        debug!("Response status: {}", response.status());

        match response.status().as_u16() {
            200..=202 => IppParser::new(BufReader::new(util::StreamReader::new(response.bytes_stream())))
                .parse()
                .await
                .map_err(IppError::from),
            other => Err(IppError::RequestError(other)),
        }
    }
}

mod util {
    use std::{
        cmp, io,
        pin::Pin,
        task::{Context, Poll},
    };

    use bytes::Buf;
    use futures_util::{io::AsyncRead, stream::Stream};
    use pin_project::pin_project;

    const CHUNK_SIZE: usize = 32768;

    #[pin_project]
    pub(super) struct StreamReader<S, B> {
        #[pin]
        inner: S,
        chunk: Option<B>,
    }

    impl<S, B, E> StreamReader<S, B>
    where
        S: Stream<Item = Result<B, E>>,
        B: Buf,
        E: Into<reqwest::Error>,
    {
        pub fn new(stream: S) -> StreamReader<S, B> {
            StreamReader {
                inner: stream,
                chunk: None,
            }
        }

        fn has_chunk(self: Pin<&mut Self>) -> bool {
            if let Some(chunk) = self.project().chunk {
                chunk.remaining() > 0
            } else {
                false
            }
        }
    }

    impl<S, B, E> AsyncRead for StreamReader<S, B>
    where
        S: Stream<Item = Result<B, E>>,
        B: Buf,
        E: Into<reqwest::Error>,
    {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
            loop {
                if self.as_mut().has_chunk() {
                    let chunk = self.project().chunk.as_mut().unwrap();
                    let len = cmp::min(chunk.remaining(), buf.len());
                    buf[..len].copy_from_slice(&chunk.bytes()[..len]);
                    chunk.advance(len);
                    return Poll::Ready(Ok(len));
                } else {
                    match futures_util::ready!(self.as_mut().project().inner.poll_next(cx)) {
                        Some(Ok(chunk)) if chunk.remaining() > 0 => {
                            *self.as_mut().project().chunk = Some(chunk);
                        }
                        Some(Err(err)) => {
                            return Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, err.into())))
                        }
                        _ => return Poll::Ready(Ok(0)),
                    }
                }
            }
        }
    }

    #[pin_project]
    pub(super) struct ReaderStream<R> {
        #[pin]
        inner: R,
        buf: Vec<u8>,
    }

    impl<R> ReaderStream<R> {
        pub fn new(reader: R) -> ReaderStream<R> {
            ReaderStream {
                inner: reader,
                buf: vec![0u8; CHUNK_SIZE],
            }
        }
    }

    impl<R: AsyncRead> Stream for ReaderStream<R> {
        type Item = io::Result<Vec<u8>>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
            let mut me = self.project();
            match futures_util::ready!(me.inner.poll_read(cx, &mut me.buf)) {
                Ok(0) => Poll::Ready(None),
                Ok(size) => Poll::Ready(Some(Ok(me.buf[0..size].into()))),
                Err(e) => Poll::Ready(Some(Err(e))),
            }
        }
    }
}
