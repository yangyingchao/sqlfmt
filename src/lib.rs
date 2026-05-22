// sqlfmt - SQL formatter with intelligent WITH clause compression
// and Greenplum dialect support

pub mod config;
pub mod errors;
pub mod formatter;

pub use config::FormatterConfig;
pub use errors::SqlFmtError;
pub use formatter::{format_json, format_sql};

/// Version information (sourced from Cargo.toml at compile time)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
