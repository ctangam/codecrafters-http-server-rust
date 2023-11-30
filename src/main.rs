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

                let start_line = buffer.lines().next().unwrap();
                let (method, path, _) = start_line.split_whitespace().collect_tuple().unwrap();
                println!("method: {}, path: {}", method, path);

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
