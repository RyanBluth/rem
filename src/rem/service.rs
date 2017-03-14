use futures::{future, Future, BoxFuture};
use tokio_service::Service;

use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::cache::CacheOperation;
use rem::op;
use rem::error::RemError;

pub const OK:    &'static str = "OK";
pub const ERROR: &'static str = "ERROR";

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
        let cache_op = CacheOperation::new_from_string(&req);
        let prim_cmd: char = cache_op.commands[0];
        let cache_res:Result<String, RemError> = match prim_cmd {
            'W' => {
                // Todo handle errors 
                op::write_stream_str_to_cache(cache_op.value, &self.cache);
                Ok(String::from(OK))
            },
            'R' => {
                let result = op::read_value_from_cache(cache_op.value, &self.cache);
                // Todo handle errors 
                Ok(String::from_utf8(result.unwrap()).unwrap())
            },
            'D' => {
                // Todo handler errors 
                op::delete_value_from_cache(cache_op.value, &self.cache);
                Ok(String::from(OK))
            }
            _ => Err(RemError::with_reason(format!("Invalid cache command {:?}", prim_cmd))),
        };

        let ret = match cache_res {
            Ok(res) =>  res,
            Err(cause) =>{
                let err_desc = String::from(cause.description());
                format!("{}:{}", ERROR, err_desc)
            }
        };
        return future::ok(ret).boxed();
    }
}