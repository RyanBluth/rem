use futures::{future, Future, BoxFuture};
use tokio_service::Service;
use std::io;

use rem::cache;

pub struct CacheService;

impl Service for CacheService {
    // These types must match the corresponding protocol types:
    type Request =  String;
    type Response = String;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
       // cache::CacheOperation::new_from_string(req);
        future::ok(req).boxed()
    }
}