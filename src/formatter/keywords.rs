use crate::config::CaseMode;
use regex::Regex;

/// Normalize keywords to the specified case mode
pub fn normalize_keywords(sql: &str, case_mode: CaseMode) -> String {
    match case_mode {
        CaseMode::Upper => to_upper(sql),
        CaseMode::Lower => to_lower(sql),
        CaseMode::Title => to_title(sql),
        CaseMode::Spongebob => to_spongebob(sql),
    }
}

fn to_upper(sql: &str) -> String {
    // Normalize keywords to uppercase
    let mut result = sql.to_string();

    // Normalize special keywords
    result = normalize_distributed_by(&result, "DISTRIBUTED BY");
    result = normalize_partition_by(&result, "PARTITION BY");
    result = normalize_with_keyword(&result, "WITH");

    // Also normalize parameter names inside WITH clauses
    result = normalize_with_parameters(&result);

    result
}

fn to_lower(sql: &str) -> String {
    // sqlparser should handle this, we just need to handle our special keywords
    let mut result = sql.to_string();

    result = normalize_distributed_by(&result, "distributed by");
    result = normalize_partition_by(&result, "partition by");
    result = normalize_with_keyword(&result, "with");

    result
}

fn to_title(sql: &str) -> String {
    // Convert main keywords to Title Case
    let mut result = sql.to_string();

    result = normalize_distributed_by(&result, "Distributed By");
    result = normalize_partition_by(&result, "Partition By");
    result = normalize_with_keyword(&result, "With");

    result
}

fn to_spongebob(sql: &str) -> String {
    // sPoNgEbOb CaSe - alternating case
    let mut result = sql.to_string();

    result = normalize_distributed_by(&result, "DiStRiBuTeD bY");
    result = normalize_partition_by(&result, "PaRtItIoN bY");
    result = normalize_with_keyword(&result, "WiTh");

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
        let result = to_upper(sql);
        assert!(result.contains("DISTRIBUTED BY"));
    }

    #[test]
    fn test_normalize_keywords_with_case_mode() {
        let sql = "with ( key = value ) distributed by (a)";
        let result = normalize_keywords(sql, CaseMode::Upper);
        assert!(result.contains("WITH") || result.contains("DISTRIBUTED BY"));
    }
}
