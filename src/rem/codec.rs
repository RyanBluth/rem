use std::io;
use std::str;
use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;

pub struct CacheCodec {}

impl Decoder for CacheCodec{
     type Item  = String;
     type Error = io::Error;

     fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if let Some(idx) = buf.as_ref().iter().position(|&b| b == b'|') {
            let buf_clone = buf.clone();
            let descriptor_b = &(buf_clone)[..idx];
            match str::from_utf8(descriptor_b){
                Ok(descriptor) => {
                    match descriptor.parse::<i32>() {
                        Ok(size) => {
                            if buf.len() >= size as usize + descriptor.len() as usize {
                                buf.split_to(idx + 1);
                                let content = buf.split_to(size as usize);
                                match str::from_utf8(content.as_ref()){
                                    Ok(s) => Ok(Some(String::from(s))),
                                    Err(_) => Err(io::Error::from(io::ErrorKind::InvalidData))
                                }
                            }else {
                                return Ok(None);
                            }
                        }
                        Err(_) => {
                            return Err(io::Error::from(io::ErrorKind::InvalidInput));
                        }
                    }
                }
                Err(_) => {
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for CacheCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let formatted = format!("{}|{}", msg.len(), msg);
        buf.extend(formatted.as_bytes());
        Ok(())
    }
}