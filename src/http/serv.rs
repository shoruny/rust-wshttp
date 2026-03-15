use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    http::{
        request::{check_path, erase_query_params, get_headers},
        response::{build_file_response, build_response},
    },
    ThreadPool,
};
use base64::{engine::general_purpose, Engine};
use bytes::{BufMut, BytesMut};
use percent_encoding::percent_decode_str;
use sha1::{Digest, Sha1};

pub fn run(serv: String, root: String) {
    let listener = TcpListener::bind(serv).unwrap();
    let pool = ThreadPool::build(200).unwrap();
    let path = Arc::new(root);
    for incomming in listener.incoming() {
        let Ok(stream) = incomming else {
            continue;
        };
        let root = Arc::clone(&path);
        pool.execute(move || {
            _ = handle(stream, root);
        });
    }
}

pub fn handle(mut stream: TcpStream, root: Arc<String>) -> io::Result<()> {
    let headers = get_headers(&stream);
    let headers = match headers {
        Ok(v) => v,
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    };

    let path = check_path(&headers.path);
    let Ok(uri) = path else {
        let body = "Forbiden".into(); // 这个更快，但是 into 的变量会丢失
        let response = build_response("403 Forbiden", body, "text/html");
        stream.write_all(&response)?;
        stream.flush()?;
        return Ok(());
    };
    // "/Volumes/nvme/.hide/commic"
    let path = erase_query_params(uri);
    let path = path.strip_prefix("/").unwrap_or(path);
    if path == "ws" {
        //websocket连接
        let upgrade = headers
            .headers
            .get("Upgrade")
            .map(|l| l.to_string())
            .unwrap_or_else(|| "".to_string());
        println!("{upgrade}");
        if upgrade.len() == 0 {
            return Ok(());
        }
        let sec_key = headers
            .headers
            .get("Sec-WebSocket-Key")
            .map(|l| l.to_string())
            .unwrap_or_else(|| "".to_string());
        let magic_uuid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let mut hasher = Sha1::new();
        hasher.update(sec_key.as_bytes());
        hasher.update(magic_uuid.as_bytes());
        let result = hasher.finalize();
        let encoded = general_purpose::STANDARD.encode(result);
        let response = [
            "HTTP/1.1 101 Switching Protocols",
            "Upgrade: websocket",
            "Connection: Upgrade",
            &format!("Sec-WebSocket-Accept: {}", encoded),
            "\r\n",
        ]
        .join("\r\n");

        stream.write_all(response.as_bytes()).expect("握手失败");
        println!("{response}");
        let mut buf = BytesMut::with_capacity(4096);

        loop {
            let mut buffer = [0u8; 4096];
            // 这里的 read 是阻塞的，会一直等到有数据或者连接断开
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("客户端优雅地关闭了连接");
                    break; // 跳出循环，结束线程
                }
                Ok(n) => {
                    buf.put_slice(&buffer[..n]);
                    if buf.len() < 2 {
                        continue;
                    }
                    let fin: u8 = buf[0] >> 7;
                    let opcode: u8 = buf[0] & 0x0f;
                    let is_masked = buf[1] >> 7;
                    let mut len: u64 = (buf[1] & 0x7F) as u64;
                    let mut header_len = 2;
                    if len == 126 && buf.len() < 4 {
                        continue;
                    } else if len == 127 {
                        println!("包太大，溜了溜了");
                        return Ok(());
                    }
                    if len == 126 {
                        header_len = 4;
                        len = u16::from_be_bytes([buf[2], buf[3]]) as u64;
                    }
                    if buf.len() < (header_len + len + 4) as usize {
                        continue;
                    }

                    // println!("is_fin: {fin}, opcode: {opcode}, is_masked: {is_masked}, len: {len}, {buf:#?}");
                    let _ = buf.split_to(header_len as usize);
                    let mask = buf.split_to(4).to_vec();
                    let body = buf.split_to(len as usize).to_vec();

                    let mut decoded_body = body;

                    // 2. 进行 XOR 异或运算
                    for i in 0..decoded_body.len() {
                        // 掩码只有 4 字节，所以用 i % 4 循环取用
                        decoded_body[i] ^= mask[i % 4];
                    }

                    if opcode == 0x08 {
                        // 关闭
                        let _ = stream.write_all(&[0x88, 0x00]);
                        let _ = stream.flush();
                        return Ok(());
                    }
                    let message = String::from_utf8_lossy(&decoded_body);
                    println!("解密后的消息: {}", message);
                    println!("is_fin: {fin}, opcode: {opcode}, is_masked: {is_masked}, len: {len}, body: {message}");
                    println!("收到来自的数据: {} 字节", n);

                    let mut frame = Vec::with_capacity(2 + message.len());
                    frame.push(0x81);
                    let message = format!("收到了{}", message);
                    println!("{message}");
                    let send = message.as_bytes();
                    if send.len() <= 125 {
                        frame.push(send.len() as u8);
                    } else {
                        frame.push(126u8);
                        let bytes = (send.len() as u16).to_be_bytes();
                        frame.extend_from_slice(&bytes);
                    }
                    frame.extend_from_slice(send);
                    stream.write_all(&frame).unwrap();
                }
                Err(e) => {
                    println!("客户端连接发生错误: {}", e);
                    break; // 同样跳出循环
                }
            }
        }
        return Ok(());
    } else {
        return handle_http(stream, path, root);
    }
}

fn handle_http(mut stream: TcpStream, path: &str, root: Arc<String>) -> io::Result<()> {
    let mut dir = PathBuf::from(&*root);
    dir.push(percent_decode_str(path).decode_utf8_lossy().to_string());
    let mut response: Vec<u8> = vec![];
    if dir.is_file() {
        response = build_file_response(&dir);
    } else {
        let body = "Not Found".into();
        response = build_response("404 Not Found", body, "text/html");
    }
    stream.write_all(&response)?;
    stream.flush()?;
    return Ok(());
}
