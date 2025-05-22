//! # ClickHouse Type Parser
//!
//! This module provides parsers and converters for ClickHouse data types.
//! It handles conversion between ClickHouse type strings and the framework's
//! type system, supporting complex nested structures and various type formats.

use crate::framework::core::infrastructure::table::{
    Column, ColumnType, DataEnum, EnumMember, EnumValue, FloatType, IntType, Nested,
};
use std::fmt;
use thiserror::Error;

// =========================================================
// Error Types
// =========================================================

/// Errors that can occur during ClickHouse type tokenization
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum TokenizerError {
    /// Invalid string format
    #[error("Invalid string literal: {message}")]
    InvalidString { message: String },

    /// Invalid number format
    #[error("Invalid number literal: {message}")]
    InvalidNumber { message: String },

    /// Unexpected character encountered
    #[error("Unexpected character '{character}' at position {position}")]
    UnexpectedCharacter { character: char, position: usize },

    /// Unterminated string literal
    #[error("Unterminated string literal starting at position {position}")]
    UnterminatedString { position: usize },
}

/// Errors that can occur during ClickHouse type parsing
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// Unexpected token encountered during parsing
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    /// End of input reached unexpectedly
    #[error("Unexpected end of input while parsing {context}")]
    UnexpectedEOF { context: &'static str },

    /// Missing parameter
    #[error("Missing parameter in {type_name}: {message}")]
    MissingParameter { type_name: String, message: String },

    /// Invalid parameter
    #[error("Invalid parameter in {type_name}: {message}")]
    InvalidParameter { type_name: String, message: String },

    /// General syntax error
    #[error("Syntax error: {message}")]
    SyntaxError { message: String },

    /// Unsupported type or feature
    #[error("Unsupported type: {type_name}")]
    UnsupportedType { type_name: String },

    /// Tokenizer error
    #[error("Tokenizer error: {0}")]
    TokenizerError(#[from] TokenizerError),
}

/// Errors that can occur during conversion from ClickHouse types to framework types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConversionError {
    /// The ClickHouse type doesn't have an equivalent in the framework type system
    #[error("Unsupported ClickHouse type: {type_name}")]
    UnsupportedType { type_name: String },

    /// The ClickHouse type's parameters are invalid or out of range
    #[error("Invalid type parameters for {type_name}: {message}")]
    InvalidParameters { type_name: String, message: String },

    /// Error during parsing of the ClickHouse type
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

/// Errors that can occur during the full ClickHouse type processing
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ClickHouseTypeError {
    /// Error related to parsing the type string
    #[error("Error parsing ClickHouse type string '{input}': {source}")]
    Parse {
        input: String,
        #[source]
        source: ParseError,
    },

    /// Error related to converting a type to a framework type
    #[error("Error converting ClickHouse type to framework type: {source}")]
    Conversion {
        #[source]
        source: ConversionError,
    },
}

// =========================================================
// Token and AST definitions
// =========================================================

/// Represents a token in the ClickHouse type syntax
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Identifier (type name, function name, etc.)
    Identifier(String),
    /// A string literal 'value' or "value"
    StringLiteral(String),
    /// A numeric literal
    NumberLiteral(u64),
    /// Left parenthesis (
    LeftParen,
    /// Right parenthesis )
    RightParen,
    /// Comma separator for parameters
    Comma,
    /// Equals sign in enum definitions
    Equals,
    /// End of input
    Eof,
}

/// Represents an AST node for a ClickHouse type
#[derive(Debug, Clone, PartialEq)]
pub enum ClickHouseTypeNode {
    /// Simple types without parameters (UInt8, String, etc.)
    Simple(String),

    /// Nullable(T)
    Nullable(Box<ClickHouseTypeNode>),

    /// Array(T)
    Array(Box<ClickHouseTypeNode>),

    /// LowCardinality(T)
    LowCardinality(Box<ClickHouseTypeNode>),

    /// Decimal with precision and scale
    Decimal { precision: u8, scale: u8 },

    /// Specialized Decimal with precision
    DecimalSized { bits: u16, precision: u8 },

    /// DateTime with optional timezone
    DateTime { timezone: Option<String> },

    /// DateTime64 with precision and optional timezone
    DateTime64 {
        precision: u8,
        timezone: Option<String>,
    },

    /// FixedString with length
    FixedString(u64),

    /// Enum8 or Enum16 with members
    Enum {
        bits: u8, // 8 or 16
        members: Vec<(String, u64)>,
    },

    /// Tuple with elements
    Tuple(Vec<TupleElement>),

    /// Nested with elements
    Nested(Vec<TupleElement>),

    /// Map with key and value types
    Map {
        key_type: Box<ClickHouseTypeNode>,
        value_type: Box<ClickHouseTypeNode>,
    },

    /// Aggregate function
    AggregateFunction {
        function_name: String,
        argument_types: Vec<ClickHouseTypeNode>,
    },

    /// SimpleAggregateFunction
    SimpleAggregateFunction {
        function_name: String,
        argument_type: Box<ClickHouseTypeNode>,
    },
}

/// Represents an element in a Tuple or Nested type
#[derive(Debug, Clone, PartialEq)]
pub enum TupleElement {
    /// Named element (name Type)
    Named {
        name: String,
        type_node: ClickHouseTypeNode,
    },
    /// Unnamed element (just Type)
    Unnamed(ClickHouseTypeNode),
}

