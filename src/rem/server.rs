use std::string::String;
use std::sync::{Arc, Mutex};

use rem::cache::Cache;
use rem::service::CacheService;
use rem::proto::CacheProto;
use rem::config::Config;

use futures_cpupool::CpuPool;

use native_tls::{Pkcs12, TlsAcceptor};

use tokio_tls::proto::Server;
use tokio_proto::TcpServer;

use std::fs::File;
use std::io::{Read};

pub fn launch(config: Config, ip: String, port: String) {
   // Specify the localhost address
    let addr = format!("{}:{}", ip, port).parse().unwrap();

    let pkcs12 = get_pkcs12(&config.server.cert_file, &config.server.cert_password);
    let acceptor = TlsAcceptor::builder(pkcs12).unwrap().build().unwrap();

    let proto = Server::new(CacheProto{}, acceptor);

    // The builder requires a protocol and an address
    let server = TcpServer::new(proto, addr);

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

fn get_pkcs12(cert:&String, password:&String) -> Pkcs12{
    let mut file = File::open(cert).unwrap();
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12, password).unwrap();
    return pkcs12;
}


