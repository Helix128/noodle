use crate::ndl::response::body::{
    html_error_page, json_error, CompressionType, ResponseBody,
};
use crate::ndl::response::content_type::ContentType;
use crate::ndl::response::headers::Headers;
use crate::ndl::response::negotiate::ErrorFormat;
use crate::ndl::response::status::StatusCode;

pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: ResponseBody,
    pub version: &'static str,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Response {
            status,
            headers: Headers::new(),
            body: ResponseBody::Empty,
            version: "HTTP/1.1",
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.set(key, value);
        self
    }

    pub fn content_type(mut self, ct: ContentType) -> Self {
        self.headers.set("Content-Type", ct.as_str());
        self
    }

    pub fn body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = ResponseBody::Bytes(bytes);
        self
    }

    pub fn body_text(mut self, text: String) -> Self {
        self.body = ResponseBody::Text(text);
        self
    }

    pub fn body_json(mut self, json: String) -> Self {
        self.body = ResponseBody::Json(json);
        self
    }

    pub fn with_security_headers(mut self) -> Self {
        self.headers.apply_security();
        self
    }

    pub fn with_cache(mut self, max_age_secs: u64) -> Self {
        self.headers.apply_cache(max_age_secs);
        self
    }

    pub fn with_no_cache(mut self) -> Self {
        self.headers.apply_no_cache();
        self
    }

    pub fn with_cors(mut self, origin: &str, methods: &str, headers: &str) -> Self {
        self.headers.apply_cors(origin, methods, headers);
        self
    }

    pub fn to_bytes(self, compression: &CompressionType) -> Vec<u8> {
        let body_data = self.body.compress(compression);

        let content_type_set = self.headers.contains("Content-Type");

        let mut headers = self.headers;

        if matches!(compression, CompressionType::Gzip) && !body_data.is_empty() {
            headers.set("Content-Encoding", "gzip");
        }

        headers.set("Content-Length", &body_data.len().to_string());

        if !content_type_set && !body_data.is_empty() {
            headers.set("Content-Type", ContentType::ApplicationOctetStream.as_str());
        }

        headers.set("Connection", "close");

        let status_line = format!(
            "{} {} {}\r\n",
            self.version,
            self.status.code(),
            self.status.reason_phrase()
        );

        let mut output = status_line.into_bytes();
        output.extend_from_slice(&headers.to_bytes());
        output.extend_from_slice(b"\r\n");
        output.extend_from_slice(&body_data);
        output
    }

    pub fn ok() -> Self {
        Response::new(StatusCode::Ok)
    }

    pub fn created() -> Self {
        Response::new(StatusCode::Created)
    }

    pub fn no_content() -> Self {
        Response::new(StatusCode::NoContent)
    }

    pub fn not_modified() -> Self {
        Response::new(StatusCode::NotModified)
    }

    pub fn error_response(status: StatusCode, message: &str, format: &ErrorFormat) -> Self {
        match format {
            ErrorFormat::Json => {
                let body = json_error(&status, message);
                Response::new(status)
                    .content_type(ContentType::ApplicationJson)
                    .body_json(match body {
                        ResponseBody::Json(s) => s,
                        _ => String::new(),
                    })
            }
            ErrorFormat::Html => {
                let body = html_error_page(&status, message);
                Response::new(status)
                    .content_type(ContentType::TextHtml)
                    .body_text(match body {
                        ResponseBody::Text(s) => s,
                        _ => String::new(),
                    })
            }
        }
    }

    pub fn bad_request(message: &str, format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::BadRequest, message, format)
    }

    pub fn not_found(message: &str, format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::NotFound, message, format)
    }

    pub fn method_not_allowed(format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::MethodNotAllowed, "The request method is not allowed for this resource.", format)
    }

    pub fn payload_too_large(format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::PayloadTooLarge, "The request body exceeds the maximum allowed size.", format)
    }

    pub fn too_many_requests(format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::TooManyRequests, "Too many requests. Please slow down.", format)
    }

    pub fn internal_error(message: &str, format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::InternalServerError, message, format)
    }

    pub fn http_version_not_supported(format: &ErrorFormat) -> Self {
        Response::error_response(StatusCode::HttpVersionNotSupported, "HTTP version not supported. Use HTTP/1.0 or HTTP/1.1.", format)
    }

    pub fn serve_file(file_path: &str, contents: Vec<u8>) -> Self {
        let content_type = ContentType::from_extension(file_path);
        let is_text = content_type.is_text();
        let cache_secs: u64 = if is_text { 300 } else { 86400 };

        Response::ok()
            .content_type(content_type)
            .body_bytes(contents)
            .with_cache(cache_secs)
            .with_security_headers()
    }
}
