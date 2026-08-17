//! Typed errors for URL-encoded form parsing.

use core::fmt;

/// An error produced while parsing a URL-encoded form body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlEncodedError {
    /// The body is not valid UTF-8.
    InvalidUtf8,
    /// A key-value pair in the form is malformed.
    MalformedPair(String),
}

impl fmt::Display for UrlEncodedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => formatter.write_str("URL-encoded body is not valid UTF-8"),
            Self::MalformedPair(details) => {
                write!(formatter, "malformed key-value pair in URL-encoded form: {details}")
            }
        }
    }
}

impl std::error::Error for UrlEncodedError {}
