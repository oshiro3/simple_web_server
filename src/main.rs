use std::net::Shutdown;
use std::net::TcpListener;

fn main() {
    println!("=== server start ===");

    let listener = TcpListener::bind("localhost:8080").unwrap();

    match listener.accept() {
        Ok((sock, addr)) => {
            println!("=== complete communication! remote_address: {} ===", addr);
            sock.shutdown(Shutdown::Both)
                .expect("shutdown call failed");
        }
        Err(e) => println!("couldn't get client: {:?}", e),
    }
}
