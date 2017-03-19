use std::string::String;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::service::CacheService;
use rem::proto::CacheProto;
use futures::Future;
use futures_cpupool::CpuPool;

use tokio_proto::TcpServer;

pub fn launch(ip: String, port: String) {
   // Specify the localhost address
    let addr = format!("{}:{}", ip, port).parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(CacheProto{}, addr);

    let pool = Box::new(CpuPool::new_num_cpus());

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    let cache = Arc::new(Mutex::new(Cache::new()));
    let cache_service = CacheService {
        cache: cache.clone(),
        pool : pool
    };
    server.serve( move || Ok(cache_service.clone()));
}




