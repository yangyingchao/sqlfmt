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
    let mut current_comments = Vec::new();
    let mut pending_comment = String::new();

    for line in sql.lines() {
        let trimmed = line.trim();
        
        // Handle comments - save them but don't add to current statement yet
        if trimmed.starts_with("--") {
            pending_comment.push_str(line);
            pending_comment.push('\n');
            continue;
        }
        
        // If we have pending comments and now have a non-comment line,
        // add the comments to the current statement
        if !pending_comment.is_empty() {
            if current.is_empty() {
                // Comments before the first statement line
                current.push_str(&pending_comment);
            } else {
                // Comments between statements - save for next statement
                current_comments.push(pending_comment.clone());
            }
            pending_comment.clear();
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
            let stmt = current.trim_end().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current.clear();
            current_comments.clear();
        } else {
            current.push('\n');
        }
    }
    
    // Add remaining content if any (including pending comments)
    if !pending_comment.is_empty() {
        current.push_str(&pending_comment);
    }
    
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
