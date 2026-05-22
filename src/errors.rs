use std::fmt;

/// Errors that can occur during SQL formatting
#[derive(Debug)]
pub enum SqlFmtError {
    /// SQL parsing error
    ParseError(String),

    /// Invalid configuration
    InvalidConfig(String),

    /// IO error
    IoError(String),

    /// Other errors
    Other(String),
}

impl fmt::Display for SqlFmtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlFmtError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SqlFmtError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            SqlFmtError::IoError(msg) => write!(f, "IO error: {}", msg),
            SqlFmtError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for SqlFmtError {}

impl From<std::io::Error> for SqlFmtError {
    fn from(err: std::io::Error) -> Self {
        SqlFmtError::IoError(err.to_string())
    }
}

impl From<sqlparser::parser::ParserError> for SqlFmtError {
    fn from(err: sqlparser::parser::ParserError) -> Self {
        SqlFmtError::ParseError(err.to_string())
    }
}

/// Result type for sqlformat operations
pub type Result<T> = std::result::Result<T, SqlFmtError>;
