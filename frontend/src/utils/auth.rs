use web_sys::Window;

pub fn get_token() -> Option<String> {
    let window: Window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("jwt").ok().flatten()
}

pub fn set_token(tok: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("jwt", tok);
        }
    }
}

pub fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("jwt");
        }
    }
}
