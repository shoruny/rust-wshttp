use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use percent_encoding::percent_decode_str;

use crate::http::{
    request::{check_path, erase_query_params, get_headers, get_path},
    response::{build_file_response, build_response},
};

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for incomming in listener.incoming() {
        let Ok(stream) = incomming else {
            continue;
        };
        let _ = handle(stream);
    }
}
pub fn handle(mut stream: TcpStream) -> io::Result<()> {
    let lines = get_headers(&stream);
    let header = lines.get(0);

    let header = match header {
        Some(p) => p,
        None => "GET / HTTP/1.1",
    };
    let path = get_path(header);
    let path = check_path(path);
    let Ok(uri) = path else {
        // let body = "Forbiden".as_bytes().to_vec();
        let body = "Forbiden".into(); // 这个更快，但是 into 的变量会丢失
        let response = build_response("403 Forbiden", body, "text/html");
        stream.write_all(&response)?;
        stream.flush()?;
        return Ok(());
    };
    let path = erase_query_params(uri);
    let mut dir = PathBuf::from("/Volumes/nvme/.hide/commic");
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

// fn build_file_response(dir: &PathBuf) -> Vec<u8> {
//     let ext = dir.extension().and_then(|s| s.to_str()).unwrap_or("");
//     let content_type = match ext {
//         "html" => "text/html",
//         "css" => "text/css",
//         "js" => "application/javascript",
//         "png" => "image/png",
//         "jpg" => "image/jpeg",
//         "jpeg" => "image/jpeg",
//         "webp" => "image/webp",
//         _ => "application/octet-stream", // 万能二进制流
//     };
//     let content = std::fs::read(&dir);
//     match content {
//         Ok(content) => build_response("200 OK", content, content_type),
//         Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
//             // 文件没找到，返回 404
//             build_response("404 NOT FOUND", "File Not Found".into(), "text/plain")
//         }
//         Err(e) => {
//             // 其他错误（如权限问题），返回 500
//             eprintln!("服务器内部错误: {:?}", e); // 打印日志方便调试
//             build_response("500 INTERNAL SERVER ERROR", vec![], "text/plain")
//         }
//     }
// }
