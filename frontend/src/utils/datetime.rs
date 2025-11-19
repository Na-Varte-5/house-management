use crate::i18n::current_language;

/// Format a backend datetime string into localized human-friendly form.
/// Accepts formats like "YYYY-MM-DD HH:MM:SS" or ISO "YYYY-MM-DDTHH:MM:SS".
pub fn format_dt_local(raw: &str) -> String {
    if raw.is_empty() { return String::new(); }
    // Normalize separators
    let mut base = raw.trim().to_string();
    if let Some(tpos) = base.find('T') { // replace T with space
        // take date + time portion up to seconds
        let slice = &base[..std::cmp::min(base.len(), tpos + 9)]; // may not be right length but we'll adjust below
        base = raw.replace('T', " ");
    }
    // Extract date and time (assume first 19 chars contain YYYY-MM-DD HH:MM:SS)
    // Fallback to original if unexpected length
    let cleaned = base.replace('Z', "");
    let core = if cleaned.len() >= 19 { &cleaned[..19] } else { cleaned.as_str() };
    // Split
    let (date_part, time_part) = match core.split_once(' ') {
        Some((d,t)) => (d,t),
        None => return raw.to_string(),
    };
    // time HH:MM:SS -> HH:MM
    let time_short = &time_part[..std::cmp::min(time_part.len(),5)];
    let lang = current_language();
    if lang.starts_with("cs") {
        // YYYY-MM-DD -> DD.MM.YYYY
        if date_part.len() == 10 {
            let y = &date_part[0..4];
            let m = &date_part[5..7];
            let d = &date_part[8..10];
            format!("{}.{}.{} {}", d, m, y, time_short)
        } else { raw.to_string() }
    } else {
        // EN default keep YYYY-MM-DD HH:MM
        format!("{} {}", date_part, time_short)
    }
}
