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
        println!("Writing to cache: key={:?} val={:?}", key, val);
        self.map_internal.insert(key, val);
    }

    fn read_item( &self, key:String ) -> Option<&Vec<u8>>{
        //println!("Reading value from cache: key:{}, value:{:?}", key, self.map_internal.get(key));
        return self.map_internal.get(&key);
    }

}


fn handle_client(mut stream: &mut TcpStream, mut cache:&mut Cache) {

    stream.set_nodelay(true);
    println!("Before Read");

    io::stdout().flush().unwrap();

    let mut buf_arr:[u8; 64] = [0; 64];
    stream.read(&mut buf_arr);

    let mut size_str = String::new();
    let mut size_idx:usize = 0;
    for i in 0..64{
        size_idx += 1;
        if buf_arr[i] == '|' as u8{
            break;
        }
        size_str.push(buf_arr[i as usize] as char);
    }

    println!("Size {}", size_str);
    io::stdout().flush().unwrap();

    let upper_idx:usize = size_str.parse::<i32>().unwrap() as usize;
    let msg_buf = buf_arr;
    let mut buf_temp: Vec<u8> = msg_buf.to_vec();
    let mut buf: Vec<u8> = buf_temp.drain(size_idx..upper_idx).collect();

    println!("buf {:?}", buf);
    io::stdout().flush().unwrap();

    stream.flush().unwrap();

    let buf_str:String = String::from_utf8(buf).unwrap();
    if buf_str.len() > 0 {

        let cache_op:CacheOperation = CacheOperation::new_from_string(&buf_str);

        print!("cmds = {:?}, value = {}", cache_op.commands, cache_op.value );
        io::stdout().flush().unwrap();

        if cache_op.commands.len() > 0 {

            let prim_cmd:char = cache_op.commands[0];

            match prim_cmd{
                'W' => write_stream_str_to_cache(cache_op.value, &mut cache),
                'R' => read_value_from_cache( cache_op.value, &mut cache, &mut stream),
                _ => panic!("Invalid cache command")
            }
        }
    }else{
        panic!("Empty cache operation");
    }
}


fn read_value_from_cache( key:String, cache:&Cache, stream: &mut TcpStream){
    let cache_opt = cache.read_item(String::from("a"));

    match cache_opt{
        Some(val)=> {
            println!("Writing to stream {:?}", val);
            io::stdout().flush().unwrap();
            stream.write(&val[0..val.len()]);
            stream.flush();
        },
        None =>{
            stream.write(&[65,65,65,65]);
            stream.flush();
        }
    }
}

fn write_stream_str_to_cache(stream_str:String, cache:&mut Cache){
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
    cache.cache_item( key, bytes);
}

fn main() {

    let listener:TcpListener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut cache:Cache = Cache::new();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                //thread::spawn(||{
                println!("Connection");
                io::stdout().flush().unwrap();
                loop{
                    handle_client(&mut stream, &mut cache);
                }
                //});
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

