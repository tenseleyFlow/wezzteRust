//! Recursive descent parser for wezzte decoration annotations
//!
//! Converts tokens from the lexer into structured AST nodes. Based on the
//! Python "Wexler" parser but optimized for Rust ownership patterns.

use crate::{
    ast::{Annotation, ParamValue, UiType},
    error::{WezzteError, Result},
    lexer::{Token, TokenType},
    markers::{DECORATOR_PREFIX, TUNER_END, TUNER_START},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A parsed configuration entry with its decoration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// Configuration key (e.g., "config.font_size")
    pub key: String,
    /// Current value as string
    pub value: String,
    /// Parsed annotation from the decorator
    pub annotation: Annotation,
    /// Original decorator line for reference
    pub decorator: String,
}

impl ConfigEntry {
    pub fn new(key: String, value: String, annotation: Annotation, decorator: String) -> Self {
        Self {
            key,
            value,
            annotation,
            decorator,
        }
    }
}

/// Recursive descent parser for decorator annotations
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser with tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse a complete annotation from tokens
    pub fn parse_annotation(&mut self) -> Result<Annotation> {
        // Parse UI type first
        let ui_type = self.parse_ui_type()?;
        let mut params = HashMap::new();

        // Check for parenthesized parameters
        if self.peek_token_type(TokenType::LParen) {
            self.consume_token(TokenType::LParen)?;
            params = self.parse_param_list()?;
            self.consume_token(TokenType::RParen)?;
        }

        // Parse trailing parameters (outside parentheses)
        while self.current_token().token_type == TokenType::Identifier {
            // Check if this is actually a parameter (has '=' after)
            let saved_pos = self.position;
            if let Ok((key, value)) = self.try_parse_parameter() {
                params.insert(key, value);
                
                // Optional trailing comma
                if self.peek_token_type(TokenType::Comma) {
                    self.consume_token(TokenType::Comma)?;
                }
            } else {
                // Not a parameter, restore position and break
                self.position = saved_pos;
                break;
            }
        }

        // Should be at EOF now
        if self.current_token().token_type != TokenType::Eof {
            return Err(WezzteError::parse(
                format!("Unexpected token: {}", self.current_token().token_type),
                self.current_token().position,
            ));
        }

        let annotation = Annotation::new(ui_type, params);
        annotation.validate()?;
        Ok(annotation)
    }

    /// Parse UI type identifier
    fn parse_ui_type(&mut self) -> Result<UiType> {
        let token = self.consume_token(TokenType::Identifier)?;
        Ok(UiType::from(token.value.as_str()))
    }

    /// Parse a comma-separated list of parameters
    fn parse_param_list(&mut self) -> Result<HashMap<String, ParamValue>> {
        let mut params = HashMap::new();

        // Empty parameter list is valid
        if self.peek_token_type(TokenType::RParen) {
            return Ok(params);
        }

        // Parse first parameter
        let (key, value) = self.parse_parameter()?;
        params.insert(key, value);

        // Parse remaining parameters
        while self.peek_token_type(TokenType::Comma) {
            self.consume_token(TokenType::Comma)?;

            // Allow trailing commas
            if self.peek_token_type(TokenType::RParen) {
                break;
            }

            let (key, value) = self.parse_parameter()?;
            params.insert(key, value);
        }

        Ok(params)
    }

    /// Parse a single parameter: identifier = value
    fn parse_parameter(&mut self) -> Result<(String, ParamValue)> {
        let key_token = self.consume_token(TokenType::Identifier)?;
        let key = key_token.value.clone();
        self.consume_token(TokenType::Equal)?;
        let value = self.parse_value()?;
        Ok((key, value))
    }

    /// Try to parse a parameter, returning error if not valid
    fn try_parse_parameter(&mut self) -> Result<(String, ParamValue)> {
        let key_token = self.consume_token(TokenType::Identifier)?;
        let key = key_token.value.clone();
        self.consume_token(TokenType::Equal)?;
        let value = self.parse_value()?;
        Ok((key, value))
    }

    /// Parse any value type (number, string, boolean, list, object)
    fn parse_value(&mut self) -> Result<ParamValue> {
        let token = self.current_token();

        match &token.token_type {
            TokenType::Number => {
                let token = self.consume_token(TokenType::Number)?;
                let value = token.value.parse::<f64>().map_err(|e| {
                    WezzteError::parse(format!("Invalid number: {}", e), token.position)
                })?;
                Ok(ParamValue::Number(value))
            }
            TokenType::String => {
                let token = self.consume_token(TokenType::String)?;
                Ok(ParamValue::String(token.value.clone()))
            }
            TokenType::Boolean => {
                let token = self.consume_token(TokenType::Boolean)?;
                let value = token.value == "true";
                Ok(ParamValue::Boolean(value))
            }
            TokenType::Identifier => {
                let token = self.consume_token(TokenType::Identifier)?;
                Ok(ParamValue::String(token.value.clone()))
            }
            TokenType::LBracket => {
                self.parse_list()
            }
            TokenType::LBrace => {
                self.parse_object()
            }
            _ => Err(WezzteError::parse(
                format!("Expected value, got {}", token.token_type),
                token.position,
            )),
        }
    }

    /// Parse a list: [value, value, ...]
    fn parse_list(&mut self) -> Result<ParamValue> {
        self.consume_token(TokenType::LBracket)?;
        let mut values = Vec::new();

        // Empty list
        if self.peek_token_type(TokenType::RBracket) {
            self.consume_token(TokenType::RBracket)?;
            return Ok(ParamValue::List(values));
        }

        // Parse first value
        values.push(self.parse_value()?);

        // Parse remaining values
        while self.peek_token_type(TokenType::Comma) {
            self.consume_token(TokenType::Comma)?;

            // Allow trailing comma
            if self.peek_token_type(TokenType::RBracket) {
                break;
            }

            values.push(self.parse_value()?);
        }

        self.consume_token(TokenType::RBracket)?;
        Ok(ParamValue::List(values))
    }

    /// Parse an object: {key: value, key: value, ...}
    fn parse_object(&mut self) -> Result<ParamValue> {
        self.consume_token(TokenType::LBrace)?;
        let mut object = HashMap::new();

        // Empty object
        if self.peek_token_type(TokenType::RBrace) {
            self.consume_token(TokenType::RBrace)?;
            return Ok(ParamValue::Object(object));
        }

        // Parse first pair
        let key = self.consume_token(TokenType::Identifier)?.value.clone();
        self.consume_token(TokenType::Colon)?;
        let value = self.parse_value()?;
        object.insert(key, value);

        // Parse remaining pairs
        while self.peek_token_type(TokenType::Comma) {
            self.consume_token(TokenType::Comma)?;

            // Allow trailing comma
            if self.peek_token_type(TokenType::RBrace) {
                break;
            }

            let key = self.consume_token(TokenType::Identifier)?.value.clone();
            self.consume_token(TokenType::Colon)?;
            let value = self.parse_value()?;
            object.insert(key, value);
        }

        self.consume_token(TokenType::RBrace)?;
        Ok(ParamValue::Object(object))
    }

    /// Get current token
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or_else(|| {
            // Return EOF token if past end
            self.tokens.last().unwrap()
        })
    }

    /// Check if current token matches type without consuming
    fn peek_token_type(&self, expected: TokenType) -> bool {
        self.current_token().token_type == expected
    }

    /// Consume a token of expected type
    fn consume_token(&mut self, expected: TokenType) -> Result<&Token> {
        let token = self.current_token();

        if token.token_type != expected {
            return Err(WezzteError::parse(
                format!("Expected {}, got {}", expected, token.token_type),
                token.position,
            ));
        }

        let result = &self.tokens[self.position];
        self.position += 1;
        Ok(result)
    }
}

