use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Pattern to match DISTRIBUTED BY clause
    pub static ref DISTRIBUTED_BY: Regex =
        Regex::new(r"(?i)(?:^|\s+)distributed\s+by\s*\([^)]*\)").unwrap();

    /// Pattern to match PARTITION BY clause
    pub static ref PARTITION_BY: Regex =
        Regex::new(r"(?i)(?:^|\s+)partition\s+by\s+\w+\s*\([^)]*\)").unwrap();

    /// Pattern to match WITH clause
    pub static ref WITH_CLAUSE: Regex =
        Regex::new(r"(?i)(?:^|\s+)with\s*\([^)]*\)").unwrap();

    /// Pattern to match TEXT type
    pub static ref TEXT_TYPE: Regex =
        Regex::new(r"(?i)\bTEXT\b").unwrap();

    /// Pattern to match CREATE TABLE statements
    pub static ref CREATE_TABLE: Regex =
        Regex::new(r"(?i)CREATE\s+TABLE\s+[^;]+?;").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributed_by_pattern() {
        let sql = "DISTRIBUTED BY (a)";
        assert!(DISTRIBUTED_BY.is_match(sql));
    }

    #[test]
    fn test_partition_by_pattern() {
        let sql = "PARTITION BY list(a)";
        assert!(PARTITION_BY.is_match(sql));
    }

    #[test]
    fn test_with_clause_pattern() {
        let sql = "WITH (key = value)";
        assert!(WITH_CLAUSE.is_match(sql));
    }

    #[test]
    fn test_text_type_pattern() {
        let sql = "col TEXT";
        assert!(TEXT_TYPE.is_match(sql));
    }
}
