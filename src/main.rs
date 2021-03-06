#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate backtrace;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_io;
extern crate bytes;
extern crate native_tls;
extern crate tokio_tls;
extern crate toml;

mod rem;

use std::string::String;
use std::env;
use std::env::Args;

use rem::error::*;
use rem::config::Config;

/// The different run modes for REM
enum Mode {
    NONE,
    CLIENT,
    SERVER,
}

fn main() {
    env_logger::init().unwrap();

    let mut args: Args = env::args();

    // Set default values for arguments
    let mut ip: String = String::from("127.0.0.1");
    let mut port: String = String::from("8080");
    let mut config_file: String = String::from("rem.toml");

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
                    "-config" =>{
                        match args.next() {
                            Some(x) => config_file = x,
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

    // Todo handle errors
    let config: Config = Config::from_file(config_file).unwrap();

    match mode {
        Mode::CLIENT => rem::client::launch(config, ip, port),
        Mode::SERVER => rem::server::launch(config, ip, port),
        Mode::NONE => {
            RemError::with_reason_str(REM_00001).log();
        }
    }

    
}
