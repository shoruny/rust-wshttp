use std::{
    collections::HashMap,
    io::{self, Read},
    net::TcpStream,
};

use bytes::{BufMut, BytesMut};
use httparse::{Status, EMPTY_HEADER};

#[derive(Debug)]
pub struct ParsedHeader {
    pub headers: HashMap<String, String>,
    pub method: String,
    pub path: String,
    pub body: BytesMut,
}
impl ParsedHeader {
    pub fn from(req: httparse::Request) -> ParsedHeader {
        ParsedHeader {
            method: req.method.unwrap_or("GET").to_string(),
            path: req.path.unwrap_or("/").to_string(),
            headers: req
                .headers
                .iter()
                .filter(|h| !h.name.is_empty())
                .map(|h| {
                    (
                        h.name.to_string(),
                        String::from_utf8_lossy(h.value).to_string(),
                    )
                })
                .collect(),
            body: BytesMut::with_capacity(0),
        }
    }
}
pub fn get_headers(mut stream: &TcpStream) -> io::Result<ParsedHeader> {
    let mut buf = BytesMut::with_capacity(4096);
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream.read(&mut chunk).unwrap_or(0);
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, ""));
        }
        buf.put_slice(&chunk[..n]);
        let mut headers = [EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&buf) {
            Ok(Status::Complete(amt)) => {
                let req = ParsedHeader::from(req);
                let len = req.headers.get("Content-Length");
                let len = match len {
                    Some(len) => len.to_string(),
                    _ => "0".to_string(),
                };
                let len = len.parse::<u16>().unwrap_or(0);
                let _ = buf.split_to(amt);
                if len == 0 {
                    assert_eq!(len, buf.len() as u16);
                }
                return Ok(req);
            }
            Ok(Status::Partial) => continue,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

pub fn check_path(path: &str) -> Result<&str, i16> {
    if path.contains("..") {
        Err(403)
    } else {
        Ok(path)
    }
}

pub fn erase_query_params(uri: &str) -> &str {
    uri.split("?").next().unwrap_or(uri)
}
