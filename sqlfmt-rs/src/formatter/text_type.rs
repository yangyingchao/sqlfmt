use super::patterns::TEXT_TYPE;
use regex::Regex;

/// Track TEXT type occurrences before parsing
/// CockroachDB parser converts TEXT to STRING, so we need to restore it later
pub fn track_text_types(sql: &str) -> (String, usize) {
    let text_count = TEXT_TYPE.find_iter(sql).count();
    (sql.to_string(), text_count)
}

/// Restore TEXT type after formatting
/// Replace the first N STRING occurrences with TEXT (where N = number of original TEXT types)
pub fn restore_text_types(formatted: &str, text_count: usize) -> String {
    if text_count == 0 {
        return formatted.to_string();
    }

    let mut result = formatted.to_string();
    let mut replaced = 0;

    // Replace STRING with TEXT for the first text_count occurrences
    let string_pattern = Regex::new(r"(?i)\bSTRING\b").unwrap();
    
    result = string_pattern
        .replace_all(&result, |_: &regex::Captures| {
            if replaced < text_count {
                replaced += 1;
                "TEXT".to_string()
            } else {
                "STRING".to_string()
            }
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_text_types() {
        let sql = "CREATE TABLE t (a TEXT, b TEXT)";
        let (_clean, count) = track_text_types(sql);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_restore_text_types() {
        let formatted = "CREATE TABLE t (a STRING, b STRING)";
        let restored = restore_text_types(formatted, 2);
        assert!(restored.contains("TEXT"));
    }
}
