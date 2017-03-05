#[macro_use]
extern crate log;
extern crate env_logger;
extern crate backtrace;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs;
use std::sync::{Arc, Mutex};
use std::env;
use std::env::Args;
use std::fmt;
use std::error::Error;
use std::mem;

use backtrace::Backtrace;


const REM_00001: &'static str = "REM_00001: A run mode must be specified. One of [server, client] \
                                 expected";
const REM_00002: &'static str = "REM_00002: Unexpected argument encountered";
const REM_00003: &'static str = "REM_00003: IO operation failed";
const REM_00004: &'static str = "REM_00004: Failed to parse integer value from string";

/// The different run modes for REM
enum Mode {
    NONE,
    CLIENT,
    SERVER,
}

/// Simple error structure to be used when errors occur during a cache operation
#[derive(Debug)]
struct RemError{
    reason: String
}

impl RemError {
    pub fn with_reason(reason: String) -> RemError {
        return RemError {
            reason: reason
        };
    }

    pub fn with_reason_str(reason: &'static str) -> RemError {
        return RemError {
            reason: String::from(reason)
        };
    }

    pub fn with_reason_str_and_details(reason: &'static str, details: String) -> RemError {
        return RemError {
            reason: String::from(format!("{}: {}\n{:?}", reason, details, Backtrace::new())) 
        };
    }

    pub fn log(self) {
        error!("{}", self);
    }

    pub fn log_and_exit(self) {
        self.log();
        std::process::exit(1);
    }
    
}


impl<'a> Error for RemError {
    fn description(&self) -> &str {
        return &self.reason;
    }
}


impl fmt::Display for RemError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.description());
    }
}

impl From<io::Error> for RemError {
    fn from(e: io::Error) -> RemError {
       return RemError::with_reason_str_and_details(REM_00003, String::from(e.description()));
    }
}

impl From<std::num::ParseIntError> for RemError {
     fn from(e: std::num::ParseIntError) -> RemError {
       return RemError::with_reason_str_and_details(REM_00004, String::from(e.description()));
    }
}

/// A structure to store a series of cache operations and a value
/// Cache operations are represented by a single character
struct CacheOperation {
    commands: Vec<char>,
    value: String,
}

impl CacheOperation {
    /// Creates a new CacheOperation instance from an input string
    /// Stores the commands and corresponding value
    /// Write Example: ```W$abc:def```
    /// The resulting command would be W with a value of abc:def
    fn new_from_string(cache_op_str: &String) -> CacheOperation {
        let mut cache_op = CacheOperation {
            commands: Vec::new(),
            value: String::new(),
        };
        let mut idx = 0;
        let chars_iter = cache_op_str.chars();
        // Read the string until $ is found
        for c in chars_iter {
            idx += 1;
            if c == '$' {
                // Set the value to everything after $
                cache_op.value = String::from(&cache_op_str[idx..]);
                break;
            }
            // Add a new command for each iteration where $ hasn't been found yet
            cache_op.commands.push(c);
        }
        return cache_op;
    }
}


/// Cache object -- Simple wrapper around a map
struct Cache {
    map_internal: HashMap<String, Vec<u8>>,
}

impl Cache {
    fn new() -> Cache {
        return Cache { map_internal: HashMap::new() };
    }

    /// Writes the provided value to the cache using the provided key
    ///
    /// The value will be written to the in memory store and the file store
    ///
    /// The _cache directory will be created if it does not exist
    ///
    /// If a value for the provided key already exists it will be overwritten
    fn cache_item(&mut self, key: String, val: Vec<u8>) -> Result<(), RemError> {
        debug!("Writing to cache: key={:?} val={:?}", key, val);
        let dir_res = fs::create_dir("_cache");
        if dir_res.is_err() {
            let err_kind = dir_res.unwrap_err().kind();
            if err_kind == io::ErrorKind::PermissionDenied {
                return Result::Err(RemError::from(io::Error::from(err_kind)));
            }
        }
        let mut f: File = File::create(format!("_cache/{}", key)).unwrap();
        try!(f.write(&val.as_slice()));
        try!(f.flush());
        self.map_internal.insert(key, val);
        return Ok(());
    }

    /// Reads a value from the cache
    ///
    /// If the key is found in the in memory map then the corresponding value is returned
    ///
    /// If the key cannot be found in the map then an attempt will be made to load the value
    /// from the file corresponding with the key
    fn read_item(&self, key: String) -> Result<Option<Box<Vec<u8>>>, RemError> {
        let res: Option<Box<Vec<u8>>>;
        if self.map_internal.contains_key(&key) {
            let res_opt = self.map_internal.get(&key);
            match res_opt {
                Some(vec_ref) => res = Some(Box::new(vec_ref.clone())),
                None => res = None,
            }
        } else {
            let mut buf: Vec<u8> = Vec::new();
            match File::open(format!("_cache/{}", key)) {
                Err(_) => res = None,
                Ok(mut file) => {
                    try!(file.read_to_end(&mut buf));
                    res = Some(Box::new(buf));
                }
            };
        }
        return Ok(res);
    }

