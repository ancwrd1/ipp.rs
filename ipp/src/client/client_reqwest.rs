use futures_util::io::BufReader;
use log::debug;
use reqwest::{Body, ClientBuilder};

use crate::{
    client::{IppClient, IppError, CONNECT_TIMEOUT},
    proto::{parser::IppParser, request::IppRequestResponse},
};

const USER_AGENT: &str = concat!("ipp.rs/", env!("CARGO_PKG_VERSION"), ";reqwest");

pub(super) type ClientError = reqwest::Error;

pub(super) struct ReqwestClient<'a>(pub(super) &'a IppClient);

impl<'a> ReqwestClient<'a> {
    pub async fn send_request(&self, request: IppRequestResponse) -> Result<IppRequestResponse, IppError> {
        let mut builder = ClientBuilder::new().connect_timeout(CONNECT_TIMEOUT);

        if let Some(timeout) = self.0.request_timeout {
            debug!("Setting request timeout to {:?}", timeout);
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
                    let me = self.as_mut().project();
                    match futures_util::ready!(me.inner.poll_next(cx)) {
                        Some(Ok(chunk)) => {
                            *me.chunk = Some(chunk);
                        }
                        Some(Err(err)) => {
                            return Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, err.into())))
                        }
                        None => return Poll::Ready(Ok(0)),
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

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures_util::{io::AsyncReadExt, stream::iter};

    #[test]
    fn test_stream_reader() {
        let _ = futures::executor::block_on(async {
            let stream = iter(vec![
                Result::Ok::<_, reqwest::Error>(Bytes::from_static(&[])),
                Ok(Bytes::from_static(&[0, 1, 2, 3])),
                Ok(Bytes::from_static(&[])),
                Ok(Bytes::from_static(&[4, 5, 6, 7])),
                Ok(Bytes::from_static(&[])),
                Ok(Bytes::from_static(&[8, 9, 10, 11])),
                Ok(Bytes::from_static(&[])),
            ]);

            let mut read = super::util::StreamReader::new(stream);

            let mut buf = [0; 5];
            read.read_exact(&mut buf).await?;
            assert_eq!(buf, [0, 1, 2, 3, 4]);

            assert_eq!(read.read(&mut buf).await?, 3);
            assert_eq!(&buf[..3], [5, 6, 7]);

            assert_eq!(read.read(&mut buf).await?, 4);
            assert_eq!(&buf[..4], [8, 9, 10, 11]);

            assert_eq!(read.read(&mut buf).await?, 0);

            Ok::<(), std::io::Error>(())
        });
    }
}