impl fmt::Display for ClickHouseTypeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickHouseTypeNode::Simple(name) => write!(f, "{}", name),
            ClickHouseTypeNode::Nullable(inner) => write!(f, "Nullable({})", inner),
            ClickHouseTypeNode::Array(inner) => write!(f, "Array({})", inner),
            ClickHouseTypeNode::LowCardinality(inner) => write!(f, "LowCardinality({})", inner),
            ClickHouseTypeNode::Decimal { precision, scale } => {
                write!(f, "Decimal({}, {})", precision, scale)
            }
            ClickHouseTypeNode::DecimalSized { bits, precision } => {
                write!(f, "Decimal{}({})", bits, precision)
            }
            ClickHouseTypeNode::DateTime { timezone } => match timezone {
                Some(tz) => write!(f, "DateTime('{}')", tz),
                None => write!(f, "DateTime"),
            },
            ClickHouseTypeNode::DateTime64 {
                precision,
                timezone,
            } => match timezone {
                Some(tz) => write!(f, "DateTime64({}, '{}')", precision, tz),
                None => write!(f, "DateTime64({})", precision),
            },
            ClickHouseTypeNode::FixedString(length) => write!(f, "FixedString({})", length),
            ClickHouseTypeNode::Enum { bits, members } => {
                write!(f, "Enum{}(", bits)?;
                for (i, (name, value)) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "'{}' = {}", name, value)?;
                }
                write!(f, ")")
            }
            ClickHouseTypeNode::Tuple(elements) => {
                write!(f, "Tuple(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match element {
                        TupleElement::Named { name, type_node } => {
                            write!(f, "{} {}", name, type_node)?;
                        }
                        TupleElement::Unnamed(type_node) => {
                            write!(f, "{}", type_node)?;
                        }
                    }
                }
                write!(f, ")")
            }
            ClickHouseTypeNode::Nested(elements) => {
                write!(f, "Nested(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match element {
                        TupleElement::Named { name, type_node } => {
                            write!(f, "{} {}", name, type_node)?;
                        }
                        TupleElement::Unnamed(_) => {
                            // Nested elements should always be named
                            write!(f, "[invalid unnamed element]")?;
                        }
                    }
                }
                write!(f, ")")
            }
            ClickHouseTypeNode::Map {
                key_type,
                value_type,
            } => {
                write!(f, "Map({}, {})", key_type, value_type)
            }
            ClickHouseTypeNode::AggregateFunction {
                function_name,
                argument_types,
            } => {
                write!(f, "AggregateFunction({}", function_name)?;
                for arg_type in argument_types {
                    write!(f, ", {}", arg_type)?;
                }
                write!(f, ")")
            }
            ClickHouseTypeNode::SimpleAggregateFunction {
                function_name,
                argument_type,
            } => {
                write!(
                    f,
                    "SimpleAggregateFunction({}, {})",
                    function_name, argument_type
                )
            }
        }
    }
}

// =========================================================
// Lexer / Tokenizer
// =========================================================

