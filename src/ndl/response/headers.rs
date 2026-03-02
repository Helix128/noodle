#[derive(Debug, Clone, Default)]
pub struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    pub fn new() -> Self {
        Headers { entries: Vec::new() }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let key_lower = key.to_lowercase();
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| k.to_lowercase() == key_lower) {
            entry.1 = value.to_string();
        } else {
            self.entries.push((key.to_string(), value.to_string()));
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_string(), value.to_string()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_lowercase();
        self.entries
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }

    pub fn remove(&mut self, key: &str) {
        let key_lower = key.to_lowercase();
        self.entries.retain(|(k, _)| k.to_lowercase() != key_lower);
    }

    pub fn contains(&self, key: &str) -> bool {
        let key_lower = key.to_lowercase();
        self.entries.iter().any(|(k, _)| k.to_lowercase() == key_lower)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = String::new();
        for (key, value) in &self.entries {
            output.push_str(key);
            output.push_str(": ");
            output.push_str(value);
            output.push_str("\r\n");
        }
        output.into_bytes()
    }

    pub fn apply_security(&mut self) {
        self.set("X-Frame-Options", "DENY");
        self.set("X-Content-Type-Options", "nosniff");
        self.set("X-XSS-Protection", "1; mode=block");
        self.set("Referrer-Policy", "strict-origin-when-cross-origin");
        self.set(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:",
        );
    }

    pub fn apply_cache(&mut self, max_age_secs: u64) {
        self.set("Cache-Control", &format!("public, max-age={}", max_age_secs));
    }

    pub fn apply_no_cache(&mut self) {
        self.set("Cache-Control", "no-store, no-cache, must-revalidate");
        self.set("Pragma", "no-cache");
    }

    pub fn apply_cors(&mut self, origin: &str, methods: &str, headers: &str) {
        self.set("Access-Control-Allow-Origin", origin);
        self.set("Access-Control-Allow-Methods", methods);
        self.set("Access-Control-Allow-Headers", headers);
        self.set("Access-Control-Max-Age", "86400");
    }
}
