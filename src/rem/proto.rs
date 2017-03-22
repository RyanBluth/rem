use tokio_proto::pipeline::ServerProto;
use tokio_io::codec::{Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_tls::{TlsConnectorExt, TlsAcceptorExt};
use rem::codec::CacheCodec;
use native_tls::TlsAcceptor;
use std::io;

pub struct CacheProto {}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for CacheProto {
    /// For this protocol style, `Request` matches the codec `In` type
    type Request = String;

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = String;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, CacheCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CacheCodec{}))
    }
}