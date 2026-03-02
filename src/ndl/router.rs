use std::fs;
use std::path::Path;

use crate::ndl::debug::log;
use crate::ndl::files;
use crate::ndl::pipeline::{self, ProcessingLocks};
use crate::ndl::response::{
    body::{CompressionType, ResponseBody},
    builder::Response,
    negotiate::{select_compression, select_error_format},
    status::StatusCode,
};

pub fn route_request(
    method: &str,
    path: &str,
    _version: &str,
    accept: Option<&str>,
    accept_encoding: Option<&str>,
    locks: &ProcessingLocks,
) -> Vec<u8> {
    let format = select_error_format(accept);
    let compression = select_compression(accept_encoding);

    if method == "OPTIONS" {
        return Response::no_content()
            .with_cors(
                "*",
                "GET, POST, PUT, DELETE, OPTIONS, PATCH",
                "Content-Type, Authorization, Accept",
            )
            .to_bytes(&CompressionType::None);
    }

    if method == "GET" || method == "HEAD" {
        let file_path = if path == "/" {
            format!("{}/index.html", files::LIVE_PATH)
        } else {
            format!("{}{}", files::LIVE_PATH, path)
        };

        let live = Path::new(&file_path);
        if let Err(e) = pipeline::ensure_up_to_date(live, locks) {
            log::error(&format!("Pipeline error for '{}': {}", file_path, e));
            return Response::error_response(StatusCode::InternalServerError, "Internal server error.", &format)
                .with_security_headers()
                .with_no_cache()
                .to_bytes(&CompressionType::None);
        }

        match fs::read(&file_path) {
            Ok(contents) => {
                log::info(&format!("Serving '{}'", file_path));
                let mut response = Response::serve_file(&file_path, contents);

                if method == "HEAD" {
                    response.body = ResponseBody::Empty;
                }

                response.to_bytes(&compression)
            }
            Err(_) => {
                log::warn(&format!("Not found: '{}'", file_path));
                Response::not_found("The requested resource could not be found.", &format)
                    .with_security_headers()
                    .with_no_cache()
                    .to_bytes(&CompressionType::None)
            }
        }
    } else {
        log::warn(&format!("{} not allowed for '{}'", method, path));
        Response::method_not_allowed(&format)
            .with_security_headers()
            .with_no_cache()
            .to_bytes(&CompressionType::None)
    }
}