/// Tokenizes a ClickHouse type string into a sequence of tokens
struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
    chars: std::str::Chars<'a>,
    current_char: Option<char>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current_char = chars.next();
        Self {
            input,
            position: 0,
            chars,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.chars.next();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();

        self.skip_whitespace();

        while let Some(c) = self.current_char {
            match c {
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equals);
                    self.advance();
                }
                '\'' | '"' => {
                    tokens.push(self.read_string()?);
                }
                '0'..='9' => {
                    tokens.push(self.read_number()?);
                }
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier());
                }
                _ => {
                    return Err(TokenizerError::UnexpectedCharacter {
                        character: c,
                        position: self.position,
                    });
                }
            }

            self.skip_whitespace();
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn read_string(&mut self) -> Result<Token, TokenizerError> {
        let quote_char = self.current_char.unwrap();
        let start_pos = self.position;
        self.advance();

        let mut value = String::new();
        let mut escape = false;

        while let Some(c) = self.current_char {
            if escape {
                match c {
                    '\\' | '\'' | '"' => value.push(c),
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    _ => {
                        return Err(TokenizerError::InvalidString {
                            message: format!(
                                "Invalid escape sequence '\\{}' at position {} in input: {}",
                                c, self.position, self.input
                            ),
                        });
                    }
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == quote_char {
                self.advance();
                return Ok(Token::StringLiteral(value));
            } else {
                value.push(c);
            }

            self.advance();
        }

        Err(TokenizerError::UnterminatedString {
            position: start_pos,
        })
    }

    fn read_number(&mut self) -> Result<Token, TokenizerError> {
        let mut value = String::new();

        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match value.parse::<u64>() {
            Ok(num) => Ok(Token::NumberLiteral(num)),
            Err(e) => Err(TokenizerError::InvalidNumber {
                message: format!("Invalid number '{}': {}", value, e),
            }),
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut value = String::new();

        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Token::Identifier(value)
    }
}

// =========================================================
// Parser
// =========================================================

/// Parser for ClickHouse type expressions
struct Parser {
    tokens: Vec<Token>,
    current_pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current_pos: 0,
        }
    }

    fn current_token(&self) -> &Token {
        if self.current_pos < self.tokens.len() {
            &self.tokens[self.current_pos]
        } else {
            // This should never happen as the tokenizer always adds an Eof token
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn consume(&mut self, expected: &Token) -> Result<(), ParseError> {
        let current = self.current_token();

        // Special case for Eof
        if let Token::Eof = expected {
            if !matches!(current, Token::Eof) {
                return Err(ParseError::UnexpectedToken {
                    expected: "end of input".to_string(),
                    found: self.token_to_string(current),
                });
            }
            return Ok(());
        }

        // For all other tokens
        if std::mem::discriminant(current) != std::mem::discriminant(expected) {
            return Err(ParseError::UnexpectedToken {
                expected: self.token_to_string(expected),
                found: self.token_to_string(current),
            });
        }

        self.advance();
        Ok(())
    }

    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::Identifier(s) => format!("identifier '{}'", s),
            Token::StringLiteral(s) => format!("string '{}'", s),
            Token::NumberLiteral(n) => format!("number {}", n),
            Token::LeftParen => "(".to_string(),
            Token::RightParen => ")".to_string(),
            Token::Comma => ",".to_string(),
            Token::Equals => "=".to_string(),
            Token::Eof => "end of input".to_string(),
        }
    }

    fn advance(&mut self) {
        if self.current_pos < self.tokens.len() - 1 {
            self.current_pos += 1;
        }
    }

    pub fn parse(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        let type_node = self.parse_type()?;
        self.consume(&Token::Eof)?;
        Ok(type_node)
    }

    fn parse_type(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        match self.current_token() {
            Token::Identifier(name) => {
                let name_clone = name.clone();
                self.advance();

                match name_clone.as_str() {
                    "Nullable" => self.parse_nullable(),
                    "Array" => self.parse_array(),
                    "LowCardinality" => self.parse_low_cardinality(),
                    "Decimal" => self.parse_decimal(),
                    "DateTime" => self.parse_datetime(),
                    "DateTime64" => self.parse_datetime64(),
                    "FixedString" => self.parse_fixed_string(),
                    "Tuple" => self.parse_tuple(),
                    "Nested" => self.parse_nested(),
                    "Map" => self.parse_map(),
                    "AggregateFunction" => self.parse_aggregate_function(),
                    "SimpleAggregateFunction" => self.parse_simple_aggregate_function(),
                    name if name.starts_with("Decimal") => self.parse_decimal_sized(&name_clone),
                    name if name.starts_with("Enum") => self.parse_enum(&name_clone),
                    name => Ok(ClickHouseTypeNode::Simple(name.to_string())),
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "type name".to_string(),
                found: self.token_to_string(self.current_token()),
            }),
        }
    }

    fn parse_nullable(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;
        let inner_type = self.parse_type()?;
        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Nullable(Box::new(inner_type)))
    }

    fn parse_array(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;
        let inner_type = self.parse_type()?;
        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Array(Box::new(inner_type)))
    }

    fn parse_low_cardinality(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;
        let inner_type = self.parse_type()?;
        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::LowCardinality(Box::new(inner_type)))
    }

    fn parse_decimal(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse precision
        let precision = match self.current_token() {
            Token::NumberLiteral(n) => *n as u8,
            _ => {
                return Err(ParseError::MissingParameter {
                    type_name: "Decimal".to_string(),
                    message: "number literal for precision".to_string(),
                });
            }
        };
        self.advance();

        // Parse comma
        self.consume(&Token::Comma)?;

        // Parse scale
        let scale = match self.current_token() {
            Token::NumberLiteral(n) => *n as u8,
            _ => {
                return Err(ParseError::MissingParameter {
                    type_name: "Decimal".to_string(),
                    message: "number literal for scale".to_string(),
                });
            }
        };
        self.advance();

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Decimal { precision, scale })
    }

    fn parse_decimal_sized(&mut self, type_name: &str) -> Result<ClickHouseTypeNode, ParseError> {
        // Extract bits from type name
        let bits = match type_name {
            "Decimal32" => 32,
            "Decimal64" => 64,
            "Decimal128" => 128,
            "Decimal256" => 256,
            _ => {
                return Err(ParseError::SyntaxError {
                    message: format!("Invalid decimal type name: {}", type_name),
                });
            }
        };

        self.consume(&Token::LeftParen)?;

        // Parse precision
        let precision = match self.current_token() {
            Token::NumberLiteral(n) => *n as u8,
            _ => {
                return Err(ParseError::MissingParameter {
                    type_name: type_name.to_string(),
                    message: "number literal for precision".to_string(),
                });
            }
        };

        self.advance();
        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::DecimalSized {
            bits: bits as u16,
            precision,
        })
    }

    fn parse_datetime(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        // Check if there are parameters (timezone)
        if matches!(self.current_token(), Token::LeftParen) {
            self.consume(&Token::LeftParen)?;

            // Parse timezone string
            let timezone = match self.current_token() {
                Token::StringLiteral(tz) => {
                    let tz_str = tz.clone();
                    self.advance();
                    Some(tz_str)
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "string literal for timezone".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            };

            self.consume(&Token::RightParen)?;

            Ok(ClickHouseTypeNode::DateTime { timezone })
        } else {
            // No parameters, just DateTime
            Ok(ClickHouseTypeNode::DateTime { timezone: None })
        }
    }

    fn parse_datetime64(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse precision
        let precision = match self.current_token() {
            Token::NumberLiteral(n) => *n as u8,
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "number literal for precision".to_string(),
                    found: format!("{:?}", self.current_token()),
                })
            }
        };
        self.advance();

        // Check for optional timezone
        let timezone = if matches!(self.current_token(), Token::Comma) {
            self.advance(); // Consume comma

            // Parse timezone string
            match self.current_token() {
                Token::StringLiteral(tz) => {
                    let tz_str = tz.clone();
                    self.advance();
                    Some(tz_str)
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "string literal for timezone".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            }
        } else {
            None
        };

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::DateTime64 {
            precision,
            timezone,
        })
    }

    /// Parse a FixedString(N) type
    fn parse_fixed_string(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse length
        let length = match self.current_token() {
            Token::NumberLiteral(n) => *n,
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "number literal for length".to_string(),
                    found: format!("{:?}", self.current_token()),
                })
            }
        };
        self.advance();

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::FixedString(length))
    }

    /// Parse an Enum8/16('value' = number, ...) type
    fn parse_enum(&mut self, type_name: &str) -> Result<ClickHouseTypeNode, ParseError> {
        // Extract bits from type name
        let bits = match type_name {
            "Enum8" => 8,
            "Enum16" => 16,
            _ => {
                return Err(ParseError::SyntaxError {
                    message: format!("Invalid enum type name: {}", type_name),
                });
            }
        };

        self.consume(&Token::LeftParen)?;

        let mut members = Vec::new();
        loop {
            // Parse string literal
            let name = match self.current_token() {
                Token::StringLiteral(s) => s.clone(),
                Token::RightParen if members.is_empty() => {
                    // Empty enum, break early
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "string literal or ')'".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            };
            self.advance();

            // Parse equals sign
            self.consume(&Token::Equals)?;

            // Parse number
            let value = match self.current_token() {
                Token::NumberLiteral(n) => *n,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "number literal for enum value".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            };
            self.advance();

            members.push((name, value));

            // Check for comma or end of list
            match self.current_token() {
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::RightParen => break,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "comma or ')'".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            }
        }

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Enum { bits, members })
    }

    /// Parse a Tuple(T1, T2, ...) or Tuple(name1 T1, name2 T2, ...) type
    fn parse_tuple(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        let mut elements = Vec::new();

        // Handle empty tuple case
        if matches!(self.current_token(), Token::RightParen) {
            self.advance();
            return Ok(ClickHouseTypeNode::Tuple(elements));
        }

        loop {
            // Try to parse a named tuple element first
            let element = match self.current_token() {
                Token::Identifier(name) => {
                    let element_name = name.clone();
                    self.advance();

                    // Check if next token is a type identifier
                    if let Token::Identifier(_) = self.current_token() {
                        // This is a named element
                        let type_node = self.parse_type()?;
                        TupleElement::Named {
                            name: element_name,
                            type_node,
                        }
                    } else {
                        // This is an unnamed element with the identifier as the type
                        self.current_pos -= 1; // Go back to re-parse the identifier as a type
                        let type_node = self.parse_type()?;
                        TupleElement::Unnamed(type_node)
                    }
                }
                _ => {
                    // This is an unnamed element
                    let type_node = self.parse_type()?;
                    TupleElement::Unnamed(type_node)
                }
            };

            elements.push(element);

            // Check for comma or end of list
            match self.current_token() {
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::RightParen => break,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "comma or ')'".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            }
        }

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Tuple(elements))
    }

    /// Parse a Nested(name1 T1, name2 T2, ...) type
    fn parse_nested(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        let mut elements = Vec::new();

        // Handle empty nested case
        if matches!(self.current_token(), Token::RightParen) {
            self.advance();
            return Ok(ClickHouseTypeNode::Nested(elements));
        }

        loop {
            // Nested type requires named elements
            let element = match self.current_token() {
                Token::Identifier(name) => {
                    let element_name = name.clone();
                    self.advance();

                    // Parse the type
                    let type_node = self.parse_type()?;
                    TupleElement::Named {
                        name: element_name,
                        type_node,
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier for column name".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            };

            elements.push(element);

            // Check for comma or end of list
            match self.current_token() {
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::RightParen => break,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "comma or ')'".to_string(),
                        found: format!("{:?}", self.current_token()),
                    })
                }
            }
        }

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Nested(elements))
    }

    /// Parse a Map(K, V) type
    fn parse_map(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse key type
        let key_type = self.parse_type()?;

        // Parse comma
        self.consume(&Token::Comma)?;

        // Parse value type
        let value_type = self.parse_type()?;

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::Map {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        })
    }

    /// Parse an AggregateFunction(name, T1, T2, ...) type
    fn parse_aggregate_function(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse function name
        let function_name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier for function name".to_string(),
                    found: format!("{:?}", self.current_token()),
                })
            }
        };
        self.advance();

        let mut argument_types = Vec::new();

        // Check if there are any arguments
        if matches!(self.current_token(), Token::Comma) {
            loop {
                self.consume(&Token::Comma)?;

                // Parse argument type
                let arg_type = self.parse_type()?;
                argument_types.push(arg_type);

                // Check if there are more arguments
                if !matches!(self.current_token(), Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::AggregateFunction {
            function_name,
            argument_types,
        })
    }

    /// Parse a SimpleAggregateFunction(name, T) type
    fn parse_simple_aggregate_function(&mut self) -> Result<ClickHouseTypeNode, ParseError> {
        self.consume(&Token::LeftParen)?;

        // Parse function name
        let function_name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier for function name".to_string(),
                    found: format!("{:?}", self.current_token()),
                })
            }
        };
        self.advance();

        // Parse comma
        self.consume(&Token::Comma)?;

        // Parse argument type
        let argument_type = self.parse_type()?;

        self.consume(&Token::RightParen)?;

        Ok(ClickHouseTypeNode::SimpleAggregateFunction {
            function_name,
            argument_type: Box::new(argument_type),
        })
    }
}

