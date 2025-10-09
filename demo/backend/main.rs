// Backend service
use std::net::TcpListener;

mod routes;
mod handlers;
mod middleware;

fn main() {
    println!("Starting backend server...");
    
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("Connection established!");
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

