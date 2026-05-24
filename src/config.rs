/// SQL formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Maximum line width for pretty printing (default: 80)
    pub print_width: usize,

    /// Use spaces for indentation instead of tabs (default: false, uses tabs)
    pub use_spaces: bool,

    /// Indent width in spaces (also used as tab width in char counting)
    pub indent_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            print_width: 80,
            use_spaces: true, // 4 spaces by default
            indent_width: 4,
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
