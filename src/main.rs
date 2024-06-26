use std::collections::HashMap;
use std::fmt::Formatter;
// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::prelude::*;
use std::str::FromStr;

use anyhow::Error;
use itertools::Itertools;

enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
}

impl Method {
    fn from_str(s: &str) -> Option<Method> {
        match s {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "PATCH" => Some(Method::PATCH),
            "DELETE" => Some(Method::DELETE),
            "OPTIONS" => Some(Method::OPTIONS),
            "HEAD" => Some(Method::HEAD),
            _ => None,
        }
    }
}

struct Request {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl FromStr for Request {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (request_line, remained) = s.split_once("\r\n").unwrap();
        let mut request_line = request_line.split_whitespace().take(3);
        let method = request_line.next().unwrap();
        let path = request_line.next().unwrap();
        let version = request_line.next().unwrap();

        let (headers, body) = remained.split_once("\r\n\r\n").unwrap();
        let headers: HashMap<String, String> = headers.split("\r\n").map(|h| {
            let (h, v) = h.split_once(": ").unwrap();
            (h.to_string(), v.to_string())
        }).collect();

        Ok(Request {
            method: Method::from_str(method).unwrap(),
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body: body.as_bytes().to_vec(),
        })
    }
}

struct Response {
    version: String,
    status: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl ToString for Response {
    fn to_string(&self) -> String {
        let mut response = String::new();
        response.push_str(&format!("{} {}\r\n", self.version, self.status));
        for (k, v) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", k, v));
        }
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));
        response
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 8192];
                stream.read(&mut buffer).unwrap();
                let buffer = String::from_utf8_lossy(&buffer);
                println!("request: {}", buffer);

                let request = Request::from_str(&buffer).unwrap();

                match &request.path[..] {
                    "/" => {
                        let response = "HTTP/1.1 200 OK\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    path if path.starts_with("/echo") => {
                        let uri = path.strip_prefix("/echo/").unwrap();
                        
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "text/plain".to_string());
                        headers.insert("Content-Length".to_string(), uri.len().to_string());
                        let response = Response {
                            version: request.version,
                            status: "200 OK".to_string(),
                            headers,
                            body: uri.as_bytes().to_vec(),
                        };

                        stream.write_all(&response.to_string().as_bytes()).unwrap();
                    }
                    _ => {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
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
