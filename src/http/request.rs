use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

pub fn get_headers(stream: &TcpStream) -> Vec<String> {
    let reader = BufReader::new(stream);
    let lines: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    lines
}

pub fn get_path(header: &str) -> &str {
    let paths: Vec<&str> = header
        .split_ascii_whitespace()
        // .map(|s| s.to_string()) // 此时 paths 应为 Vec<String>
        .collect();
    let path = paths.get(1).copied().unwrap_or("/");
    let path = match path {
        "/" => "index.html",
        _ => &path[1..],
    };
    // let path = &path[1..];
    path
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
