use std::io::prelude::*;
use std::io;
use rem::config::Config;
use std::net;
use std::net::ToSocketAddrs;
use rem::error::RemError;

use native_tls::{TlsConnector};

trait ReadWrite : Read + Write {}

impl<T: Read + Write> ReadWrite for T {}

pub struct TcpStream{
    io_delegate : Box<ReadWrite>
}

/// Wraps std::net::TcpStream and native_tls::TcpStream
/// This allows for one Tcp interface which supports TLS and regular TCP
impl TcpStream {

    pub fn connect<A: ToSocketAddrs>(config: &Config, addr: A) -> Result<TcpStream, RemError> {
        let tcp_stream = try!(net::TcpStream::connect(addr));
        if config.ssl {
            let tls_builder   = try!(TlsConnector::builder());
            let tls_connector = try!(tls_builder.build());
            let tls_stream    = try!(tls_connector.connect("rem", tcp_stream));
            return Ok(TcpStream {
                io_delegate: Box::new(tls_stream)
            });
        }
        return Ok(TcpStream{
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