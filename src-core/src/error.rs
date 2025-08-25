//! Error types for wezzte-core

use thiserror::Error;

/// Result type alias for wezzte operations
pub type Result<T> = std::result::Result<T, WezzteError>;

/// Main error type for wezzte operations
#[derive(Error, Debug, Clone)]
pub enum WezzteError {
    /// Parse errors - when decoration parsing fails
    #[error("Parse error at position {position}: {message}")]
    Parse { message: String, position: usize },

    /// IO errors - file operations
    #[error("IO error: {message}")]
    Io { message: String },

    /// Configuration errors
    #[error("Config error: {message}")]
    Config { message: String },

    /// Invalid decoration format
    #[error("Invalid decoration: {message}")]
    InvalidDecoration { message: String },

    /// Missing required parameters
    #[error("Missing required parameter: {param}")]
    MissingParameter { param: String },

    /// Invalid parameter value
    #[error("Invalid parameter value for '{param}': {message}")]
    InvalidParameter { param: String, message: String },

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Permission denied
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
}

impl WezzteError {
    /// Create a new parse error
    pub fn parse(message: impl Into<String>, position: usize) -> Self {
        Self::Parse {
            message: message.into(),
            position,
        }
    }

    /// Create a new IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Create a new config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new invalid decoration error
    pub fn invalid_decoration(message: impl Into<String>) -> Self {
        Self::InvalidDecoration {
            message: message.into(),
        }
    }

    /// Create a new missing parameter error
    pub fn missing_parameter(param: impl Into<String>) -> Self {
        Self::MissingParameter {
            param: param.into(),
        }
    }

    /// Create a new invalid parameter error
    pub fn invalid_parameter(param: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            param: param.into(),
            message: message.into(),
        }
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for WezzteError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound {
                path: "unknown".to_string(),
            },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                path: "unknown".to_string(),
            },
            _ => Self::io(err.to_string()),
        }
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for WezzteError {
    fn from(err: serde_json::Error) -> Self {
        Self::config(format!("JSON serialization error: {}", err))
    }
}