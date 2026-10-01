use std::fmt;

/// An error produced when a literal string does not conform to the grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        ParseError(message.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}
