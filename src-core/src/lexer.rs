//! Lexer for wezzte decoration annotations
//!
//! Converts annotation text into tokens for parsing. Based on the Python implementation
//! but optimized for Rust patterns.

use crate::error::{WezzteError, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, take_while},
    character::complete::{char, digit1, multispace1},
    combinator::{map, opt, recognize},
    sequence::{delimited, pair},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Token types that can appear in decoration annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    /// Comma separator
    Comma,
    /// Equals sign
    Equal,
    /// Left parenthesis
    LParen,
    /// Right parenthesis
    RParen,
    /// Left bracket (for lists)
    LBracket,
    /// Right bracket
    RBracket,
    /// Left brace (for objects/dicts)
    LBrace,
    /// Right brace
    RBrace,
    /// Colon (for dict key:value)
    Colon,
    /// String literal (without quotes)
    String,
    /// Numeric literal
    Number,
    /// Identifier
    Identifier,
    /// Boolean literal
    Boolean,
    /// End of input
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::LParen => write!(f, "LPAREN"),
            TokenType::RParen => write!(f, "RPAREN"),
            TokenType::LBracket => write!(f, "LBRACKET"),
            TokenType::RBracket => write!(f, "RBRACKET"),
            TokenType::LBrace => write!(f, "LBRACE"),
            TokenType::RBrace => write!(f, "RBRACE"),
            TokenType::Colon => write!(f, "COLON"),
            TokenType::String => write!(f, "STRING"),
            TokenType::Number => write!(f, "NUMBER"),
            TokenType::Identifier => write!(f, "IDENTIFIER"),
            TokenType::Boolean => write!(f, "BOOLEAN"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its type, value, and position in the source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub position: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, position: usize) -> Self {
        Self {
            token_type,
            value,
            position,
        }
    }
}

/// Lexer for tokenizing annotation text
pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            position: 0,
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut remaining = self.input.as_str();
        let mut current_pos = 0;

        while !remaining.is_empty() {
            // Skip whitespace
            if let Ok((rest, _)) = multispace1::<&str, nom::error::Error<&str>>(remaining) {
                let consumed = remaining.len() - rest.len();
                current_pos += consumed;
                remaining = rest;
                continue;
            }

            // Try to parse a token
            match self.parse_token(remaining) {
                Ok((rest, token)) => {
                    let consumed = remaining.len() - rest.len();
                    tokens.push(Token::new(token.0, token.1, current_pos));
                    current_pos += consumed;
                    remaining = rest;
                }
                Err(_) => {
                    return Err(WezzteError::parse(
                        format!("Unexpected character: '{}'", remaining.chars().next().unwrap_or('?')),
                        current_pos,
                    ));
                }
            }
        }

        tokens.push(Token::new(TokenType::Eof, String::new(), current_pos));
        Ok(tokens)
    }

    /// Parse a single token from the input
    fn parse_token<'a>(&self, input: &'a str) -> IResult<&'a str, (TokenType, String)> {
        alt((
            // Single character tokens
            map(char(','), |_| (TokenType::Comma, ",".to_string())),
            map(char('='), |_| (TokenType::Equal, "=".to_string())),
            map(char('('), |_| (TokenType::LParen, "(".to_string())),
            map(char(')'), |_| (TokenType::RParen, ")".to_string())),
            map(char('['), |_| (TokenType::LBracket, "[".to_string())),
            map(char(']'), |_| (TokenType::RBracket, "]".to_string())),
            map(char('{'), |_| (TokenType::LBrace, "{".to_string())),
            map(char('}'), |_| (TokenType::RBrace, "}".to_string())),
            map(char(':'), |_| (TokenType::Colon, ":".to_string())),
            
            // String literals
            map(
                delimited(
                    char('"'),
                    take_while(|c: char| c != '"'),
                    char('"')
                ),
                |s: &str| (TokenType::String, s.to_string())
            ),
            
            // Numbers (integer or float)
            map(
                recognize(pair(
                    digit1,
                    opt(pair(char('.'), digit1))
                )),
                |s: &str| (TokenType::Number, s.to_string())
            ),
            
            // Boolean literals
            map(tag("true"), |s: &str| (TokenType::Boolean, s.to_string())),
            map(tag("false"), |s: &str| (TokenType::Boolean, s.to_string())),
            
            // Identifiers (must come after boolean to avoid conflicts)
            map(
                recognize(pair(
                    take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
                    take_while(|c: char| c.is_ascii_alphanumeric() || c == '_')
                )),
                |s: &str| (TokenType::Identifier, s.to_string())
            ),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("slider(min=10, max=42)");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 8); // identifier, lparen, identifier, equal, number, comma, identifier, equal, number, rparen, eof
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, "slider");
        assert_eq!(tokens[1].token_type, TokenType::LParen);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].value, "min");
        assert_eq!(tokens[3].token_type, TokenType::Equal);
        assert_eq!(tokens[4].token_type, TokenType::Number);
        assert_eq!(tokens[4].value, "10");
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#"options="Dark, Light, Auto""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, "options");
        assert_eq!(tokens[1].token_type, TokenType::Equal);
        assert_eq!(tokens[2].token_type, TokenType::String);
        assert_eq!(tokens[2].value, "Dark, Light, Auto");
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("alpha=true linked=false");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[1].token_type, TokenType::Equal);
        assert_eq!(tokens[2].token_type, TokenType::Boolean);
        assert_eq!(tokens[2].value, "true");
        assert_eq!(tokens[5].token_type, TokenType::Boolean);
        assert_eq!(tokens[5].value, "false");
    }

    #[test]
    fn test_complex_annotation() {
        let mut lexer = Lexer::new(r#"multi_select(options=["resize", "title"], defaults={resize: true})"#);
        let tokens = lexer.tokenize().unwrap();

        // Should have: identifier, lparen, identifier, equal, lbracket, string, comma, string, rbracket, comma, identifier, equal, lbrace, identifier, colon, boolean, rbrace, rparen, eof
        assert!(tokens.len() >= 15);
        assert_eq!(tokens[0].value, "multi_select");
        assert_eq!(tokens[5].token_type, TokenType::LBracket);
        assert_eq!(tokens[12].token_type, TokenType::LBrace);
        assert_eq!(tokens[14].token_type, TokenType::Colon);
    }

    #[test]
    fn test_error_handling() {
        let mut lexer = Lexer::new("test@invalid");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}