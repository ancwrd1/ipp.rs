use std::{error, fmt, io, net::SocketAddr, pin::Pin, sync::Arc, task::Context};

use futures::{Future, Poll, TryFutureExt, TryStreamExt};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use log::{debug, error};

use ipp_proto::{
    attribute::STATUS_MESSAGE, ipp::DelimiterTag, AsyncIppParser, IppAttribute, IppRequestResponse, IppValue,
};

use crate::handler::IppRequestHandler;

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

impl error::Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::HyperError(e) => write!(f, "{}", e),
            ServerError::IOError(e) => write!(f, "{}", e),
        }
    }
}

async fn ipp_service(
    handler: Arc<dyn IppRequestHandler + Send + Sync>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let handler = handler.clone();

    let stream = req
        .into_body()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));

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
                        IppAttribute::new(STATUS_MESSAGE, IppValue::TextWithoutLanguage(status.to_string())),
                    );
                    response
                }
            };
            Ok(Response::new(Body::wrap_stream(response.into_stream())))
        }
        Err(e) => {
            error!("{}", e);
            let mut resp = Response::new(e.to_string().into());
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            Ok(resp)
        }
    }
}

/// IPP server
pub struct IppServer {
    inner: Box<dyn Future<Output = Result<(), ServerError>> + Send + Unpin>,
}

impl IppServer {
    fn new(address: SocketAddr, handler: Arc<dyn IppRequestHandler + Send + Sync>) -> Result<IppServer, ServerError> {
        let builder = Server::try_bind(&address).map_err(ServerError::from)?;

        let inner = builder
            .serve(make_service_fn(move |_| {
                let handler = handler.clone();
                async { Ok::<_, hyper::Error>(service_fn(move |req| ipp_service(handler.clone(), req))) }
            }))
            .map_err(|e| ServerError::HyperError(e));

        Ok(IppServer { inner: Box::new(inner) })
    }
}

impl Future for IppServer {
    type Output = Result<(), ServerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.inner).as_mut().poll(cx)
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
    pub fn build(self) -> Result<IppServer, ServerError> {
        IppServer::new(self.address, self.handler)
    }
}
