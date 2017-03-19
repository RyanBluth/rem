use std::io::prelude::*;
use std::string::String;
use std::vec::Vec;
use std::net::{TcpStream};
use std::sync::{Mutex};

use rem::cache::Cache;
use rem::error::*;

pub fn read_value_from_cache(key: String,
                         cache_mtx: &Mutex<Cache>)
                         -> Result<(Vec<u8>), RemError> {
    let cache = cache_mtx.lock().unwrap();
    let cache_opt: Option<Box<Vec<u8>>> = try!(cache.read_item(key));
    match cache_opt {
        Some(boxed_val) => {
            let val: Vec<u8> = *boxed_val;
            return Ok(val.clone());
        }
        None => {
            return Err(RemError::with_reason(String::from(REM_00005)));
        }
    }
}

/// Parses a TCP input stream and extracts the data
/// Allocates a 64 byte buffer which is used to read the input info from the stream
/// The expected format is ```{size}|{content}```
/// Ex. ```5|W$a:b```
pub fn string_from_stream(stream: &mut TcpStream) -> Result<String, RemError> {
    //Read in the first 54 bytes of the stram
    try!(stream.set_nodelay(true));
    let mut buf_arr: [u8; 64] = [0; 64];
    try!(stream.read(&mut buf_arr));
    // Parse the message size
    let mut size_str = String::new();
    let mut buf_size: usize = 0;
    for i in 0..64 {
        buf_size += 1;
        if buf_arr[i] == '|' as u8 {
            break;
        }
        size_str.push(buf_arr[i as usize] as char);
    }

    // Convert the size string to a usize so it can be used to drain the buffer
    let upper_idx: usize = try!(size_str.parse::<i32>()) as usize;
    let mut buf_temp: Vec<u8> = buf_arr.to_vec();
    // Create a new buffer using the parsed indicies
    let buf: Vec<u8> = buf_temp.drain(buf_size..upper_idx + buf_size).collect();

    stream.flush().unwrap();

    // Return the value as a string
    let buf_str: String = String::from_utf8(buf).unwrap();
    return Ok(buf_str);
}

pub fn write_stream_str_to_cache(stream_str: String,
                             cache_mtx: &Mutex<Cache>)
                             -> Result<(), RemError> {
    let mut key: String = String::new();
    let mut val: String = String::new();
    let mut idx = 0;
    let chars_iter = stream_str.chars();
    for c in chars_iter {
        idx += 1;
        if c == ':' {
            val = String::from(&stream_str[idx..]);
            break;
        }
        key.push(c);
    }
    let bytes = val.into_bytes();
    let mut cache = cache_mtx.lock().unwrap();
    return cache.cache_item(key.as_str(), bytes);
}

pub fn delete_value_from_cache(key: String, cache_mtx: &Mutex<Cache>) -> Result<(), RemError> {
    let mut cache = cache_mtx.lock().unwrap();
    return cache.delete_item(key);
}

pub fn write_str_to_stream_with_size(stream: &mut TcpStream, value: String) -> Result<(), RemError> {
    let sized_val = String::from(format!("{}|{}", value.len(), value));
    try!(stream.write(String::from(sized_val).as_bytes()));
    try!(stream.flush());
    return Ok(());
}