use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Pattern to match SQL comments (-- comment style)
    static ref COMMENT_PATTERN: Regex = Regex::new(r"^(\s*)--(.*)$").unwrap();
}

/// Comment with its position in the original text
#[derive(Debug, Clone)]
pub struct Comment {
    pub content: String,
    pub is_line_comment: bool,
    pub position: usize, // approximate position in original SQL
}

/// Extract comments from SQL text while preserving their structure
pub fn extract_comments(sql: &str) -> (String, Vec<(usize, Comment)>) {
    let mut clean_sql = String::new();
    let mut comments = Vec::new();
    let mut char_position = 0;

    for line in sql.lines() {
        if let Some(caps) = COMMENT_PATTERN.captures(line) {
            let leading_space = &caps[1];
            let comment_text = &caps[2];
            
            comments.push((
                char_position,
                Comment {
                    content: format!("--{}", comment_text),
                    is_line_comment: true,
                    position: char_position,
                },
            ));
            
            // Keep leading whitespace and newline structure
            clean_sql.push_str(leading_space);
            clean_sql.push('\n');
        } else {
            clean_sql.push_str(line);
            clean_sql.push('\n');
        }
        
        char_position += line.len() + 1; // +1 for newline
    }

    (clean_sql, comments)
}

/// Restore comments to formatted SQL
pub fn restore_comments(formatted: &str, comments: &[(usize, Comment)]) -> String {
    if comments.is_empty() {
        return formatted.to_string();
    }

    // For now, return formatted as-is
    // Full comment restoration requires line-by-line processing
    // This is a simplified approach
    formatted.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_comments() {
        let sql = "-- comment\nSELECT * FROM t;";
        let (clean, comments) = extract_comments(sql);
        assert!(comments.len() >= 1);
        assert!(!clean.contains("--"));
    }

    #[test]
    fn test_multiple_comments() {
        let sql = "-- first\nSELECT 1;\n-- second\nSELECT 2;";
        let (_clean, comments) = extract_comments(sql);
        assert_eq!(comments.len(), 2);
    }
}