/// Parse a single decorator line into annotation
pub fn parse_decorator_line(line: &str) -> Result<Annotation> {
    if !line.starts_with(DECORATOR_PREFIX) {
        return Err(WezzteError::invalid_decoration(
            format!("Decorator must start with '{}'", DECORATOR_PREFIX)
        ));
    }

    let annotation_text = line[DECORATOR_PREFIX.len()..].trim();
    if annotation_text.is_empty() {
        return Err(WezzteError::invalid_decoration("Empty decorator annotation"));
    }

    // Tokenize and parse
    let mut lexer = crate::lexer::Lexer::new(annotation_text);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_annotation()
}

/// Parse all annotations from configuration content
pub fn parse_annotations(content: &str) -> Result<Vec<ConfigEntry>> {
    let start_pos = content.find(TUNER_START).ok_or_else(|| {
        WezzteError::config("No tuner start marker found in configuration")
    })?;

    let end_pos = content.find(TUNER_END).ok_or_else(|| {
        WezzteError::config("No tuner end marker found in configuration")
    })?;

    if end_pos <= start_pos {
        return Err(WezzteError::config("Invalid tuner marker positions"));
    }

    // Extract tuner block
    let tuner_start = start_pos + TUNER_START.len();
    let tuner_block = &content[tuner_start..end_pos];
    let lines: Vec<&str> = tuner_block.lines().collect();

    let mut entries = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with(DECORATOR_PREFIX) {
            let decorator_line = line.to_string();

            // Skip empty lines after decorator
            i += 1;
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Skip table initialization lines
            while i < lines.len() {
                let config_line = lines[i].trim();
                // Pattern: config.something = config.something or {}
                if config_line.contains(" = config.") && config_line.ends_with(" or {}") {
                    i += 1;
                    continue;
                } else {
                    break;
                }
            }

            // Next line should be the actual config assignment
            if i < lines.len() {
                let config_line = lines[i].trim();

                // Parse config assignment: config.key = value
                if let Some(eq_pos) = config_line.find(" = ") {
                    let key = config_line[..eq_pos].trim().to_string();
                    let value = config_line[eq_pos + 3..].trim().to_string();

                    // Parse the decorator
                    match parse_decorator_line(&decorator_line) {
                        Ok(annotation) => {
                            entries.push(ConfigEntry::new(key, value, annotation, decorator_line));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse decorator '{}': {}", decorator_line, e);
                        }
                    }
                }
            }
        }

        i += 1;
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_annotation_str(input: &str) -> Result<Annotation> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_annotation()
    }

    #[test]
    fn test_simple_slider() {
        let annotation = parse_annotation_str("slider(min=0, max=100, step=1)").unwrap();
        assert_eq!(annotation.ui_type, UiType::Slider);
        assert_eq!(annotation.get_param_as_number("min", -1.0), 0.0);
        assert_eq!(annotation.get_param_as_number("max", -1.0), 100.0);
        assert_eq!(annotation.get_param_as_number("step", -1.0), 1.0);
    }

    #[test]
    fn test_trailing_params() {
        let annotation = parse_annotation_str("slider(min=0, max=100) type=int").unwrap();
        assert_eq!(annotation.ui_type, UiType::Slider);
        assert_eq!(annotation.get_param_as_string("type", ""), "int");
    }

    #[test]
    fn test_select_with_string() {
        let annotation = parse_annotation_str(r#"select(options="Dark, Light, Auto") type=string"#).unwrap();
        assert_eq!(annotation.ui_type, UiType::Select);
        assert_eq!(annotation.get_param_as_string("options", ""), "Dark, Light, Auto");
        assert_eq!(annotation.get_param_as_string("type", ""), "string");
    }

    #[test]
    fn test_boolean_params() {
        let annotation = parse_annotation_str("color_picker(alpha=true, format=hex)").unwrap();
        assert_eq!(annotation.ui_type, UiType::ColorPicker);
        assert_eq!(annotation.get_param_as_bool("alpha", false), true);
        assert_eq!(annotation.get_param_as_string("format", ""), "hex");
    }

    #[test]
    fn test_list_params() {
        let annotation = parse_annotation_str(r#"multi_select(options=["resize", "title", "close"])"#).unwrap();
        assert_eq!(annotation.ui_type, UiType::Custom("multi_select".to_string()));
        let options = annotation.get_param("options").unwrap().as_list().unwrap();
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].as_string().unwrap(), "resize");
    }

    #[test]
    fn test_decorator_line_parsing() {
        let line = "-- @ui: slider(min=10, max=42, step=1) type=int";
        let annotation = parse_decorator_line(line).unwrap();
        assert_eq!(annotation.ui_type, UiType::Slider);
        assert_eq!(annotation.get_param_as_number("min", -1.0), 10.0);
        assert_eq!(annotation.get_param_as_string("type", ""), "int");
    }

    #[test]
    fn test_full_config_parsing() {
        let config = r#"
-- <<TUNER-START>>
-- @ui: slider(min=10, max=42, step=1) type=int
config.font_size = 18
-- @ui: select(options="JetBrains Mono, Fira Code") type=string
config.font = wezterm.font("JetBrains Mono")
-- <<TUNER-END>>
"#;

        let entries = parse_annotations(config).unwrap();
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].key, "config.font_size");
        assert_eq!(entries[0].value, "18");
        assert_eq!(entries[0].annotation.ui_type, UiType::Slider);

        assert_eq!(entries[1].key, "config.font");
        assert_eq!(entries[1].annotation.ui_type, UiType::Select);
    }

    #[test]
    fn test_error_handling() {
        assert!(parse_annotation_str("invalid syntax here").is_err());
        assert!(parse_decorator_line("not a decorator").is_err());
        assert!(parse_annotations("no markers here").is_err());
    }
}