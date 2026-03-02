#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    TextHtml,
    TextPlain,
    TextCss,
    ApplicationJson,
    ApplicationJavascript,
    ApplicationOctetStream,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageSvg,
    ImageAvif,
    ImageIcon,
    ImageWebp,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::TextHtml => "text/html; charset=utf-8",
            ContentType::TextPlain => "text/plain; charset=utf-8",
            ContentType::TextCss => "text/css",
            ContentType::ApplicationJson => "application/json; charset=utf-8",
            ContentType::ApplicationJavascript => "application/javascript",
            ContentType::ApplicationOctetStream => "application/octet-stream",
            ContentType::ImagePng => "image/png",
            ContentType::ImageJpeg => "image/jpeg",
            ContentType::ImageGif => "image/gif",
            ContentType::ImageSvg => "image/svg+xml",
            ContentType::ImageAvif => "image/avif",
            ContentType::ImageIcon => "image/x-icon",
            ContentType::ImageWebp => "image/webp",
        }
    }

    pub fn from_extension(path: &str) -> ContentType {
        let lower = path.to_lowercase();
        if lower.ends_with(".html") || lower.ends_with(".htm") {
            ContentType::TextHtml
        } else if lower.ends_with(".css") {
            ContentType::TextCss
        } else if lower.ends_with(".js") || lower.ends_with(".mjs") {
            ContentType::ApplicationJavascript
        } else if lower.ends_with(".json") {
            ContentType::ApplicationJson
        } else if lower.ends_with(".png") {
            ContentType::ImagePng
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            ContentType::ImageJpeg
        } else if lower.ends_with(".gif") {
            ContentType::ImageGif
        } else if lower.ends_with(".svg") {
            ContentType::ImageSvg
        } else if lower.ends_with(".avif") {
            ContentType::ImageAvif
        } else if lower.ends_with(".ico") {
            ContentType::ImageIcon
        } else if lower.ends_with(".webp") {
            ContentType::ImageWebp
        } else if lower.ends_with(".txt") {
            ContentType::TextPlain
        } else {
            ContentType::ApplicationOctetStream
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self,
            ContentType::TextHtml
                | ContentType::TextPlain
                | ContentType::TextCss
                | ContentType::ApplicationJson
                | ContentType::ApplicationJavascript
        )
    }
}
