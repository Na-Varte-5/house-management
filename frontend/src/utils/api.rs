// Utility to construct full backend API URLs.
// In dev, frontend runs on 8081 (Trunk) and backend on 8080. We rewrite the port accordingly.
// In production (same origin), it will keep the existing host.
pub fn api_url(path: &str) -> String {
    let base = if let Some(window) = web_sys::window() {
        let location = window.location();
        let protocol = location
            .protocol()
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "http:".into());
        let host = location
            .host()
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "127.0.0.1:8081".into());
        let adjusted_host = if host.ends_with(":8081") {
            host.replace(":8081", ":8080")
        } else {
            host
        };
        format!("{}//{}", protocol, adjusted_host)
    } else {
        "http://127.0.0.1:8080".into()
    };
    if path.starts_with('/') {
        format!("{}{}", base, path)
    } else {
        format!("{}/{}", base, path)
    }
}
