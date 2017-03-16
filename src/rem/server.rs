use std::io::prelude::*;
use std::io;
use std::string::String;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::cache::CacheOperation;
use rem::error::*;
use rem::op;

use rem::codec::CacheCodec;
use rem::service::CacheService;
use rem::proto::CacheProto;

use tokio_proto::TcpServer;

pub fn launch(ip: String, port: String) {
   // Specify the localhost address
    let addr = format!("{}:{}", ip, port).parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(CacheProto{}, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    let cache = Arc::new(Mutex::new(Cache::new()));
    let cache_service = CacheService {
        cache: cache.clone()
    };
    server.serve( move || Ok(cache_service.clone()));
}




