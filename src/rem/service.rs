use futures::{future, Future, BoxFuture};
use tokio_service::Service;

use std::io;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::cache::CacheOperation;
use rem::op;
use rem::error::RemError;

#[derive(Clone)]
pub struct CacheService{
    pub cache: Arc<Mutex<Cache>>
}

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
           error!("Request {}", req);
        // In this case, the response is immediate.
        let cache_op = CacheOperation::new_from_string(&req);
        let prim_cmd: char = cache_op.commands[0];
        let mut ret = String::new();
        match prim_cmd {
            'W' => op::write_stream_str_to_cache(cache_op.value, &self.cache),
            'R' => {
                let result = op::read_value_from_cache(cache_op.value, &self.cache);
                ret = String::from_utf8(result.unwrap()).unwrap(); // Todo handle errors 
                Ok(())
            },
            'D' => op::delete_value_from_cache(cache_op.value, &self.cache),
            _ => Err(RemError::with_reason(format!("Invalid cache command {:?}", prim_cmd))),
        };
        future::ok(ret).boxed()
    }
}