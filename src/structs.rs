use std::path::PathBuf;

pub(crate) struct Opts {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) copy_bool: bool,
    pub(crate) move_bool: bool,
    pub(crate) force_run: bool,
    pub(crate) log_bool: bool,
    pub(crate) source_pattern: String,
    pub(crate) dest_pattern: String,
}

pub(crate) struct OperationStatus {
    pub(crate) files: Vec<(&'static str, &'static str)>,
    pub(crate) status: Vec<(String, String)>,
}

impl OperationStatus {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            files: Vec::with_capacity(size + 1),
            status: Vec::with_capacity(size + 1),
        }
    }
}

pub(crate) enum Color {
    Red,
    Green,
    Blue,
    Default,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color_code = match self {
            Color::Red => 31,
            Color::Green => 32,
            Color::Blue => 34,
            Color::Default => 0,
        };
        write!(f, "\x1b[{}m", color_code)
    }
}

impl Color {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Blue => "\x1b[34m",
            Color::Default => "\x1b[0m",
        }
    }
}
