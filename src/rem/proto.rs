use tokio_proto::pipeline::ServerProto;
use tokio_core::io::{Io, Framed};

use rem::codec;

pub struct CacheProto {}

impl<T: Io + 'static> ServerProto<T> for CacheProto {
    /// For this protocol style, `Request` matches the codec `In` type
    type Request = String;

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = String;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}