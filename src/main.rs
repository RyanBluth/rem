use std::io::prelude::*;
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

impl CacheItem{
    fn default() -> CacheItem {
        return CacheItem{
            value: ptr::null_mut(),
            size: 0,
        }
    }

    fn new(val: * mut u8, size: i32) -> CacheItem {
        return CacheItem{
            value: val,
            size: size,
        }
    }
}

struct Cache{
    map_internal : HashMap<String,CacheItem>
}

impl Cache{

    fn new() -> Cache{
        return Cache{
            map_internal: HashMap::new()
        }
    }

    fn cache_item( &mut self, key: String, val: * mut u8, size: i32 ){
        self.map_internal.insert(key, CacheItem::new(val, size));
    }

}


fn handle_client(stream: &mut TcpStream, cache:&mut Cache) {
    let mut buf: Vec<u8> = Vec::new();
    stream.read_to_end( &mut buf );
    let buf_str:String = String::from_utf8(buf).unwrap();
    write_stream_str_to_cache(buf_str, cache);
}


fn write_stream_str_to_cache(stream_str:String, cache:&mut Cache){
    let mut key:String = String::from("");
    let mut val:String = String::from("");
    let mut idx = 0;
    for c in stream_str.chars(){
        idx += 1;
        if c == ':'{
            val = String::from(&stream_str[idx..]);
            break;
        }
        key.push(c);
    }
    println!("Writing to cache: key={:?} val={:?}", key, val);
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

