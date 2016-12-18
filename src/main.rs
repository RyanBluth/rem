#[macro_use]
extern crate log;
extern crate env_logger;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;
use std::path::Path;
use std::ptr;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::result;
use std::fs;
use std::sync::{Arc, Mutex};
use std::env;


struct CacheOperation{
    commands: Vec<char>,
    value: String
}

impl CacheOperation{

    fn new_from_string(cache_op_str: &String) -> CacheOperation {
        let mut cache_op = CacheOperation{
            commands:Vec::new(),
            value:String::new()
        };
        let mut idx = 0;
        let chars_iter = cache_op_str.chars();
        for c in chars_iter {
            idx += 1;
            if c == '$'{
                cache_op.value = String::from(&cache_op_str[idx..]);
                break;
            }
            cache_op.commands.push(c);
        }
        return cache_op;
    }
}


struct Cache{
    map_internal : HashMap<String, Vec<u8>>
}

impl Cache{

    fn new() -> Cache{
        return Cache{
            map_internal: HashMap::new()
        }
    }

    fn cache_item( &mut self, key: String, val: Vec<u8>){
        debug!("Writing to cache: key={:?} val={:?}", key, val);
        fs::create_dir("_cache");
        let mut f:File = File::create(format!("_cache/{}", key)).unwrap();
        f.write(&val.as_slice());
        f.flush();
        self.map_internal.insert(key, val);
    }

    fn read_item( &self, key:String ) -> Option<Box<Vec<u8>>> {
        let mut res: Option<Box<Vec<u8>>> = None;
        if self.map_internal.contains_key(&key) {
            let res_opt = self.map_internal.get(&key);
            match res_opt {
                Some(vec_ref) => res = Some(Box::new(vec_ref.clone())),
                None => res = None
            }
        } else {
            let mut buf: Vec<u8> = Vec::new();
            let mut f_res = match File::open(&key) {
                Err(why) => res = None,
                Ok(mut file) => {
                    file.read_to_end(&mut buf);
                    res = Some(Box::new(buf));
                }
            };
        }
        return res;
    }
}


fn handle_client(mut stream: &mut TcpStream, mut cache:&Arc<Mutex<Cache>>) {
    stream.set_nodelay(true);
    let mut buf_arr:[u8; 64] = [0; 64];
    stream.read(&mut buf_arr);

    let mut size_str = String::new();
    let mut buf_size:usize = 0;
    for i in 0..64{
        buf_size += 1;
        if buf_arr[i] == '|' as u8{
            break;
        }
        size_str.push(buf_arr[i as usize] as char);
    }


    let upper_idx:usize = size_str.parse::<i32>().unwrap() as usize;
    let msg_buf = buf_arr;
    let mut buf_temp: Vec<u8> = msg_buf.to_vec();
    let mut buf: Vec<u8> = buf_temp.drain(buf_size..upper_idx + buf_size).collect();

    stream.flush().unwrap();

    let buf_str:String = String::from_utf8(buf).unwrap();
    if buf_str.len() > 0 {

        let cache_op:CacheOperation = CacheOperation::new_from_string(&buf_str);

        debug!("cmds = {:?}, value = {}", cache_op.commands, cache_op.value );

        if cache_op.commands.len() > 0 {
            let prim_cmd:char = cache_op.commands[0];
            match prim_cmd{
                'W' => write_stream_str_to_cache(cache_op.value, cache),
                'R' => read_value_from_cache( cache_op.value, cache, &mut stream),
                _ => error!("Invalid cache command {:?}", prim_cmd)
            }
        }
    }else{
        panic!("Empty cache operation");
    }
}


fn read_value_from_cache<'a>( key:String, cache_mtx:&Arc<Mutex<Cache>>, stream: &mut TcpStream){
    let cache = cache_mtx.lock().unwrap();
    let cache_opt:Option<Box<Vec<u8>>> = cache.read_item(key);
    match cache_opt{
        Some(boxed_val)=> {
            let val = *boxed_val;
            debug!("Writing to stream {:?}", val);
            stream.write(&val[0..val.len()]);
            stream.flush();
        },
        None =>{
            stream.write(&[65,65,65,65]);
            stream.flush();
        }
    }
}

fn write_stream_str_to_cache(stream_str:String, cache_mtx:&Arc<Mutex<Cache>>){
    let mut key:String = String::from("");
    let mut val:String = String::from("");
    let mut idx = 0;
    let mut chars_iter = stream_str.chars();
    for c in chars_iter {
        idx += 1;
        if c == ':'{
            val = String::from(&stream_str[idx..]);
            break;
        }
        key.push(c);
    }
    let bytes = val.into_bytes();
    let mut cache = cache_mtx.lock().unwrap();
    cache.cache_item( key, bytes);
}

fn main() {

    env_logger::init().unwrap();

    info!("Starting on port 8080");

    let listener:TcpListener = TcpListener::bind("127.0.0.1:8080").unwrap();
    //let mut cache:Cache = Cache::new();

    let cache = Arc::new(Mutex::new(Cache::new()));

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        let cache:Arc<Mutex<Cache>> = cache.clone();
        match stream {
            Ok(mut stream) => {
                thread::spawn( move ||{
                    info!("Incoming connection: {:?}", stream.peer_addr());
                    io::stdout().flush().unwrap();
                    loop{
                        handle_client(&mut stream, &cache);
                    }
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }
}

