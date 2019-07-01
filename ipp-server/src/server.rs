use std::{
    io, mem,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures::{Future, Poll, Stream};
use hyper::{service::service_fn, Body, Chunk, Request, Response, Server};
use log::debug;

use ipp_proto::{AsyncIppParser, IppRequestResponse};

use crate::handler::IppRequestHandler;

struct DummyHandler;
impl IppRequestHandler for DummyHandler {}

#[derive(Debug)]
pub enum ServerError {
    HyperError(hyper::Error),
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError::HyperError(err)
    }
}

struct IppServer {
    inner: Box<dyn Future<Item = (), Error = ServerError> + Send>,
}

impl IppServer {
    fn new(address: SocketAddr, handler: Box<dyn IppRequestHandler + Send>) -> IppServer {
        let handler = Arc::new(Mutex::new(handler));

        let inner = Server::bind(&address)
            .serve(move || {
                let handler = handler.clone();
                service_fn(move |mut req: Request<Body>| {
                    let body = mem::replace(req.body_mut(), Body::empty());

                    let stream: Box<dyn Stream<Item = Chunk, Error = io::Error> + Send> =
                        Box::new(body.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())));

                    let handler = handler.clone();

                    AsyncIppParser::from(stream).map(move |result| {
                        debug!("Received request, payload present: {}", result.payload.is_some());

                        let request = IppRequestResponse::from_parse_result(result);
                        let req_id = request.header().request_id;

                        let mut handler = handler.lock().unwrap();

                        let response = match handler.handle_request(request) {
                            Ok(response) => response,
                            Err(status) => IppRequestResponse::new_response(handler.version(), status, req_id),
                        };
                        Response::new(Body::wrap_stream(response.into_stream()))
                    })
                })
            })
            .map_err(ServerError::from);

        IppServer { inner: Box::new(inner) }
    }
}

impl Future for IppServer {
    type Item = ();
    type Error = ServerError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

pub struct IppServerBuilder {
    address: SocketAddr,
    handler: Box<dyn IppRequestHandler + Send>,
}

impl IppServerBuilder {
    pub fn new<S>(address: S) -> IppServerBuilder
    where
        SocketAddr: From<S>,
    {
        IppServerBuilder {
            address: address.into(),
            handler: Box::new(DummyHandler),
        }
    }

    pub fn handler(mut self, handler: Box<dyn IppRequestHandler + Send>) -> Self {
        self.handler = handler;
        self
    }

    pub fn build(self) -> impl Future<Item = (), Error = ServerError> {
        IppServer::new(self.address, self.handler)
    }
}
