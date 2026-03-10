use std::path::PathBuf;

pub fn build_response(status: &str, body: Vec<u8>, mime: &str) -> Vec<u8> {
    let mut resp = format!(
        "HTTP/1.1 {status}\r\n\
        Content-Length: {}\r\n\
        Content-Type: {mime}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    resp.extend(body);
    resp
}

pub fn build_file_response(dir: &PathBuf) -> Vec<u8> {
    let ext = dir.extension().and_then(|s| s.to_str()).unwrap_or("");
    let content_type = match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream", // 万能二进制流
    };
    let content = std::fs::read(&dir);
    match content {
        Ok(content) => build_response("200 OK", content, content_type),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // 文件没找到，返回 404
            build_response("404 NOT FOUND", "File Not Found".into(), "text/plain")
        }
        Err(e) => {
            // 其他错误（如权限问题），返回 500
            eprintln!("服务器内部错误: {:?}", e); // 打印日志方便调试
            build_response("500 INTERNAL SERVER ERROR", vec![], "text/plain")
        }
    }
}
