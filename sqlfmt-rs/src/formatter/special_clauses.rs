use super::patterns::{WITH_CLAUSE, DISTRIBUTED_BY, PARTITION_BY, CREATE_TABLE};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SpecialClauses {
    pub with_compressed: String,
    pub with_original: String,
    pub distributed_by: String,
    pub partition_by: String,
}

/// Extract WITH, DISTRIBUTED BY, and PARTITION BY clauses from CREATE TABLE statements
pub fn extract_all_clauses(sql: &str) -> (String, Vec<SpecialClauses>) {
    let mut result = String::new();
    let mut all_clauses = Vec::new();
    let mut last_end = 0;

    // Find all CREATE TABLE statements and extract their clauses
    for mat in CREATE_TABLE.find_iter(sql) {
        let statement = mat.as_str();
        let start = mat.start();
        let end = mat.end();
        
        // Add content before this CREATE TABLE
        result.push_str(&sql[last_end..start]);
        
        // Extract WITH clause
        let (with_compressed, with_original) = extract_with_clause(statement);
        
        // Extract DISTRIBUTED BY clause
        let distributed_by = extract_distributed_by(statement);
        
        // Extract PARTITION BY clause
        let partition_by = extract_partition_by(statement);
        
        // Create cleaned version of statement without special clauses
        let mut cleaned = statement.to_string();
        cleaned = WITH_CLAUSE.replace_all(&cleaned, "").to_string();
        cleaned = DISTRIBUTED_BY.replace_all(&cleaned, "").to_string();
        cleaned = PARTITION_BY.replace_all(&cleaned, "").to_string();
        
        // Add cleaned statement to result
        result.push_str(&cleaned);
        
        all_clauses.push(SpecialClauses {
            with_compressed,
            with_original,
            distributed_by,
            partition_by,
        });
        
        last_end = end;
    }
    
    // Add remaining content
    result.push_str(&sql[last_end..]);
    
    (result, all_clauses)
}

fn extract_with_clause(statement: &str) -> (String, String) {
    match WITH_CLAUSE.find(statement) {
        Some(mat) => {
            let with_clause = mat.as_str().trim().to_string();
            
            // Compressed version: collapse whitespace
            let with_compressed = regex::Regex::new(r"\s+")
                .unwrap()
                .replace_all(&with_clause, " ")
                .trim()
                .to_string();
            
            (with_compressed, with_clause)
        }
        None => (String::new(), String::new()),
    }
}

fn extract_distributed_by(statement: &str) -> String {
    match DISTRIBUTED_BY.find(statement) {
        Some(mat) => mat.as_str().trim().to_string(),
        None => String::new(),
    }
}

fn extract_partition_by(statement: &str) -> String {
    match PARTITION_BY.find(statement) {
        Some(mat) => mat.as_str().trim().to_string(),
        None => String::new(),
    }
}

/// Restore special clauses to CREATE TABLE statements
pub fn restore_all_clauses(
    formatted: &str,
    all_clauses: &[SpecialClauses],
    line_width: usize,
) -> String {
    if all_clauses.is_empty() {
        return formatted.to_string();
    }

    let mut result = formatted.to_string();
    let mut clause_idx = 0;

    // Find all CREATE TABLE statements and restore the corresponding clauses
    let create_table_pattern = Regex::new(r"(?i)(CREATE\s+TABLE\s+\w+\s*\([^)]*\))").unwrap();
    
    result = create_table_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            if clause_idx >= all_clauses.len() {
                return caps[0].to_string();
            }
            
            let clauses = &all_clauses[clause_idx];
            clause_idx += 1;
            
            let mut statement = caps[0].to_string();
            
            // Decide which version of WITH clause to use
            let with_clause = if !clauses.with_compressed.is_empty() 
                && clauses.with_compressed.len() <= line_width {
                // Use compressed version if it fits
                clauses.with_compressed.clone()
            } else if !clauses.with_original.is_empty() {
                // Use original multi-line version
                clauses.with_original.clone()
            } else {
                String::new()
            };
            
            // Add WITH clause if present
            if !with_clause.is_empty() {
                statement.push('\n');
                statement.push_str(&with_clause);
            }
            
            // Add DISTRIBUTED BY clause if present
            if !clauses.distributed_by.is_empty() {
                statement.push('\n');
                statement.push_str(&clauses.distributed_by);
            }
            
            // Add PARTITION BY clause if present
            if !clauses.partition_by.is_empty() {
                statement.push('\n');
                statement.push_str(&clauses.partition_by);
            }
            
            statement
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_with_clause() {
        let sql = "WITH (key = value)";
        let (compressed, original) = extract_with_clause(sql);
        assert!(!compressed.is_empty());
        assert!(!original.is_empty());
    }

    #[test]
    fn test_extract_distributed_by() {
        let sql = "DISTRIBUTED BY (a)";
        let result = extract_distributed_by(sql);
        assert_eq!(result.to_lowercase(), "distributed by (a)");
    }

    #[test]
    fn test_extract_partition_by() {
        let sql = "PARTITION BY list(a)";
        let result = extract_partition_by(sql);
        assert!(!result.is_empty());
    }
}
