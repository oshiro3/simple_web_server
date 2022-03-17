use chunked_transfer::Encoder;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;

extern crate server;
use server::ThreadPool;

fn main() {
    println!("=== server start ===");

    let listener = TcpListener::bind("localhost:8080").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn parse_request_buffer(buffer_str: &str) -> &Path {
    let mut splitter = buffer_str.splitn(2, "\r\n");
    let status_line = splitter.next().unwrap();
    let request_header = splitter.next().unwrap();
    println!("status_line: {}", status_line);
    println!("request_header: {}", request_header);
    let mut splitter = status_line.split(" ");
    let _method = splitter.next().unwrap();
    let path = Path::new(splitter.next().unwrap());
    let _http_version = splitter.next().unwrap();
    return path.strip_prefix("/").unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let buffer_str = str::from_utf8(&buffer).unwrap();

    let current = env::current_dir().unwrap();
    let static_dir = current.join("statics").canonicalize().unwrap();
    println!("static_dir: {}", static_dir.to_str().unwrap());

    let relative_path = parse_request_buffer(buffer_str);
    let fname = static_dir.join(relative_path);
    println!("fname: {}", fname.to_str().unwrap());
    println!("fname exists?: {}", fname.exists());

    if fname.exists() {
        let response = build_response_header(fname.to_str().unwrap(), "HTTP/1.1 200 OK\r\n");
        println!("hoge2");
        stream.write(response.as_bytes()).unwrap();
        let ext = Path::new(&fname).extension().unwrap();
        let mut buf = Vec::new();
        let mut file = File::open(&fname).unwrap();
        file.read_to_end(&mut buf).unwrap();

        if ext == "jpg" || ext == "png" {
            let mut encoded = Vec::new();
            {
                let mut encoder = Encoder::with_chunks_size(&mut encoded, 8);
                encoder.write_all(&buf).unwrap();
            }
        }
        stream.write(&buf).unwrap();
    } else {
        let response = build_response_header(
            static_dir.join("404.html").to_str().unwrap(),
            "HTTP/1.1 404 NOT FOUND\r\n\r\n",
        );
        stream.write(response.as_bytes()).unwrap();
    };

    stream.flush().unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

fn build_response_header(fname: &str, header: &str) -> String {
    let mut mime_types = HashMap::new();
    mime_types.insert("html", "text/html");
    mime_types.insert("css", "text/css");
    mime_types.insert("png", "image/png");
    mime_types.insert("jpg", "image/jpg");

    let ext = Path::new(fname).extension().unwrap();
    let content_type = mime_types.get(ext.to_str().unwrap());
    println!("content_type: {}", content_type.unwrap());

    let response_header = format!("Content-Type: {}\r\n\r\n", content_type.unwrap());

    format!("{}{}", header, response_header)
}