    /// Delete's an item from the cache
    ///
    /// If the key is found in the in memory map then that entry will be removed
    ///
    /// The file corresponding to the key will also be deleted
    fn delete_item(&mut self, key: String) -> Result<(), RemError> {
        if self.map_internal.contains_key(&key) {
            self.map_internal.remove(&key);
        }
        let path = format!("_cache/{}", key);
        if Path::new(&path).exists() {
            match fs::remove_file(path) {
                Ok(x) => x,
                Err(_) => {
                    return Err(RemError::with_reason(format!("Failed to delete file for key: \
                                                                {}",
                                                             key)))
                }
            }
        }
        return Ok(());
    }
}


/// Parses a TCP input stream and extracts the data
/// Allocates a 64 byte buffer which is used to read the input info from the stream
/// The expected format is ```{size}|{content}```
/// Ex. ```5|W$a:b```
fn string_from_stream(stream: &mut TcpStream) -> Result<String, RemError> {
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

fn handle_client(mut stream: &mut TcpStream, cache: &Arc<Mutex<Cache>>) -> Result<(), RemError> {
    let buf_str: String = try!(string_from_stream(stream));
    if buf_str.len() > 0 {
        let cache_op: CacheOperation = CacheOperation::new_from_string(&buf_str);

        debug!("cmds = {:?}, value = {}", cache_op.commands, cache_op.value);

        if cache_op.commands.len() > 0 {
            let prim_cmd: char = cache_op.commands[0];
            return match prim_cmd {
                'W' => write_stream_str_to_cache(cache_op.value, cache),
                'R' => read_value_from_cache(cache_op.value, cache, &mut stream),
                'D' => delete_value_from_cache(cache_op.value, cache),
                _ => Err(RemError::with_reason(format!("Invalid cache command {:?}", prim_cmd))),
            };
        }
    } else {
        panic!("Empty cache operation");
    }
    return Ok(());
}


fn read_value_from_cache(key: String,
                         cache_mtx: &Arc<Mutex<Cache>>,
                         mut stream: &mut TcpStream)
                         -> Result<(), RemError> {
    let cache = cache_mtx.lock().unwrap();
    let cache_opt: Option<Box<Vec<u8>>> = try!(cache.read_item(key));
    match cache_opt {
        Some(boxed_val) => {
            let val: Vec<u8> = *boxed_val;
            debug!("Writing to stream {:?}", val);
            try!(write_str_to_stream_with_size(&mut stream, String::from_utf8(val).unwrap()));
            return Ok(());
        }
        None => {
            try!(write_str_to_stream_with_size(&mut stream, String::from("Invalid Key")));
            return Err(RemError::with_reason(String::from("Could not read from cache")));
        }
    }
}

fn write_stream_str_to_cache(stream_str: String,
                             cache_mtx: &Arc<Mutex<Cache>>)
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
    return cache.cache_item(key, bytes);
}

fn delete_value_from_cache(key: String, cache_mtx: &Arc<Mutex<Cache>>) -> Result<(), RemError> {
    let mut cache = cache_mtx.lock().unwrap();
    return cache.delete_item(key);
}

fn write_str_to_stream_with_size(stream: &mut TcpStream, value: String) -> Result<(), RemError>  {
    let sized_val = String::from(format!("{}|{}", value.len(), value));
    try!(stream.write(String::from(sized_val).as_bytes()));
    try!(stream.flush());
    return Ok(());
}


struct InputParser{
    args:Vec<String>,
    current:String,
    consumed_double_quote:bool,
    consumed_single_quote:bool
}

impl InputParser{

    pub fn consume_space(&mut self){
        if !self.consumed_double_quote && !self.consumed_single_quote {
            self.push_current();
        }else{
            self.current.push(' ');
        }
    }

    pub fn consume_double_quote(&mut self){
        if  !self.consumed_single_quote {
            if self.consumed_double_quote{
                self.push_current();
            }
            self.consumed_double_quote = !self.consumed_double_quote;
        }else{
            self.current.push('"');
        }
    }


    pub fn consume_single_quote(&mut self){
         if  !self.consumed_double_quote {
            if self.consumed_single_quote{
                self.push_current();
            }
            self.consumed_single_quote = !self.consumed_single_quote;
        }else{
            self.current.push('\'');
        }
    }


    pub fn consume_char(&mut self, c:char){
        self.current.push(c);
    }

    pub fn end(&mut self){
        self.push_current();
    }

    pub fn push_current(&mut self){
        if self.current.len() > 0 {
            let arg = mem::replace(&mut self.current, String::new());
            self.args.push(arg);
        }
    }

}