/// Parse a ClickHouse type string into an AST
pub fn parse_clickhouse_type(input: &str) -> Result<ClickHouseTypeNode, ParseError> {
    let mut tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize()?;

    let mut parser = Parser::new(tokens);
    parser.parse()
}

// =========================================================
// Conversion to Framework Types
// =========================================================

/// Convert a parsed ClickHouse type to the framework's ColumnType
pub fn convert_ast_to_column_type(
    node: &ClickHouseTypeNode,
) -> Result<(ColumnType, bool), ConversionError> {
    match node {
        ClickHouseTypeNode::Simple(name) => {
            let column_type = match name.as_str() {
                "String" => Ok(ColumnType::String),
                "Int8" => Ok(ColumnType::Int(IntType::Int8)),
                "Int16" => Ok(ColumnType::Int(IntType::Int16)),
                "Int32" => Ok(ColumnType::Int(IntType::Int32)),
                "Int64" => Ok(ColumnType::Int(IntType::Int64)),
                "Int128" => Ok(ColumnType::Int(IntType::Int128)),
                "Int256" => Ok(ColumnType::Int(IntType::Int256)),
                "UInt8" => Ok(ColumnType::Int(IntType::UInt8)),
                "UInt16" => Ok(ColumnType::Int(IntType::UInt16)),
                "UInt32" => Ok(ColumnType::Int(IntType::UInt32)),
                "UInt64" => Ok(ColumnType::Int(IntType::UInt64)),
                "UInt128" => Ok(ColumnType::Int(IntType::UInt128)),
                "UInt256" => Ok(ColumnType::Int(IntType::UInt256)),
                "Float32" => Ok(ColumnType::Float(FloatType::Float32)),
                "Float64" => Ok(ColumnType::Float(FloatType::Float64)),
                "Bool" | "Boolean" => Ok(ColumnType::Boolean),
                "JSON" => Ok(ColumnType::Json),
                "UUID" => Ok(ColumnType::Uuid),
                "Date" => Ok(ColumnType::Date16),
                "Date32" => Ok(ColumnType::Date),
                "DateTime" => Ok(ColumnType::DateTime { precision: None }),
                _ => Err(ConversionError::UnsupportedType {
                    type_name: name.clone(),
                }),
            }?;

            Ok((column_type, false))
        }

        ClickHouseTypeNode::Nullable(inner) => {
            let (inner_type, _) = convert_ast_to_column_type(inner)?;
            Ok((inner_type, true))
        }

        ClickHouseTypeNode::Array(inner) => {
            let (inner_type, is_nullable) = convert_ast_to_column_type(inner)?;
            Ok((
                ColumnType::Array {
                    element_type: Box::new(inner_type),
                    element_nullable: is_nullable,
                },
                false,
            ))
        }

        ClickHouseTypeNode::LowCardinality(inner) => {
            // LowCardinality is an optimization hint in ClickHouse,
            // we just use the inner type in our framework
            convert_ast_to_column_type(inner)
        }

        ClickHouseTypeNode::Decimal { precision, scale } => Ok((
            ColumnType::Decimal {
                precision: *precision,
                scale: *scale,
            },
            false,
        )),

        ClickHouseTypeNode::DecimalSized { bits, precision } => {
            // Make sure the precision is valid for the bit size
            let max_precision = match *bits {
                32 => 9,
                64 => 18,
                128 => 38,
                256 => 76,
                _ => {
                    return Err(ConversionError::InvalidParameters {
                        type_name: format!("Decimal{}", bits),
                        message: format!("Invalid bit size: {}", bits),
                    });
                }
            };

            if *precision > max_precision {
                return Err(ConversionError::InvalidParameters {
                    type_name: format!("Decimal{}", bits),
                    message: format!(
                        "Precision {} exceeds maximum {} for Decimal{}",
                        precision, max_precision, bits
                    ),
                });
            }

            // We only track precision and scale in our type system
            Ok((
                ColumnType::Decimal {
                    precision: *precision,
                    scale: 0, // Default scale for DecimalN types
                },
                false,
            ))
        }

        ClickHouseTypeNode::DateTime { timezone: _ } => {
            // We don't currently track timezone in our framework type system
            Ok((ColumnType::DateTime { precision: None }, false))
        }

        ClickHouseTypeNode::DateTime64 {
            precision,
            timezone: _,
        } => {
            // We don't currently track timezone in our framework type system
            Ok((
                ColumnType::DateTime {
                    precision: Some(*precision),
                },
                false,
            ))
        }

        ClickHouseTypeNode::FixedString(_) => {
            // FixedString is mapped to regular String in our type system
            Ok((ColumnType::String, false))
        }

        ClickHouseTypeNode::Enum { bits, members } => {
            let enum_members = members
                .iter()
                .map(|(name, value)| EnumMember {
                    name: name.clone(),
                    value: EnumValue::Int(*value as u8),
                })
                .collect::<Vec<_>>();

            Ok((
                ColumnType::Enum(DataEnum {
                    name: format!("Enum{}", bits),
                    values: enum_members,
                }),
                false,
            ))
        }

        ClickHouseTypeNode::Nested(elements) => {
            let mut columns = Vec::new();

            for element in elements {
                match element {
                    TupleElement::Named { name, type_node } => {
                        let (data_type, is_nullable) = convert_ast_to_column_type(type_node)?;

                        columns.push(Column {
                            name: name.clone(),
                            data_type,
                            required: !is_nullable,
                            unique: false,
                            primary_key: false,
                            default: None,
                            annotations: Vec::new(),
                        });
                    }
                    TupleElement::Unnamed(_) => {
                        return Err(ConversionError::InvalidParameters {
                            type_name: "Nested".to_string(),
                            message: "Unnamed elements not allowed in Nested type".to_string(),
                        });
                    }
                }
            }

            // Generate a name based on content if there are columns
            let nested_name = if !columns.is_empty() {
                format!("nested_{}", columns.len())
            } else {
                "nested".to_string()
            };

            Ok((
                ColumnType::Nested(Nested {
                    name: nested_name,
                    columns,
                    jwt: false,
                }),
                false,
            ))
        }

        ClickHouseTypeNode::Tuple(_) => {
            // We don't have a direct equivalent for Tuple in the Moose type system
            Err(ConversionError::UnsupportedType {
                type_name: "Tuple".to_string(),
            })
        }

        ClickHouseTypeNode::Map { .. } => {
            // We don't have a direct equivalent for Map in the Moose type system
            Err(ConversionError::UnsupportedType {
                type_name: "Map".to_string(),
            })
        }

        ClickHouseTypeNode::AggregateFunction { .. } => {
            // AggregateFunction is specialized, and we don't have a direct mapping.
            // These are typically used in materialized views, not in regular tables.
            Err(ConversionError::UnsupportedType {
                type_name: "AggregateFunction".to_string(),
            })
        }

        ClickHouseTypeNode::SimpleAggregateFunction { .. } => {
            // Same as AggregateFunction
            Err(ConversionError::UnsupportedType {
                type_name: "SimpleAggregateFunction".to_string(),
            })
        }
    }
}

