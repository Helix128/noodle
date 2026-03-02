use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use threadpool::ThreadPool;

use crate::ndl::debug::log;
use crate::ndl::pipeline::{self, ProcessingLocks};
use crate::ndl::response::{
    body::CompressionType,
    builder::Response,
    middleware::RateLimiter,
    negotiate::ErrorFormat,
};
use crate::ndl::router::route_request;

const MAX_REQUEST_SIZE: usize = 1024 * 1024;
const READ_TIMEOUT_SECS: u64 = 10;
const WRITE_TIMEOUT_SECS: u64 = 10;
const MAX_WORKERS: usize = 128;
const BUFFER_SIZE: usize = 8192;

pub struct HttpListener {
    port: u16,
    pool: ThreadPool,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    processing_locks: ProcessingLocks,
}

impl HttpListener {
    pub fn new(port: u16) -> Self {
        let num_workers = (num_cpus::get() * 2).min(MAX_WORKERS);
        HttpListener {
            port,
            pool: ThreadPool::new(num_workers),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
            processing_locks: pipeline::new_locks(),
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

        log::info(&format!(
            "HTTP server is running on {} with {} workers",
            address,
            self.pool.max_count()
        ));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peer_addr = match stream.peer_addr() {
                        Ok(addr) => {
                            log::info(&format!("New connection from {}", addr));
                            Some(addr.ip())
                        }
                        Err(e) => {
                            log::error(&format!("Failed to get peer address: {}", e));
                            None
                        }
                    };

                    let rate_limiter = Arc::clone(&self.rate_limiter);
                    let processing_locks = Arc::clone(&self.processing_locks);

                    self.pool.execute(move || {
                        if let Some(ip) = peer_addr {
                            let allowed = rate_limiter
                                .lock()
                                .map(|mut rl| rl.check(ip))
                                .unwrap_or(true);

                            if !allowed {
                                log::warn(&format!("Rate limit exceeded for {}", ip));
                                Self::send_response(
                                    stream,
                                    Response::too_many_requests(&ErrorFormat::Html)
                                        .with_no_cache()
                                        .to_bytes(&CompressionType::None),
                                );
                                return;
                            }
                        }

                        Self::handle_client(stream, &processing_locks);
                    });
                }
                Err(e) => {
                    log::error(&format!("Failed to accept connection: {}", e));
                }
            }
        }
    }

    fn handle_client(mut stream: std::net::TcpStream, locks: &ProcessingLocks) {
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
                        Self::send_response(
                            stream,
                            Response::payload_too_large(&ErrorFormat::Html).to_bytes(&CompressionType::None),
                        );
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
                Self::send_response(
                    stream,
                    Response::bad_request("Request contains invalid UTF-8.", &ErrorFormat::Html)
                        .to_bytes(&CompressionType::None),
                );
                return;
            }
        };

        let first_line = match request_str.lines().next() {
            Some(line) => line,
            None => {
                Self::send_response(
                    stream,
                    Response::bad_request("Request is missing the request line.", &ErrorFormat::Html)
                        .to_bytes(&CompressionType::None),
                );
                return;
            }
        };

        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            Self::send_response(
                stream,
                Response::bad_request("Malformed request line.", &ErrorFormat::Html)
                    .to_bytes(&CompressionType::None),
            );
            return;
        }

        let method = parts[0];
        let path = parts[1];
        let version = parts[2];

        if !matches!(
            method,
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH"
        ) {
            Self::send_response(
                stream,
                Response::method_not_allowed(&ErrorFormat::Html).to_bytes(&CompressionType::None),
            );
            return;
        }

        if !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
            Self::send_response(
                stream,
                Response::http_version_not_supported(&ErrorFormat::Html)
                    .to_bytes(&CompressionType::None),
            );
            return;
        }

        log::info(&format!("Request: {}", first_line));

        let accept = Self::extract_header(request_str, "Accept");
        let accept_encoding = Self::extract_header(request_str, "Accept-Encoding");

        let response = route_request(
            method,
            path,
            version,
            accept.as_deref(),
            accept_encoding.as_deref(),
            locks,
        );

        Self::send_response(stream, response);
    }

    fn extract_header(request: &str, name: &str) -> Option<String> {
        let search = format!("{}:", name.to_lowercase());
        for line in request.lines().skip(1) {
            let lower = line.to_lowercase();
            if lower.starts_with(&search) {
                return Some(line[name.len() + 1..].trim().to_string());
            }
        }
        None
    }

    fn send_response(mut stream: std::net::TcpStream, data: Vec<u8>) {
        if let Err(e) = stream.write_all(&data) {
            log::error(&format!("Failed to write response: {}", e));
            return;
        }
        if let Err(e) = stream.flush() {
            log::error(&format!("Failed to flush stream: {}", e));
        }
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}
