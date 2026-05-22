/// Keyword case mode for SQL formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    /// UPPER CASE KEYWORDS
    Upper,
    /// lower case keywords
    Lower,
    /// Title Case Keywords
    Title,
    /// sPoNgEbOb CaSe KeYwOrDs (easter egg)
    Spongebob,
}

impl CaseMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "upper" => Some(CaseMode::Upper),
            "lower" => Some(CaseMode::Lower),
            "title" => Some(CaseMode::Title),
            "spongebob" => Some(CaseMode::Spongebob),
            _ => None,
        }
    }
}

/// SQL formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Maximum line width for pretty printing (default: 80)
    pub print_width: usize,

    /// Use spaces for indentation instead of tabs (default: false, uses tabs)
    pub use_spaces: bool,

    /// Indent width in spaces (also used as tab width in char counting)
    pub indent_width: usize,

    /// Keyword case mode (default: Upper)
    pub case_mode: CaseMode,

    /// Whether to simplify query structure (default: true)
    pub simplify: bool,

    /// Whether to align keywords (default: false)
    pub align: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            print_width: 80,
            use_spaces: true, // 4 spaces by default
            indent_width: 4,
            case_mode: CaseMode::Upper,
            simplify: true,
            align: false,
        }
    }
}

impl FormatterConfig {
    /// Create a new formatter config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the indentation string based on configuration
    pub fn indent_str(&self) -> String {
        if self.use_spaces {
            " ".repeat(self.indent_width)
        } else {
            "\t".to_string()
        }
    }
}
