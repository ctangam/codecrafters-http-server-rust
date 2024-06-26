use core::panic;
use std::collections::HashMap;
use std::fmt::Formatter;
// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use std::thread;

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

    

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: std::net::TcpStream) {
    let mut buffer = [0; 8192];
    let n = stream.read(&mut buffer).unwrap();
    let buffer = String::from_utf8_lossy(&buffer[..n]);
    println!("request: {}", buffer);

    let request = Request::from_str(&buffer).unwrap();

    match (request.method, &request.path[..]) {
        (Method::GET, "/") => {
            let response = Response {
                version: request.version,
                status: "200 OK".to_string(),
                headers: HashMap::new(),
                body: vec![],
            };
            stream.write_all(&response.to_string().as_bytes()).unwrap();
        }
        (Method::GET, "/user-agent") => {
            let user_agent = request.headers.get("User-Agent").unwrap();
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "text/plain".to_string());
            headers.insert("Content-Length".to_string(), user_agent.len().to_string());
            let response = Response {
                version: request.version,
                status: "200 OK".to_string(),
                headers: headers,
                body: user_agent.as_bytes().to_vec(),
            };
            stream.write_all(&response.to_string().as_bytes()).unwrap();
        }
        (Method::GET, path) if path.starts_with("/echo") => {
            let uri = path.strip_prefix("/echo/").unwrap();
        
            let mut headers = HashMap::new();
            if request.headers.contains_key("Accept-Encoding") && request.headers.get("Accept-Encoding").unwrap().contains("gzip") {
                headers.insert("Content-Encoding".to_string(), "gzip".to_string());
            }
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
        (Method::GET, path) if path.starts_with("/files") => {
            let file_path = path.strip_prefix("/files/").unwrap();
            let args: Vec<String> = std::env::args().collect_vec();
            let file_dir = args[2].clone();
            let file_path = file_dir + file_path;
            if !Path::new(&file_path).exists() {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
            let file = std::fs::read(file_path).unwrap();
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/octet-stream".to_string());
            headers.insert("Content-Length".to_string(), file.len().to_string());
            let response = Response {
                version: request.version,
                status: "200 OK".to_string(),
                headers,
                body: file,
            };

            stream.write_all(&response.to_string().as_bytes()).unwrap();
        }
        (Method::POST, path) if path.starts_with("/files") => {
            let file_path = path.strip_prefix("/files/").unwrap();
            let args: Vec<String> = std::env::args().collect_vec();
            let file_dir = args[2].clone();
            let file_path = file_dir + file_path;
            let mut file = std::fs::File::create(file_path).unwrap();
            file.write_all(&request.body).unwrap();
            let response = "HTTP/1.1 201 Created\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}
