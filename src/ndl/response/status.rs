#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok,
    Created,
    NoContent,
    MovedPermanently,
    Found,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    PayloadTooLarge,
    TooManyRequests,
    InternalServerError,
    NotImplemented,
    ServiceUnavailable,
    HttpVersionNotSupported,
}

impl StatusCode {
    pub fn code(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::NoContent => 204,
            StatusCode::MovedPermanently => 301,
            StatusCode::Found => 302,
            StatusCode::NotModified => 304,
            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::PayloadTooLarge => 413,
            StatusCode::TooManyRequests => 429,
            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::ServiceUnavailable => 503,
            StatusCode::HttpVersionNotSupported => 505,
        }
    }

    pub fn reason_phrase(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::NoContent => "No Content",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::NotModified => "Not Modified",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::PayloadTooLarge => "Payload Too Large",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::HttpVersionNotSupported => "HTTP Version Not Supported",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self.code(), 200..=299)
    }

    pub fn is_error(&self) -> bool {
        self.code() >= 400
    }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        match code {
            200 => StatusCode::Ok,
            201 => StatusCode::Created,
            204 => StatusCode::NoContent,
            301 => StatusCode::MovedPermanently,
            302 => StatusCode::Found,
            304 => StatusCode::NotModified,
            400 => StatusCode::BadRequest,
            401 => StatusCode::Unauthorized,
            403 => StatusCode::Forbidden,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            413 => StatusCode::PayloadTooLarge,
            429 => StatusCode::TooManyRequests,
            500 => StatusCode::InternalServerError,
            501 => StatusCode::NotImplemented,
            503 => StatusCode::ServiceUnavailable,
            505 => StatusCode::HttpVersionNotSupported,
            _ => StatusCode::InternalServerError,
        }
    }
}