/// Converts a ClickHouse type string to the framework's ColumnType
///
/// # Arguments
/// * `ch_type` - The ClickHouse type string to convert
///
/// # Returns
/// * `Result<(ColumnType, bool), ClickHouseTypeError>` - A tuple containing:
///   - The converted framework type
///   - A boolean indicating if the type is nullable (true = nullable)
pub fn convert_clickhouse_type_to_column_type(
    ch_type: &str,
) -> Result<(ColumnType, bool), ClickHouseTypeError> {
    // Parse the ClickHouse type string into an AST
    let type_node = parse_clickhouse_type(ch_type).map_err(|e| ClickHouseTypeError::Parse {
        input: ch_type.to_string(),
        source: e,
    })?;

    // Convert the AST to a framework type
    convert_ast_to_column_type(&type_node)
        .map_err(|e| ClickHouseTypeError::Conversion { source: e })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let input = "Nullable(Array(String))";
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize().unwrap();

        // Compare token types and values individually
        assert_eq!(tokens.len(), 8);
        assert!(matches!(tokens[0], Token::Identifier(ref s) if s == "Nullable"));
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::Identifier(ref s) if s == "Array"));
        assert!(matches!(tokens[3], Token::LeftParen));
        assert!(matches!(tokens[4], Token::Identifier(ref s) if s == "String"));
        assert!(matches!(tokens[5], Token::RightParen));
        assert!(matches!(tokens[6], Token::RightParen));
        assert!(matches!(tokens[7], Token::Eof));
    }

    #[test]
    fn test_parse_simple_types() {
        let types = vec![
            "String", "Int32", "UInt64", "Float32", "Boolean", "UUID", "Date32",
        ];

        for type_str in types {
            let result = parse_clickhouse_type(type_str);
            assert!(result.is_ok(), "Failed to parse {}: {:?}", type_str, result);
            assert_eq!(
                result.unwrap(),
                ClickHouseTypeNode::Simple(type_str.to_string())
            );
        }

        // Test DateTime specially since it's now a separate type
        let result = parse_clickhouse_type("DateTime");
        assert!(result.is_ok(), "Failed to parse DateTime: {:?}", result);
        assert_eq!(
            result.unwrap(),
            ClickHouseTypeNode::DateTime { timezone: None }
        );
    }

    #[test]
    fn test_parse_nullable() {
        let result = parse_clickhouse_type("Nullable(String)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::Nullable(Box::new(ClickHouseTypeNode::Simple(
                "String".to_string()
            )))
        );
    }

    #[test]
    fn test_parse_array() {
        let result = parse_clickhouse_type("Array(Int32)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::Array(Box::new(ClickHouseTypeNode::Simple("Int32".to_string())))
        );
    }

    #[test]
    fn test_parse_nested_types() {
        let result = parse_clickhouse_type("Nullable(Array(String))").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::Nullable(Box::new(ClickHouseTypeNode::Array(Box::new(
                ClickHouseTypeNode::Simple("String".to_string())
            ))))
        );
    }

    #[test]
    fn test_parse_decimal() {
        let result = parse_clickhouse_type("Decimal(10, 2)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::Decimal {
                precision: 10,
                scale: 2,
            }
        );
    }

    #[test]
    fn test_parse_decimal_sized() {
        let result = parse_clickhouse_type("Decimal64(10)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::DecimalSized {
                bits: 64,
                precision: 10,
            }
        );
    }

    #[test]
    fn test_parse_datetime() {
        // Test without timezone
        let result = parse_clickhouse_type("DateTime").unwrap();
        assert_eq!(result, ClickHouseTypeNode::DateTime { timezone: None });

        // Test with timezone
        let result = parse_clickhouse_type("DateTime('UTC')").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::DateTime {
                timezone: Some("UTC".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_fixed_string() {
        let result = parse_clickhouse_type("FixedString(16)").unwrap();
        assert_eq!(result, ClickHouseTypeNode::FixedString(16));
    }

    #[test]
    fn test_parse_enum() {
        let result = parse_clickhouse_type("Enum8('red' = 1, 'green' = 2, 'blue' = 3)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::Enum {
                bits: 8,
                members: vec![
                    ("red".to_string(), 1),
                    ("green".to_string(), 2),
                    ("blue".to_string(), 3),
                ],
            }
        );
    }

    #[test]
    fn test_parse_tuple() {
        // Test unnamed tuple
        let result = parse_clickhouse_type("Tuple(String, Int32)").unwrap();
        match result {
            ClickHouseTypeNode::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], TupleElement::Unnamed(_)));
                assert!(matches!(elements[1], TupleElement::Unnamed(_)));
            }
            _ => panic!("Expected Tuple type"),
        }

        // Test named tuple
        let result = parse_clickhouse_type("Tuple(name String, id Int32)").unwrap();
        match result {
            ClickHouseTypeNode::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], TupleElement::Named { .. }));
                assert!(matches!(elements[1], TupleElement::Named { .. }));

                if let TupleElement::Named { name, .. } = &elements[0] {
                    assert_eq!(name, "name");
                }
                if let TupleElement::Named { name, .. } = &elements[1] {
                    assert_eq!(name, "id");
                }
            }
            _ => panic!("Expected Tuple type"),
        }
    }

    #[test]
    fn test_parse_nested() {
        let result = parse_clickhouse_type("Nested(name String, id UInt32)").unwrap();
        match result {
            ClickHouseTypeNode::Nested(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], TupleElement::Named { .. }));
                assert!(matches!(elements[1], TupleElement::Named { .. }));

                if let TupleElement::Named { name, type_node } = &elements[0] {
                    assert_eq!(name, "name");
                    assert_eq!(*type_node, ClickHouseTypeNode::Simple("String".to_string()));
                }
                if let TupleElement::Named { name, type_node } = &elements[1] {
                    assert_eq!(name, "id");
                    assert_eq!(*type_node, ClickHouseTypeNode::Simple("UInt32".to_string()));
                }
            }
            _ => panic!("Expected Nested type"),
        }
    }

    #[test]
    fn test_parse_map() {
        let result = parse_clickhouse_type("Map(String, Int32)").unwrap();
        match result {
            ClickHouseTypeNode::Map {
                key_type,
                value_type,
            } => {
                assert_eq!(*key_type, ClickHouseTypeNode::Simple("String".to_string()));
                assert_eq!(*value_type, ClickHouseTypeNode::Simple("Int32".to_string()));
            }
            _ => panic!("Expected Map type"),
        }
    }

    #[test]
    fn test_parse_aggregate_function() {
        let result = parse_clickhouse_type("AggregateFunction(sum, Int32)").unwrap();
        match result {
            ClickHouseTypeNode::AggregateFunction {
                function_name,
                argument_types,
            } => {
                assert_eq!(function_name, "sum");
                assert_eq!(argument_types.len(), 1);
                assert_eq!(
                    argument_types[0],
                    ClickHouseTypeNode::Simple("Int32".to_string())
                );
            }
            _ => panic!("Expected AggregateFunction type"),
        }
    }

    #[test]
    fn test_complex_types() {
        // Test an extremely complex type
        let complex_type =
            "Array(Nullable(Map(String, Tuple(x UInt32, y Array(Nullable(String))))))";
        let result = parse_clickhouse_type(complex_type);
        assert!(result.is_ok(), "Failed to parse complex type: {:?}", result);

        // Test serialization/deserialization idempotence
        let node = result.unwrap();
        let serialized = node.to_string();
        let reparsed = parse_clickhouse_type(&serialized);
        assert!(
            reparsed.is_ok(),
            "Failed to reparse serialized type: {:?}",
            reparsed
        );

        // Conversion of complex types with Map and Tuple should now fail
        let conversion = convert_ast_to_column_type(&node);
        assert!(
            conversion.is_err(),
            "Conversion of complex type with Map and Tuple should fail"
        );
    }

    #[test]
    fn test_convert_unsupported_types() {
        // Test that Map type conversion fails
        let map_type = parse_clickhouse_type("Map(String, Int32)").unwrap();
        let map_result = convert_ast_to_column_type(&map_type);
        assert!(map_result.is_err(), "Map type should not be convertible");

        if let Err(ConversionError::UnsupportedType { type_name }) = map_result {
            assert_eq!(type_name, "Map");
        } else {
            panic!("Expected UnsupportedType error for Map");
        }

        // Test that Tuple type conversion fails
        let tuple_type = parse_clickhouse_type("Tuple(String, Int32)").unwrap();
        let tuple_result = convert_ast_to_column_type(&tuple_type);
        assert!(
            tuple_result.is_err(),
            "Tuple type should not be convertible"
        );

        if let Err(ConversionError::UnsupportedType { type_name }) = tuple_result {
            assert_eq!(type_name, "Tuple");
        } else {
            panic!("Expected UnsupportedType error for Tuple");
        }

        // Test that AggregateFunction type conversion fails
        let agg_type = parse_clickhouse_type("AggregateFunction(sum, Int32)").unwrap();
        let agg_result = convert_ast_to_column_type(&agg_type);
        assert!(
            agg_result.is_err(),
            "AggregateFunction type should not be convertible"
        );

        if let Err(ConversionError::UnsupportedType { type_name }) = agg_result {
            assert_eq!(type_name, "AggregateFunction");
        } else {
            panic!("Expected UnsupportedType error for AggregateFunction");
        }

        // Test that SimpleAggregateFunction type conversion fails
        let simple_agg_type = parse_clickhouse_type("SimpleAggregateFunction(sum, Int32)").unwrap();
        let simple_agg_result = convert_ast_to_column_type(&simple_agg_type);
        assert!(
            simple_agg_result.is_err(),
            "SimpleAggregateFunction type should not be convertible"
        );

        if let Err(ConversionError::UnsupportedType { type_name }) = simple_agg_result {
            assert_eq!(type_name, "SimpleAggregateFunction");
        } else {
            panic!("Expected UnsupportedType error for SimpleAggregateFunction");
        }

        // Test the full conversion function with the top level ClickHouseTypeError
        let result = convert_clickhouse_type_to_column_type("Tuple(String, Int32)");
        assert!(result.is_err(), "Tuple type should not be convertible");

        // Check the proper error layering
        if let Err(ClickHouseTypeError::Conversion { source }) = result {
            if let ConversionError::UnsupportedType { type_name } = source {
                assert_eq!(type_name, "Tuple");
            } else {
                panic!("Expected UnsupportedType error for Tuple");
            }
        } else {
            panic!("Expected Conversion error with UnsupportedType source");
        }

        // Test parsing invalid syntax results in a Parse error
        let invalid_syntax_result = convert_clickhouse_type_to_column_type("NotValid(");
        assert!(invalid_syntax_result.is_err(), "Invalid syntax should fail");

        if let Err(ClickHouseTypeError::Parse { input, source: _ }) = invalid_syntax_result {
            assert_eq!(input, "NotValid(");
        } else {
            panic!("Expected Parse error for invalid syntax");
        }
    }

    #[test]
    fn test_idempotent_conversion() {
        // Ensure parsing and formatting is idempotent
        let test_types = vec![
            "String",
            "Nullable(String)",
            "Array(Int32)",
            "Array(Nullable(String))",
            "Decimal(10, 2)",
            "DateTime",
            "DateTime('UTC')",
            "DateTime64(3)",
            "DateTime64(3, 'UTC')",
            "Enum8('red' = 1, 'green' = 2, 'blue' = 3)",
            "Tuple(String, Int32)",
            "Tuple(name String, id UInt32)",
            "Nested(name String, id UInt32)",
            "Map(String, Int32)",
            "LowCardinality(String)",
        ];

        // Test types for parsing and string serialization idempotence
        for type_str in test_types {
            // Parse the type string
            let parsed = parse_clickhouse_type(type_str).unwrap();

            // Convert back to string
            let serialized = parsed.to_string();

            // Parse the serialized string
            let reparsed = parse_clickhouse_type(&serialized).unwrap();

            // Compare the ASTs
            assert_eq!(parsed, reparsed, "Type not idempotent: {}", type_str);
        }

        // Test types for conversion to framework types (only those we support)
        let conversion_test_types = vec![
            "String",
            "Nullable(String)",
            "Array(Int32)",
            "Array(Nullable(String))",
            "Decimal(10, 2)",
            "DateTime",
            "DateTime('UTC')",
            "DateTime64(3)",
            "DateTime64(3, 'UTC')",
            "Enum8('red' = 1, 'green' = 2, 'blue' = 3)",
            "Nested(name String, id UInt32)",
            "LowCardinality(String)",
        ];

        for type_str in conversion_test_types {
            let parsed = parse_clickhouse_type(type_str).unwrap();
            let conversion = convert_ast_to_column_type(&parsed);
            assert!(
                conversion.is_ok(),
                "Type {} should be convertible but got error: {:?}",
                type_str,
                conversion.err()
            );
        }
    }

    #[test]
    fn test_convert_to_column_type() {
        let types = vec![
            ("String", ColumnType::String, false),
            ("Int32", ColumnType::Int(IntType::Int32), false),
            ("UInt64", ColumnType::Int(IntType::UInt64), false),
            ("Float32", ColumnType::Float(FloatType::Float32), false),
            ("Boolean", ColumnType::Boolean, false),
            ("UUID", ColumnType::Uuid, false),
            ("Nullable(String)", ColumnType::String, true),
            ("Nullable(Int32)", ColumnType::Int(IntType::Int32), true),
        ];

        for (ch_type, expected_type, expected_nullable) in types {
            let (actual_type, actual_nullable) =
                convert_clickhouse_type_to_column_type(ch_type).unwrap();
            assert_eq!(actual_type, expected_type, "Failed on type {}", ch_type);
            assert_eq!(
                actual_nullable, expected_nullable,
                "Failed on nullable {}",
                ch_type
            );
        }
    }

    #[test]
    fn test_convert_array_type() {
        // Test simple array
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Array(Int32)").unwrap();
        assert!(!is_nullable);
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::Int(IntType::Int32));
                assert!(!element_nullable);
            }
            _ => panic!("Expected Array type"),
        }

        // Test array of nullable elements
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Array(Nullable(String))").unwrap();
        assert!(!is_nullable);
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::String);
                assert!(element_nullable);
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_convert_nested_type() {
        let ch_type = "Nested(col1 String, col2 Int32)";
        let (column_type, is_nullable) = convert_clickhouse_type_to_column_type(ch_type).unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::Nested(nested) => {
                assert_eq!(nested.columns.len(), 2);
                assert_eq!(nested.columns[0].name, "col1");
                assert_eq!(nested.columns[1].name, "col2");
                assert_eq!(nested.columns[0].data_type, ColumnType::String);
                assert_eq!(nested.columns[1].data_type, ColumnType::Int(IntType::Int32));
            }
            _ => panic!("Expected Nested type"),
        }
    }

    #[test]
    fn test_convert_complex_nested_type() {
        let ch_type = "Nested(name String, id UInt32, meta Nested(key String, value String))";
        let (column_type, is_nullable) = convert_clickhouse_type_to_column_type(ch_type).unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::Nested(nested) => {
                assert_eq!(nested.columns.len(), 3);
                assert_eq!(nested.columns[0].name, "name");
                assert_eq!(nested.columns[1].name, "id");
                assert_eq!(nested.columns[2].name, "meta");

                // Check the nested structure
                match &nested.columns[2].data_type {
                    ColumnType::Nested(inner_nested) => {
                        assert_eq!(inner_nested.columns.len(), 2);
                        assert_eq!(inner_nested.columns[0].name, "key");
                        assert_eq!(inner_nested.columns[1].name, "value");
                    }
                    _ => panic!("Expected Nested type for 'meta' column"),
                }
            }
            _ => panic!("Expected Nested type"),
        }
    }

    #[test]
    fn test_convert_enum_type() {
        let ch_type = "Enum8('RED' = 1, 'GREEN' = 2, 'BLUE' = 3)";
        let (column_type, is_nullable) = convert_clickhouse_type_to_column_type(ch_type).unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::Enum(data_enum) => {
                assert_eq!(data_enum.values.len(), 3);
                assert_eq!(data_enum.values[0].name, "RED");
                assert_eq!(data_enum.values[0].value, EnumValue::Int(1));
                assert_eq!(data_enum.values[1].name, "GREEN");
                assert_eq!(data_enum.values[1].value, EnumValue::Int(2));
                assert_eq!(data_enum.values[2].name, "BLUE");
                assert_eq!(data_enum.values[2].value, EnumValue::Int(3));
            }
            _ => panic!("Expected Enum type"),
        }
    }

    #[test]
    fn test_convert_decimal_type() {
        let (column_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Decimal(10, 2)").unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::Decimal { precision, scale } => {
                assert_eq!(precision, 10);
                assert_eq!(scale, 2);
            }
            _ => panic!("Expected Decimal type"),
        }
    }

    #[test]
    fn test_convert_datetime_types() {
        // Test DateTime
        let (column_type, is_nullable) =
            convert_clickhouse_type_to_column_type("DateTime").unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::DateTime { precision } => {
                assert_eq!(precision, None);
            }
            _ => panic!("Expected DateTime type"),
        }

        // Test DateTime with timezone
        let (column_type, is_nullable) =
            convert_clickhouse_type_to_column_type("DateTime('UTC')").unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::DateTime { precision } => {
                assert_eq!(precision, None);
            }
            _ => panic!("Expected DateTime type"),
        }

        // Test DateTime64 with precision
        let (column_type, is_nullable) =
            convert_clickhouse_type_to_column_type("DateTime64(3)").unwrap();
        assert!(!is_nullable);
        match column_type {
            ColumnType::DateTime { precision } => {
                assert_eq!(precision, Some(3));
            }
            _ => panic!("Expected DateTime type"),
        }
    }

    // Add a new test for error handling specifically
    #[test]
    fn test_error_handling() {
        // Test tokenizer errors
        let unterminated_string = parse_clickhouse_type("Enum8('RED = 1");
        assert!(
            unterminated_string.is_err(),
            "Unterminated string should fail"
        );

        match unterminated_string {
            Err(ParseError::TokenizerError(TokenizerError::UnterminatedString { position })) => {
                assert_eq!(position, 6); // Position where the string starts
            }
            _ => panic!("Expected TokenizerError::UnterminatedString"),
        }

        // Test invalid Nested syntax - should fail during parsing
        let invalid_nested = parse_clickhouse_type("Nested(Int32)");
        assert!(invalid_nested.is_err(), "Invalid Nested format should fail");

        match invalid_nested {
            Err(ParseError::UnexpectedToken { expected, found }) => {
                assert_eq!(expected, "type name");
                assert_eq!(found, ")");
            }
            _ => panic!("Expected ParseError::UnexpectedToken"),
        }

        // Test valid named Nested type parsing and conversion
        let valid_nested = parse_clickhouse_type("Nested(col1 String)").unwrap();
        let nested_conversion = convert_ast_to_column_type(&valid_nested).unwrap();
        // Verify the conversion succeeds and produces the expected result
        match nested_conversion.0 {
            ColumnType::Nested(nested) => {
                assert_eq!(nested.columns.len(), 1);
                assert_eq!(nested.columns[0].name, "col1");
                assert_eq!(nested.columns[0].data_type, ColumnType::String);
            }
            _ => panic!("Expected Nested type"),
        }

        // Test unsupported type conversion
        let tuple_type = parse_clickhouse_type("Tuple(Int32, String)").unwrap();
        let tuple_conversion = convert_ast_to_column_type(&tuple_type);
        assert!(
            tuple_conversion.is_err(),
            "Tuple type should not be convertible"
        );

        match tuple_conversion {
            Err(ConversionError::UnsupportedType { type_name }) => {
                assert_eq!(type_name, "Tuple");
            }
            _ => panic!("Expected ConversionError::UnsupportedType"),
        }

        // Test unsupported type string
        let unsupported_type = convert_clickhouse_type_to_column_type("CustomType");
        assert!(unsupported_type.is_err(), "Unsupported type should fail");

        match unsupported_type {
            Err(ClickHouseTypeError::Conversion {
                source: ConversionError::UnsupportedType { type_name },
            }) => {
                assert_eq!(type_name, "CustomType");
            }
            _ => panic!("Expected ClickHouseTypeError::Conversion with UnsupportedType source"),
        }
    }

    #[test]
    fn test_parse_datetime64() {
        // Test without timezone
        let result = parse_clickhouse_type("DateTime64(3)").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::DateTime64 {
                precision: 3,
                timezone: None,
            }
        );

        // Test with timezone
        let result = parse_clickhouse_type("DateTime64(3, 'UTC')").unwrap();
        assert_eq!(
            result,
            ClickHouseTypeNode::DateTime64 {
                precision: 3,
                timezone: Some("UTC".to_string()),
            }
        );
    }
}
