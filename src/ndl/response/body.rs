use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

use crate::ndl::response::status::StatusCode;

#[derive(Debug, Clone)]
pub enum CompressionType {
    Gzip,
    None,
}

#[derive(Debug, Clone)]
pub enum ResponseBody {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
    Json(String),
}

impl ResponseBody {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ResponseBody::Empty => vec![],
            ResponseBody::Bytes(b) => b.clone(),
            ResponseBody::Text(s) => s.as_bytes().to_vec(),
            ResponseBody::Json(s) => s.as_bytes().to_vec(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ResponseBody::Empty => 0,
            ResponseBody::Bytes(b) => b.len(),
            ResponseBody::Text(s) => s.len(),
            ResponseBody::Json(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn compress(&self, compression: &CompressionType) -> Vec<u8> {
        let raw = self.to_bytes();
        match compression {
            CompressionType::None => raw,
            CompressionType::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                if encoder.write_all(&raw).is_ok() {
                    encoder.finish().unwrap_or(raw)
                } else {
                    raw
                }
            }
        }
    }
}

pub fn html_error_page(status: &StatusCode, message: &str) -> ResponseBody {
    let code = status.code();
    let reason = status.reason_phrase();
    let body = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{code} {reason}</title>\
        <style>body{{font-family:sans-serif;max-width:600px;margin:4rem auto;padding:0 1rem}}\
        h1{{color:#c0392b}}p{{color:#555}}</style></head>\
        <body><h1>{code} {reason}</h1><p>{message}</p></body></html>"
    );
    ResponseBody::Text(body)
}

pub fn json_error(status: &StatusCode, message: &str) -> ResponseBody {
    let code = status.code();
    let reason = status.reason_phrase();
    let body = format!(
        "{{\"error\":{{\"code\":{code},\"status\":\"{reason}\",\"message\":\"{message}\"}}}}"
    );
    ResponseBody::Json(body)
}
