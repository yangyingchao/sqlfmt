mod patterns;
mod special_clauses;
mod text_type;
mod keywords;
mod splitter;
mod pretty;

pub use splitter::split_statements;

use crate::config::FormatterConfig;
use crate::errors::Result;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

/// Format SQL statements with the given configuration
pub fn format_sql(cfg: &FormatterConfig, statements: &[String]) -> Result<String> {
    let mut result = String::new();
    let dialect = GenericDialect {};
    
    for input in statements {
        // Split input into individual statements
        let stmts = split_statements(input);
        
        for stmt in stmts {
            let stmt_trimmed = stmt.trim();
            if stmt_trimmed.is_empty() {
                continue;
            }
            
            // Extract comments from the statement
            let (stmt_without_comments, leading_comments) = extract_leading_comments(&stmt);
            
            // 1. Track TEXT types before parsing
            let (clean_stmt, text_count) = text_type::track_text_types(&stmt_without_comments);
            
            // 2. Extract special clauses (WITH, DISTRIBUTED BY, PARTITION BY)
            let (clean_stmt, clauses) = special_clauses::extract_all_clauses(&clean_stmt);
            
            // 3. Parse SQL
            let mut parser = Parser::new(&dialect).try_with_sql(&clean_stmt)?;
            let parsed = parser.parse_statements()?;
            
            // 4. Format using sqlparser
            let formatted = parsed
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(";");
            
            // 5. Restore TEXT types
            let formatted = text_type::restore_text_types(&formatted, text_count);
            
            // 6. Restore special clauses
            let formatted = special_clauses::restore_all_clauses(&formatted, &clauses, cfg.print_width);
            
            // 7. Apply keyword normalization (before width formatting)
            let formatted = keywords::normalize_keywords(&formatted, cfg.case_mode);
            
            // 8. Apply width-aware formatting
            let formatted = pretty::apply_width(&formatted, cfg);
            
            // 9. Add semicolon if missing (always on same line)
            let mut formatted = formatted.trim().to_string();
            if !formatted.ends_with(';') {
                formatted.push(';');
            }
            
            // 9. Add leading comments back
            if !leading_comments.is_empty() {
                result.push_str(&leading_comments);
                result.push('\n');
            }
            
            result.push_str(&formatted);
            result.push('\n');
            result.push('\n');
        }
    }
    
    // Clean up extra newlines at the end
    while result.ends_with("\n\n") {
        result.pop();
    }
    
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    
    Ok(result)
}

/// Extract leading comments from a statement
fn extract_leading_comments(stmt: &str) -> (String, String) {
    let mut comments = String::new();
    let mut sql = String::new();
    let mut found_sql = false;

    for line in stmt.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            if !found_sql {
                comments.push_str(line);
                comments.push('\n');
            } else {
                // Comments after SQL - keep with SQL
                sql.push_str(line);
                sql.push('\n');
            }
        } else if !trimmed.is_empty() {
            found_sql = true;
            sql.push_str(line);
            sql.push('\n');
        } else if found_sql {
            // Empty line after SQL started
            sql.push_str(line);
            sql.push('\n');
        }
    }

    (sql.trim_end().to_string(), comments.trim_end().to_string())
}

/// Format JSON (pass-through for now, will implement if needed)
pub fn format_json(json_str: &str) -> Result<String> {
    // Try to parse and pretty-print JSON
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(value) => {
            let formatted = serde_json::to_string_pretty(&value)?;
            Ok(formatted)
        }
        Err(e) => Err(crate::errors::SqlFmtError::Other(format!("Invalid JSON: {}", e))),
    }
}
