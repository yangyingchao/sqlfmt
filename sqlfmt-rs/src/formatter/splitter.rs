use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Pattern to match comments
    static ref COMMENT_PATTERN: Regex = Regex::new(r"^--.*\s*").unwrap();
}

/// Split SQL input into statements while preserving comments
pub fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut lines = sql.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Handle comments
        if trimmed.starts_with("--") {
            current.push_str(line);
            current.push('\n');
            continue;
        }
        
        // Handle empty lines
        if trimmed.is_empty() {
            if !current.is_empty() && !current.trim().is_empty() {
                current.push('\n');
            }
            continue;
        }
        
        current.push_str(line);
        
        // Check if statement ends with semicolon
        if trimmed.ends_with(';') {
            let stmt = current.trim().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current.clear();
        } else {
            current.push('\n');
        }
    }
    
    // Add remaining content if any
    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        statements.push(remaining);
    }
    
    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_single_statement() {
        let sql = "SELECT * FROM t;";
        let result = split_statements(sql);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_split_multiple_statements() {
        let sql = "SELECT * FROM t1; SELECT * FROM t2;";
        let result = split_statements(sql);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_split_with_comments() {
        let sql = "-- comment\nSELECT * FROM t;";
        let result = split_statements(sql);
        assert_eq!(result.len(), 1);
    }
}
