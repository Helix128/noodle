use std::fs;

use crate::ndl::debug::log;
use crate::ndl::files;

pub fn route_request(method: &str, path: &str, _version: &str) -> Vec<u8> {
    if method == "GET" {
        let file_path = if path == "/" {
            format!("{}/index.html", files::PUBLIC_PATH)
        } else {
            format!("{}{}", files::PUBLIC_PATH, path)
        };

        match fs::read(&file_path) {
            Ok(contents) => {
                let content_type = get_content_type(&file_path);
                log::info(&format!("GET request routed successfully: '{}'", file_path));
                
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                    contents.len(),
                    content_type
                );
                
                let mut response = header.into_bytes();
                response.extend_from_slice(&contents);
                response
            },
            Err(_) => {
                log::warn(&format!("GET request failed (file not found at '{}')", file_path));
                "HTTP/1.1 404 (File Not Found)\r\nContent-Length: 0\r\n\r\n".as_bytes().to_vec()
            },
        }
    } else {
        log::warn(&format!("{} request not allowed for path '{}'", method, path));
        "HTTP/1.1 405 (Method Not Allowed)\r\nContent-Length: 0\r\n\r\n".as_bytes().to_vec()
    }
}

const CONTENT_TYPES: &[(&str, &str)] = &[
    (".html", "text/html"),
    (".css", "text/css"),
    (".js", "application/javascript"),
    (".png", "image/png"),
    (".jpg", "image/jpeg"),
    (".jpeg", "image/jpeg"),
    (".gif", "image/gif"),
    (".svg", "image/svg+xml"),
    (".ico", "image/x-icon"),
];

fn get_content_type(file_path: &str) -> &'static str {
    CONTENT_TYPES
        .iter()
        .find(|(ext, _)| file_path.ends_with(ext))
        .map(|(_, mime)| mime)
        .copied()
        .unwrap_or("application/octet-stream")
}