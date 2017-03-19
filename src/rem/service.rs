use futures::{future, Future, BoxFuture};
use tokio_service::Service;

use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::cache::CacheOperation;
use rem::op;
use rem::error::RemError;

use std::{thread, time};

use futures_cpupool::CpuPool;

pub const OK:    &'static str = "OK";
pub const ERROR: &'static str = "ERROR";

#[derive(Clone)]
pub struct CacheService{
    pub cache: Arc<Mutex<Cache>>,
    pub pool : Box<CpuPool>
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
        // Clone the cache arc so we can move a ref into the closure
        let cache_ref = self.cache.clone();
        // Spawn the actual work on the thread pool
        self.pool.as_ref().spawn_fn( move || {
            let cache_op = CacheOperation::new_from_string(&req);
            let prim_cmd: char = cache_op.commands[0];
            let cache_res:Result<String, RemError> = match prim_cmd {
                'W' => {
                    match op::write_stream_str_to_cache(cache_op.value, cache_ref.as_ref()) {
                        Ok(()) => Ok(String::from(OK)),
                        Err(cause) => Err(cause)
                    }      
                },
                'R' => {
                    match op::read_value_from_cache(cache_op.value, cache_ref.as_ref()){
                        // Todo handle errors
                        Ok(res) => Ok(String::from_utf8(res).unwrap()),
                        Err(cause) => Err(cause)
                    }
                },
                'D' => {
                    match op::delete_value_from_cache(cache_op.value, cache_ref.as_ref()) {
                        Ok(()) => Ok(String::from(OK)),
                        Err(cause) => Err(cause)
                    } 
                }
                _ => Err(RemError::with_reason(format!("Invalid cache command {:?}", prim_cmd))),
            };
            let ret = match cache_res {
                Ok(res) =>  res,
                Err(cause) => {
                    let err_desc = String::from(cause.description());
                    format!("{}:{}", ERROR, err_desc)
                }
            };
            return Ok(ret);
        }).boxed()
    }
}