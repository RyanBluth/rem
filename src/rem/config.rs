use std::io::prelude::*;
use std::fs::File;

use toml;

use rem::error::RemError;

#[derive(Deserialize, Debug)]
pub struct Config{
    pub ssl: bool,
    pub domain: String,
    pub client:ClientConfig,
    pub server:ServerConfig
}

#[derive(Deserialize, Debug)]
pub struct ClientConfig{
   
}


#[derive(Deserialize, Debug)]
pub struct ServerConfig{
    pub cert_file:String,
    pub cert_password:String
}


impl Config {
    pub fn from_file(file:String) -> Result<Config, RemError>{
         let mut f:File = try!(File::open(file));
         let mut buf = String::new();
         try!(f.read_to_string(&mut buf));
         /// TDDO handle toml errors
         let conf:Config = toml::from_str(buf.as_str()).unwrap();
         return Ok(conf);
    }
}