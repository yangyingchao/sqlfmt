mod keywords;
mod patterns;
mod pretty;
mod special_clauses;
mod splitter;
pub use splitter::split_statements;

use crate::config::FormatterConfig;
use crate::errors::Result;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Format SQL statements with the given configuration
pub fn format_sql(cfg: &FormatterConfig, statements: &[String]) -> Result<String> {
    let mut result = String::new();
    let dialect = PostgreSqlDialect {};

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

            let clean_stmt = stmt_without_comments;

            // 1. Extract special clauses (WITH, DISTRIBUTED BY, PARTITION BY)
            let (clean_stmt, clauses) = special_clauses::extract_all_clauses(&clean_stmt);

            // 2. Parse SQL
            let mut parser = Parser::new(&dialect).try_with_sql(&clean_stmt)?;
            let parsed = parser.parse_statements()?;

            // 3. Format using sqlparser
            let formatted = parsed
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(";");

            // 4. Restore special clauses
            let formatted =
                special_clauses::restore_all_clauses(&formatted, &clauses, cfg.print_width);

            // 5. Apply keyword normalization (before width formatting)
            let formatted = keywords::normalize_keywords(&formatted);

            // 6. Apply width-aware formatting
            let formatted = pretty::apply_width(&formatted, cfg);

            // 7. Add semicolon if missing (always on same line)
            let mut formatted = formatted.trim().to_string();
            if !formatted.ends_with(';') {
                formatted.push(';');
            }

            // 8. Add leading comments back
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
