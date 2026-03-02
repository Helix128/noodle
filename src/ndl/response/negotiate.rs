use crate::ndl::response::body::CompressionType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorFormat {
    Html,
    Json,
}

pub fn select_error_format(accept_header: Option<&str>) -> ErrorFormat {
    match accept_header {
        Some(header) => {
            let header_lower = header.to_lowercase();
            if header_lower.contains("application/json") {
                let json_q = extract_q_value(&header_lower, "application/json");
                let html_q = extract_q_value(&header_lower, "text/html");
                if json_q >= html_q {
                    ErrorFormat::Json
                } else {
                    ErrorFormat::Html
                }
            } else {
                ErrorFormat::Html
            }
        }
        None => ErrorFormat::Html,
    }
}

pub fn select_compression(accept_encoding: Option<&str>) -> CompressionType {
    match accept_encoding {
        Some(header) if header.to_lowercase().contains("gzip") => CompressionType::Gzip,
        _ => CompressionType::None,
    }
}

fn extract_q_value(header: &str, mime: &str) -> f32 {
    for part in header.split(',') {
        let part = part.trim();
        if part.starts_with(mime) || part.starts_with("*/*") {
            if let Some(q_pos) = part.find(";q=") {
                if let Ok(q) = part[q_pos + 3..].trim().parse::<f32>() {
                    return q;
                }
            }
            return 1.0;
        }
    }
    0.0
}
