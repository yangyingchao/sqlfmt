mod patterns;
mod special_clauses;
mod text_type;
mod keywords;
mod splitter;

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
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            
            // 1. Track TEXT types before parsing
            let (clean_stmt, text_count) = text_type::track_text_types(stmt);
            
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
            
            // 7. Apply keyword normalization
            let mut formatted = keywords::normalize_keywords(&formatted, cfg.case_mode);
            
            // 8. Add semicolon if missing
            if !formatted.trim().ends_with(';') {
                formatted.push(';');
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
