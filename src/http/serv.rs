use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::Arc,
};

use percent_encoding::percent_decode_str;

use crate::{
    http::{
        request::{check_path, erase_query_params, get_headers, get_path},
        response::{build_file_response, build_response},
    },
    ThreadPool,
};

pub fn run(serv: String, root: String) {
    let listener = TcpListener::bind(serv).unwrap();
    let pool = ThreadPool::build(7).unwrap();
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
    // "/Volumes/nvme/.hide/commic"
    let path = erase_query_params(uri);
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
