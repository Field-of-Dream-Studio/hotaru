//! Errors produced while parsing an HTTP `Connection` header.

use core::fmt;

use crate::message::http_value::StatusCode;

/// An invalid connection-option token.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionError {
    /// A connection-option was empty.
    EmptyToken,
    /// A connection-option contained a character outside the HTTP token grammar.
    InvalidToken(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyToken => formatter.write_str("connection-option token is empty"),
            Self::InvalidToken(token) => {
                write!(formatter, "invalid connection-option token: {token}")
            }
        }
    }
}

impl std::error::Error for ConnectionError {}

impl ConnectionError {
    /// A malformed persistence directive must not authorize connection reuse.
    pub fn can_continue(&self) -> bool {
        false
    }
}

impl From<&ConnectionError> for StatusCode {
    fn from(_: &ConnectionError) -> Self {
        StatusCode::BAD_REQUEST
    }
}
