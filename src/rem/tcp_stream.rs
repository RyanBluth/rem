use std::io::prelude::*;
use std::io;
use rem::config::Config;
use std::net;
use std::net::ToSocketAddrs;

use native_tls::{TlsConnector, TlsStream};

trait ReadWrite : Read + Write {}

impl<T: Read + Write> ReadWrite for T {}

pub struct TcpStream{
    io_delegate : Box<ReadWrite>,
    config: Config
}

impl TcpStream where {

    pub fn connect<A: ToSocketAddrs>(config: Config, addr: A) -> io::Result<TcpStream> {
        let tcp_stream = net::TcpStream::connect(addr).unwrap();
        if config.ssl {
            let tls_stream = TlsConnector::builder().unwrap().build().unwrap().connect("rem", tcp_stream).unwrap();
            return Ok(TcpStream {
                config: config,
                io_delegate: Box::new(tls_stream)
            });
        }
        return Ok(TcpStream{
                config: config,
                io_delegate:Box::new(tcp_stream)
        });
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.io_delegate.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.io_delegate.read_to_end(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { 
        self.io_delegate.write(buf) 
    }
    fn flush(&mut self) -> io::Result<()> {
         self.io_delegate.flush()
    }
}