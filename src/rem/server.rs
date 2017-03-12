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
    /*

    info!("Starting on {}:{}", ip, port);

    let listener: TcpListener = TcpListener::bind(format!("{}:{}", ip, port).as_str()).unwrap();

    let cache = Arc::new(Mutex::new(Cache::new()));

    loop {
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            let cache: Arc<Mutex<Cache>> = cache.clone();
            match stream {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        info!("Incoming connection: {:?}", stream.peer_addr());
                        io::stdout().flush().unwrap();
                        loop {
                            let client_res = handle_client(&mut stream, &cache);
                            if client_res.is_err() {
                                error!("An error occured while handling a client connection: {}",
                                       client_res.unwrap_err());
                            }
                        }
                    });
                }
                Err(e) => error!("Incoming connection failed. Error {}", e),
            }
        }
    }
    */
}


fn handle_client(mut stream: &mut TcpStream, cache: &Arc<Mutex<Cache>>) -> Result<(), RemError> {
   /* let buf_str: String = try!(op::string_from_stream(stream));
    if buf_str.len() > 0 {
        let cache_op: CacheOperation = CacheOperation::new_from_string(&buf_str);

        debug!("cmds = {:?}, value = {}", cache_op.commands, cache_op.value);

        if cache_op.commands.len() > 0 {
            let prim_cmd: char = cache_op.commands[0];
            return match prim_cmd {
                //'W' => op::write_stream_str_to_cache(cache_op.value, cache),
                'R' => op::read_value_from_cache(cache_op.value, cache, &mut stream),
                'D' => op::delete_value_from_cache(cache_op.value, cache),
                _ => Err(RemError::with_reason(format!("Invalid cache command {:?}", prim_cmd))),
            };
        }
    } else {
        panic!("Empty cache operation");
    }*/
    return Ok(());
}


