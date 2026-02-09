use crate::i18n::current_language;

fn month_name_en(month: &str) -> &str {
    match month {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        other => other,
    }
}

fn month_name_cs(month: &str) -> &str {
    match month {
        "01" => "led",
        "02" => "úno",
        "03" => "bře",
        "04" => "dub",
        "05" => "kvě",
        "06" => "čvn",
        "07" => "čvc",
        "08" => "srp",
        "09" => "zář",
        "10" => "říj",
        "11" => "lis",
        "12" => "pro",
        other => other,
    }
}

fn parse_parts(raw: &str) -> Option<(String, String, String, Option<String>)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace('T', " ").replace('Z', "");

    let (date_part, time_part) = match normalized.split_once(' ') {
        Some((d, t)) => (d.to_string(), Some(t.to_string())),
        None => (normalized.clone(), None),
    };

    let dp: Vec<&str> = date_part.split('-').collect();
    if dp.len() != 3 || dp[0].len() != 4 || dp[1].len() != 2 || dp[2].len() < 2 {
        return None;
    }
    let year = dp[0].to_string();
    let month = dp[1].to_string();
    let day = dp[2][..2].to_string();

    let time_short = time_part.map(|t| {
        let tp: Vec<&str> = t.split(':').collect();
        if tp.len() >= 2 {
            format!("{}:{}", tp[0], tp[1])
        } else {
            t
        }
    });

    Some((year, month, day, time_short))
}

/// Format a backend datetime string into a human-friendly localized form.
///
/// Examples:
/// - EN: "Jan 14, 2026 at 10:30"
/// - CS: "14. led 2026, 10:30"
///
/// Accepts "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DDTHH:MM:SS", "YYYY-MM-DDTHH:MM:SS.fffZ", etc.
/// Returns "N/A" for empty/unparseable input.
pub fn format_dt_local(raw: &str) -> String {
    let Some((year, month, day, time_short)) = parse_parts(raw) else {
        return if raw.trim().is_empty() {
            "N/A".to_string()
        } else {
            raw.to_string()
        };
    };

    let lang = current_language();

    if lang.starts_with("cs") {
        let m = month_name_cs(&month);
        match time_short {
            Some(t) => format!("{}. {} {}, {}", day, m, year, t),
            None => format!("{}. {} {}", day, m, year),
        }
    } else {
        let m = month_name_en(&month);
        match time_short {
            Some(t) => format!("{} {}, {} at {}", m, day, year, t),
            None => format!("{} {}, {}", m, day, year),
        }
    }
}

/// Format a backend datetime string showing only the date portion.
///
/// Examples:
/// - EN: "Jan 14, 2026"
/// - CS: "14. led 2026"
pub fn format_date_only(raw: &str) -> String {
    let Some((year, month, day, _)) = parse_parts(raw) else {
        return if raw.trim().is_empty() {
            "N/A".to_string()
        } else {
            raw.to_string()
        };
    };

    let lang = current_language();

    if lang.starts_with("cs") {
        format!("{}. {} {}", day, month_name_cs(&month), year)
    } else {
        format!("{} {}, {}", month_name_en(&month), day, year)
    }
}

/// Convenience wrapper for `Option<String>` datetime values.
/// Returns "N/A" for None or empty strings.
pub fn format_dt_option(raw: Option<&String>) -> String {
    match raw {
        Some(s) if !s.is_empty() => format_dt_local(s),
        _ => "N/A".to_string(),
    }
}
