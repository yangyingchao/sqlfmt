// sqlfmt - SQL formatter with intelligent WITH clause compression
// and Greenplum dialect support

pub mod formatter;
pub mod config;
pub mod errors;

pub use formatter::{format_sql, format_json};
pub use config::FormatterConfig;
pub use errors::SqlFmtError;

/// Version information
pub const VERSION: &str = "0.5.4";
