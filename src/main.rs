use std::collections::HashMap;
// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::prelude::*;

use itertools::Itertools;


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = String::new();
                stream.read_to_string(&mut buffer).unwrap();
                println!("request: {}", buffer);

                let (request_line, remained) = buffer.split_once("\r\n").unwrap();
                let mut request_line = request_line.split_whitespace().take(3);
                let method = request_line.next().unwrap();
                let path = request_line.next().unwrap();
                let version = request_line.next().unwrap();

                let (headers, body) = remained.split_once("\r\n\r\n").unwrap();
                let headers: HashMap<&str, &str> = headers.split("\r\n").map(|h| {
                    h.split_once(": ").unwrap()
                }).collect();

                match path {
                    "/" => {
                        let response = "HTTP/1.1 200 OK\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    _ => {
                        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
