use std::{io, net::SocketAddr, sync::Arc};

use futures::{Future, Poll, Stream, TryStreamExt};
use hyper::{service::service_fn, Body, Chunk, Request, Response, Server, StatusCode};
use log::debug;

use ipp_proto::{
    attribute::STATUS_MESSAGE, ipp::DelimiterTag, AsyncIppParser, IppAttribute, IppRequestResponse, IppValue,
};

use crate::handler::IppRequestHandler;
use futures::task::Context;
use std::pin::Pin;

struct DummyHandler;
impl IppRequestHandler for DummyHandler {}

/// Server-related errors
#[derive(Debug)]
pub enum ServerError {
    HyperError(hyper::Error),
    IOError(io::Error),
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError::HyperError(err)
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        ServerError::IOError(err)
    }
}

/// IPP server
pub struct IppServer {
    inner: Box<dyn Future<Output = Result<(), ServerError>> + Send>,
}

impl IppServer {
    fn new(address: SocketAddr, handler: Arc<dyn IppRequestHandler + Send + Sync>) -> Result<IppServer, ServerError> {
        let inner = Server::try_bind(&address)?
            .serve(async move {
                let handler = handler.clone();
                service_fn(move |req: Request<Body>| {
                    async {
                        let stream: Box<dyn Stream<Item=io::Result<Chunk>> + Send + Unpin> = Box::new(
                            req.into_body()
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
                        );

                        let handler = handler.clone();

                        match AsyncIppParser::from(stream).await {
                            Ok(result) => {
                                debug!("Received request, payload present: {}", result.payload.is_some());

                                let request = IppRequestResponse::from_parse_result(result);
                                let req_id = request.header().request_id;

                                let response = match handler.handle_request(request) {
                                    Ok(response) => response,
                                    Err(status) => {
                                        let mut response = IppRequestResponse::new_response(handler.version(), status, req_id);
                                        response.attributes_mut().add(
                                            DelimiterTag::OperationAttributes,
                                            IppAttribute::new(
                                                STATUS_MESSAGE,
                                                IppValue::TextWithoutLanguage(status.to_string()),
                                            ),
                                        );
                                        response
                                    }
                                };
                                Response::new(Body::wrap_stream(response.into_stream()))
                            }
                            Err(e) => {
                                let mut resp = Response::new(e.to_string().into());
                                *resp.status_mut() = StatusCode::BAD_REQUEST;
                                resp
                            }
                        }
                    }
                })
            })
            .map_err(ServerError::from);

        Ok(IppServer { inner: Box::new(inner) })
    }
}

impl Future for IppServer {
    type Output = Result<(), ServerError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll(cx)
    }
}

/// Builder to create IPP servers
pub struct IppServerBuilder {
    address: SocketAddr,
    handler: Arc<dyn IppRequestHandler + Send + Sync>,
}

impl IppServerBuilder {
    /// Create builder for a given listening address
    pub fn new<S>(address: S) -> IppServerBuilder
    where
        SocketAddr: From<S>,
    {
        IppServerBuilder {
            address: address.into(),
            handler: Arc::new(DummyHandler),
        }
    }

    /// Set request handler
    pub fn handler(mut self, handler: Arc<dyn IppRequestHandler + Send + Sync>) -> Self {
        self.handler = handler;
        self
    }

    /// Build server
    pub async fn build(self) -> Result<IppServer, ServerError> {
        IppServer::new(self.address, self.handler)
    }
}
