use std::io::prelude::*;
use std::io;
use std::string::String;
use std::vec::Vec;
use std::mem;

use rem::tcp_stream::TcpStream;
use rem::config::Config;

use rem::op;
use rem::error::*;

pub fn launch(ip: String, port: String) {
    info!("Connection to {}:{}", ip, port);
    let config = Config{
        ssl: false
    };
    match TcpStream::connect(&config, format!("rem:{}", port).as_str()) {
        Ok(mut stream) => {
            loop {
                // Continue looping, executing any commands from the user
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


/// Executes a write operation by parsing the client command and converting it to REM format
/// ex: write abc:def would be converted to 9|W$abc:def and sent to the REM server
fn client_exec_write(key:&String, val:&String, mut stream: &mut TcpStream)-> Result<(), RemError> {
    let sized_val = String::from(format!("W${}:{}", key, val));
    let res = op::write_str_to_stream_with_size(&mut stream, sized_val);
    try!(print_response(&mut stream));
    return res;
}

/// Executes a read operation by parsing the client command and converting it to REM format
/// ex: read abc:def would be converted to 5|R$abc and sent to the REM launch_server
/// The response from the REM server is written to stdout
/// If stdout::flush fail a warning will be logged
fn client_exec_read(key: &String, mut stream: &mut TcpStream)-> Result<(), RemError>{
    let cmd_val = String::from(format!("R${}", key));
    try!(op::write_str_to_stream_with_size(&mut stream, cmd_val));
    try!(print_response(&mut stream));
    return Ok(());
}

/// Executes a delete operation by parsing the client command and converting it to REM format
/// ex: delete abc would be converted to 5|D$abc and sent to the REM server
fn client_exec_delete(key: &String, mut stream: &mut TcpStream) -> Result<(), RemError>{
    let cmd_val = String::from(format!("D${}", key));
    let res = op::write_str_to_stream_with_size(&mut stream, cmd_val);
    try!(print_response(&mut stream));
    return res;
}

fn print_response(mut stream: &mut TcpStream) -> Result<(), RemError>{
    let val: String = try!(op::string_from_stream(&mut stream));
    println!("{}", val);
    try!(io::stdout().flush());
    return Ok(());
}

struct InputParser{
    args:Vec<String>,
    current:String,
    consumed_double_quote:bool,
    consumed_single_quote:bool
}

impl InputParser{

    /// Consumes a space charater taking quotes into consideration
    /// If the parser has consumed an opening quote then the space will be consumed as a character
    pub fn consume_space(&mut self){
        // If neither a single quote or a double quote has been consumed then its a new argument
        if !self.consumed_double_quote && !self.consumed_single_quote {
            self.push_current();
        }else{
            self.current.push(' ');
        }
    }

    /// Consumes a double quote, keeping track of whether it is an opening or cloing quote
    /// Takes single quotes into account when determening if the double quote is a delimiter or character
    pub fn consume_double_quote(&mut self){
        // If a single quote hasn't been consumed we're at the end or 
        // beginning of an argument in double quotes
        if  !self.consumed_single_quote {
            if self.consumed_double_quote{
                self.push_current();
            }
            // Flip the value so we know the sate for the next double quote that is consumed
            self.consumed_double_quote = !self.consumed_double_quote;
        }else{
            // If we're in double quotes just treat the double quote as a regular character 
            self.current.push('"');
        }
    }

    /// Consumes a single quote, keeping track of whether it is an opening or cloing quote
    /// Takes double quotes into account when determening if the single quote is a delimiter or character
    pub fn consume_single_quote(&mut self){
         // If a double quote hasn't been consumed we're at the end or 
        // beginning of an argument in single quotes
         if !self.consumed_double_quote {
            if self.consumed_single_quote{
                self.push_current();
            }
            // Flip the value so we know the sate for the next single quote that is consumed
            self.consumed_single_quote = !self.consumed_single_quote;
        }else{
            // If we're in double quotes just treat the single quote as a regular character 
            self.current.push('\'');
        }
    }

    /// Adds the character onto the current argument
    pub fn consume_char(&mut self, c:char){
        self.current.push(c);
    }

    /// To be called when everything has been parsed
    pub fn end(&mut self){
        self.push_current();
    }

    /// Pushes the current string into the list of args
    /// If the length of current is 0 no actions are performed
    pub fn push_current(&mut self){
        if self.current.len() > 0 {
            let arg = mem::replace(&mut self.current, String::new());
            self.args.push(arg);
        }
    }

}

/// Parses the arguments out of an input string taking quotes and spaces into consideration
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