pub fn parse_input(input: String) -> Vec<String>{
    let mut parser = InputParser{
        args:Vec::new(),
        current:String::new(),
        consumed_double_quote:false,
        consumed_single_quote:false
    };    
    for c in input.chars(){
        match c {
            '"'  => parser.consume_double_quote(),
            ' '  => parser.consume_space(),
            '\'' => parser.consume_single_quote(), 
            _    => parser.consume_char(c)
        }
    }
    parser.end();

    return parser.args;
}


fn launch_client(ip: String, port: String) {
    info!("Connection to {}:{}", ip, port);

    match TcpStream::connect(format!("{}:{}", ip, port).as_str()) {
        Ok(mut stream) => {
            loop {
                // Contine looping, executing any commands from the user
                let handle = io::stdin();
                for line_res in handle.lock().lines() {
                    let line: String = line_res.unwrap();
                    let args:Vec<String> = parse_input(line);
                    if args.len() > 0{
                        let arg_ref = args[0].as_ref();
                        match arg_ref {
                            "write" => {
                                if args.len() == 3 {
                                    match client_exec_write(&args[1], &args[2], &mut stream){
                                        Ok(_) => (),
                                        Err(why) => why.log()
                                    }
                                }else{
                                    error!("Write expects two arguments - key and value");
                                }
                            },
                            "read" => {
                                if args.len() == 2 {
                                    match client_exec_read(&args[1], &mut stream){
                                        Ok(_) => (),
                                        Err(why) => why.log()
                                    }
                                }else{
                                    error!("Read expects one argument - key");
                                }
                            },
                            "delete" => {
                                if args.len() == 2 {
                                    match client_exec_delete(&args[1], &mut stream){
                                        Ok(_) => (),
                                        Err(why) => why.log()
                                    }
                                }else{
                                    error!("Delete expects one argument - key");
                                }
                            }
                            _ => error!("Not a valid command")
                        }
                    }
                }
            }
        }
        Err(e) => {
            panic!("Failed to connect to server. Error '{}'", e);
        }
    }
}


/// Executres a write operation by parsing the client command and converting it to REM format
/// ex: write abc:def would be converted to 9|W$abc:def and sent to the REM server
fn client_exec_write(key:&String, val:&String, mut stream: &mut TcpStream)-> Result<(), RemError> {
    let sized_val = String::from(format!("W${}:{}", key, val));
    return write_str_to_stream_with_size(&mut stream, sized_val);
}

/// Executres a read operation by parsing the client command and converting it to REM format
/// ex: read abc:def would be converted to 5|R$abc and sent to the REM launch_server
/// The respone from the REM server is writen to stdout
/// If stdout::flush fail a warning will be logged
fn client_exec_read(key: &String, mut stream: &mut TcpStream)-> Result<(), RemError>{
    let cmd_val = String::from(format!("R${}", key));
    try!(write_str_to_stream_with_size(&mut stream, cmd_val));
    let val: String = try!(string_from_stream(&mut stream));
    println!("{}", val);
    try!(io::stdout().flush());
    return Ok(());
}

/// Executes a delete operation by parsing the client command and converting it to REM format
/// ex: delete abc would be converted to 5|D$abc and sent to the REM server
fn client_exec_delete(key: &String, mut stream: &mut TcpStream) -> Result<(), RemError>{
    let cmd_val = String::from(format!("D${}", key));
    return write_str_to_stream_with_size(&mut stream, cmd_val);
}


fn launch_server(ip: String, port: String) {

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
                                break;
                            }
                        }
                    });
                }
                Err(e) => error!("Incoming connection failed. Error {}", e),
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut args: Args = env::args();

    // Set default values for arguments
    let mut ip: String = String::from("127.0.0.1");
    let mut port: String = String::from("8080");

    let mut mode: Mode = Mode::NONE;

    // Consume the first argument since it is just the program
    args.next();

    loop {
        match args.next().as_ref() {
            Some(opt) => {
                match opt.as_ref() {
                    "server" => {
                        mode = Mode::SERVER;
                    }
                    "client" => {
                        mode = Mode::CLIENT;
                    }
                    "-port" => {
                        match args.next() {
                            Some(x) => port = x,
                            None => break,
                        }
                    }
                    "-ip" => {
                        match args.next() {
                            Some(x) => ip = x,
                            None => break,
                        }
                    }
                    _ => {
                        RemError::with_reason_str_and_details(REM_00002,
                                                              format!("Argument {} is not a \
                                                                       valid option",
                                                                      opt))
                            .log_and_exit();
                    }
                }
            }
            None => break,
        }
    }


    match mode {
        Mode::CLIENT => launch_client(ip, port),
        Mode::SERVER => launch_server(ip, port),
        Mode::NONE => {
            RemError::with_reason_str(REM_00001).log();
        }
    }
}
