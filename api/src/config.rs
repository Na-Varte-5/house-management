use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub attachments_base_path: String,
    pub max_attachment_size_bytes: u64,
    pub allowed_mime_types: Vec<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        let attachments_base_path =
            env::var("ATTACHMENTS_BASE_PATH").unwrap_or_else(|_| "attachments".into());
        let max_attachment_size_bytes = env::var("MAX_ATTACHMENT_SIZE_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10 * 1024 * 1024); // 10MB default
        let allowed_mime_types = env::var("ALLOWED_ATTACHMENT_MIME_TYPES")
            .unwrap_or_else(|_| "image/jpeg,image/png,application/pdf".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        AppConfig {
            attachments_base_path,
            max_attachment_size_bytes,
            allowed_mime_types,
        }
    }
}
