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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let current = env::current_dir().unwrap();
    let static_dir = current.join("statics").canonicalize().unwrap();

    println!("static_dir: {}", static_dir.to_str().unwrap());

    let buffer_str = str::from_utf8(&buffer).unwrap();

    let mut splitter = buffer_str.splitn(2, "\r\n");
    let status_line = splitter.next().unwrap();
    println!("status_line: {}", status_line);
    let mut splitter = status_line.split(" ");
    let method = splitter.next().unwrap();
    let path = Path::new(splitter.next().unwrap());
    let http_version = splitter.next().unwrap();
    let relative_path = path.strip_prefix("/").unwrap();
    println!("relative path: {}", relative_path.to_str().unwrap());
    let fname = static_dir.join(relative_path);
    println!("fname: {}", fname.to_str().unwrap());
    println!("fname exists?: {}", fname.exists());
    let mut response = String::new();
    if fname.exists() {
        response = build_response(fname.to_str().unwrap(), "HTTP/1.1 200 OK\r\n\r\n");
    } else {
        response = build_response(
            static_dir.join("404.html").to_str().unwrap(),
            "HTTP/1.1 404 NOT FOUND\r\n\r\n",
        );
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    println!("Response: {}", response);

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

fn build_response(fname: &str, header: &str) -> String {
    let mut file = File::open(fname).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    format!("{}{}", header, contents)
}
