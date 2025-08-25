//! Wezzte Core - Parser and utilities for WezTerm configuration decorations
//!
//! This library provides the core functionality for parsing, processing, and generating
//! WezTerm configuration files with UI decorator annotations.

pub mod error;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod config;
pub mod widgets;
pub mod layout;

pub use error::{WezzteError, Result};
pub use lexer::{Token, TokenType, Lexer};
pub use parser::{Parser, ConfigEntry};
pub use ast::{Annotation, UiType, ParamValue};
pub use config::{ConfigManager, BackupManager};

/// Version of the wezzte-core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Magic markers for tuner sections in configuration files
pub mod markers {
    pub const TUNER_START: &str = "-- <<TUNER-START>>";
    pub const TUNER_END: &str = "-- <<TUNER-END>>";
    pub const DECORATOR_PREFIX: &str = "-- @ui:";
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_markers() {
        assert_eq!(markers::TUNER_START, "-- <<TUNER-START>>");
        assert_eq!(markers::TUNER_END, "-- <<TUNER-END>>");
        assert_eq!(markers::DECORATOR_PREFIX, "-- @ui:");
    }
}