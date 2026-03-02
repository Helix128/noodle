use std::io::{Read, Write};
use std::time::Duration;
use threadpool::ThreadPool;

use crate::ndl::debug::log;
use crate::ndl::router::route_request;

const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
const READ_TIMEOUT_SECS: u64 = 10;
const WRITE_TIMEOUT_SECS: u64 = 10;
const MAX_WORKERS: usize = 128;
const BUFFER_SIZE: usize = 8192;

pub struct HttpListener {
    port: u16,
    pool: ThreadPool,
}

impl HttpListener {
    pub fn new(port: u16) -> Self {
        let num_workers = (num_cpus::get() * 2).min(MAX_WORKERS);
        HttpListener { 
            port,
            pool: ThreadPool::new(num_workers),
        }
    }

    pub fn start(&self) {
        let address = format!("127.0.0.1:{}", self.port);
        let listener = match std::net::TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                log::error(&format!("Failed to bind to address {}: {}", address, e));
                return;
            }
        };
        
        log::info(&format!("HTTP server is running on {} with {} workers", address, self.pool.max_count()));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    match stream.peer_addr() {
                        Ok(addr) => log::info(&format!("New connection from {}", addr)),
                        Err(e) => log::error(&format!("Failed to get peer address: {}", e)),
                    }
                    self.pool.execute(move || {
                        Self::handle_client(stream);
                    });
                }
                Err(e) => {
                    log::error(&format!("Failed to accept connection: {}", e));
                }
            }
        }
    }

    fn handle_client(mut stream: std::net::TcpStream) {
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS))) {
            log::error(&format!("Failed to set read timeout: {}", e));
            return;
        }
        if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT_SECS))) {
            log::error(&format!("Failed to set write timeout: {}", e));
            return;
        }

        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut total_read = 0;
        
        loop {
            match stream.read(&mut buffer[total_read..]) {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n;
                    
                    if total_read > MAX_REQUEST_SIZE {
                        Self::send_error_response(&mut stream, 413, "Payload Too Large");
                        return;
                    }

                    if total_read >= buffer.len() && buffer.len() < MAX_REQUEST_SIZE {
                        buffer.resize((buffer.len() * 2).min(MAX_REQUEST_SIZE), 0);
                    }
                    
                    if total_read >= 4 {
                        let read_data = &buffer[..total_read];
                        if read_data.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log::error(&format!("Failed to read from stream: {}", e));
                    return;
                }
            }
        }

        if total_read == 0 {
            return;
        }

        let request_str = match std::str::from_utf8(&buffer[..total_read]) {
            Ok(s) => s,
            Err(_) => {
                Self::send_error_response(&mut stream, 400, "Bad Request");
                return;
            }
        };

        let first_line = match request_str.lines().next() {
            Some(line) => line,
            None => {
                Self::send_error_response(&mut stream, 400, "Bad Request");
                return;
            }
        };

        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            Self::send_error_response(&mut stream, 400, "Bad Request");
            return;
        }

        let method = parts[0];
        let _path = parts[1];
        let version = parts[2];
        if !matches!(method, "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH") {
            Self::send_error_response(&mut stream, 405, "Method Not Allowed");
            return;
        }

        if !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
            Self::send_error_response(&mut stream, 505, "HTTP Version Not Supported");
            return;
        }

        log::info(&format!("Received request: {}", first_line));

       let response = route_request(method, _path, version);

        if let Err(e) = stream.write_all(&response) {
            log::error(&format!("Failed to write response: {}", e));
            return;
        }

        if let Err(e) = stream.flush() {
            log::error(&format!("Failed to flush stream: {}", e));
        }

        let _ = stream.shutdown(std::net::Shutdown::Both);
    }

    fn send_error_response(stream: &mut std::net::TcpStream, code: u16, message: &str) {
        let response = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Length: 0\r\n\
             Connection: close\r\n\
             \r\n",
            code, message
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}
