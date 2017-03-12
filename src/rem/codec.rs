use std;
use std::io;
use std::str;
use tokio_core::io::{Codec, EasyBuf};

pub struct CacheCodec {}

impl Codec for CacheCodec {
    type In  = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        if let Some(idx) = buf.as_slice().iter().position(|&b| b == b'|') {
            let buf_clone = buf.clone();
            let descriptor_b = &(buf_clone.as_slice())[..idx];
            match str::from_utf8(descriptor_b){
                Ok(descriptor) => {
                    match descriptor.parse::<i32>() {
                        Ok(size) => {
                            if buf.len() >= size as usize + descriptor.len() as usize {
                                buf.drain_to(idx + 1);
                                let content = buf.drain_to(size as usize);
                                // TODO - improve and add error handling
                                let content_string = String::from_utf8(Vec::from(content.as_slice())).unwrap();
                                return Ok(Some(content_string));
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

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        let formatted = format!("{}|{}", msg.len(), msg);
        buf.extend(formatted.as_bytes());
        Ok(())
    }
}