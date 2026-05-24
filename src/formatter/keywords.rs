use regex::Regex;

/// Normalize Greenplum-specific keywords to uppercase
pub fn normalize_keywords(sql: &str) -> String {
    let mut result = sql.to_string();
    result = normalize_distributed_by(&result, "DISTRIBUTED BY");
    result = normalize_partition_by(&result, "PARTITION BY");
    result = normalize_with_keyword(&result, "WITH");
    result = normalize_with_parameters(&result);
    result
}

fn normalize_distributed_by(sql: &str, replacement: &str) -> String {
    let pattern = Regex::new(r"(?i)DISTRIBUTED\s+BY").unwrap();
    pattern.replace_all(sql, replacement).to_string()
}

fn normalize_partition_by(sql: &str, replacement: &str) -> String {
    let pattern = Regex::new(r"(?i)PARTITION\s+BY").unwrap();
    pattern.replace_all(sql, replacement).to_string()
}

fn normalize_with_keyword(sql: &str, replacement: &str) -> String {
    let pattern = Regex::new(r"(?i)\bWITH\s*\(").unwrap();
    pattern
        .replace_all(sql, &format!("{} (", replacement))
        .to_string()
}

fn normalize_with_parameters(sql: &str) -> String {
    // Normalize parameter names inside WITH clauses only
    // Pattern: WITH ( ... ) - only process content within parentheses after WITH
    let with_pattern = Regex::new(r"(?i)WITH\s*\(([^)]*)\)").unwrap();

    with_pattern
        .replace_all(sql, |caps: &regex::Captures| {
            let with_content = &caps[1];
            let param_pattern = Regex::new(r"\b([a-zA-Z_]\w*)\s*=").unwrap();

            let normalized = param_pattern
                .replace_all(with_content, |inner_caps: &regex::Captures| {
                    format!("{} =", inner_caps[1].to_uppercase())
                })
                .to_string();

            format!("WITH ({})", normalized)
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_to_upper() {
        let sql = "distributed by (a)";
        let result = normalize_keywords(sql);
        assert!(result.contains("DISTRIBUTED BY"));
    }

    #[test]
    fn test_normalize_keywords() {
        let sql = "with ( key = value ) distributed by (a)";
        let result = normalize_keywords(sql);
        assert!(result.contains("WITH") || result.contains("DISTRIBUTED BY"));
    }
}
