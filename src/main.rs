use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;
use std::path::Path;
use std::ptr;
use std::net::{TcpListener, TcpStream};


struct CacheItem{
    value: *mut u8,
    size: i32
}


struct CacheOperation{
    commands: Vec<char>,
    key: String,
    value: Vec<u8>
}

impl CacheOperation{

    fn new_from_string(cache_op_str: &String) -> CacheOperation {
        let mut cache_op = CacheOperation{
            commands:Vec::new(),
            key:String::new(),
            value:Vec::new()
        };
        let mut idx = 0;
        let mut chars_iter = cache_op_str.chars();
        let mut parsed_cmds = false;
        for c in chars_iter {
            idx += 1;
            let is_tok = c == '$' || c == ':';
            if c == '$'{
                parsed_cmds = true;
            } else if c == ':'{
                cache_op.value = String::from(&cache_op_str[idx..]).into_bytes();
                break;
            }
            if parsed_cmds && !is_tok{
                cache_op.key.push(c);
            }else if !is_tok{
               cache_op.commands.push(c);
            }
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
        println!("Writing to cache: key={:?} val={:?}", key, val);
        self.map_internal.insert(key, val);
    }

}


fn handle_client(stream: &mut TcpStream, cache:&mut Cache) {
    let mut buf: Vec<u8> = Vec::new();
    stream.read_to_end( &mut buf );
    let buf_str:String = String::from_utf8(buf).unwrap();
    if buf_str.len() > 0 {
        let mut cache_op:CacheOperation = CacheOperation::new_from_string(&buf_str);
        print!("cmds = {:?}, key = {}, value {:?}", cache_op.commands, cache_op.key, cache_op.value );
        io::stdout().flush().unwrap();
    }else{
        panic!("Empty cache operation");
    }
}


fn read_value_from_cache( key:&String, cache:&Cache){

}

fn write_stream_str_to_cache(stream_str:String, cache:&mut Cache){
    let mut key:String = String::from("");
    let mut val:String = String::from("");
    let mut idx = 0;
    let mut chars_iter = stream_str.chars();
    chars_iter.next(); // Consume command
    for c in chars_iter {
        idx += 1;
        if c == ':'{
            val = String::from(&stream_str[idx..]);
            break;
        }
        key.push(c);
    }
    let bytes = val.into_bytes();
    cache.cache_item( key, bytes);
}

fn main() {

    let listener:TcpListener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut cache:Cache = Cache::new();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_client(&mut stream, &mut cache);
            }
                Err(e) => { /* connection failed */ }
            }

    }

    let file_res = make_cache();
    if file_res.is_err() {
        panic!("Could not create cache file");
    }
    let mut cache_file = file_res.unwrap();
    let write_res = cache_file.write_all(b"test");
    if write_res.is_err() {
        panic!("Could not write to cache");
    }
    read_index(cache_file);
}


fn make_cache() -> Result<File, std::io::Error>{
    return File::create("cache.txt");
}

fn read_index(mut cache_file: File) -> HashMap<String, i32> {
    let path = Path::new("cache.txt");
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open"),
        Ok(file) => file,
    };
    let mut buf:Vec<u8> = Vec::new();
    file.read_to_end(&mut buf);
    for char in buf {
        println!("{}", char);
    }

    return HashMap::new();
}

