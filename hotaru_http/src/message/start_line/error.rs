//! Errors for HTTP start-line parsing.

use core::fmt;

use crate::message::http_value::StatusCode;

/// Errors raised while parsing or validating a request or response start line.
#[derive(Debug)]
#[non_exhaustive]
pub enum StartLineError {
    /// Input line was empty or whitespace-only.
    Empty,
    /// The line could not be parsed as either a request or a response start line.
    Unrecognised,
    /// Response status code was not a parseable number.
    InvalidStatusCode,
    /// The HTTP-version token did not match the `HTTP/<major>.<minor>` grammar.
    InvalidHttpVersion(String),
    /// The HTTP-version token was valid but is not supported by this HTTP/1 handler.
    UnsupportedHttpVersion(String),
    /// The request method is not a valid HTTP token.
    InvalidMethodToken,
}

impl fmt::Display for StartLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("start line is empty"),
            Self::Unrecognised => formatter.write_str("start line could not be parsed"),
            Self::InvalidStatusCode => formatter.write_str("response status code is not a number"),
            Self::InvalidHttpVersion(version) => {
                write!(formatter, "invalid HTTP version: {version}")
            }
            Self::UnsupportedHttpVersion(version) => {
                write!(formatter, "unsupported HTTP version: {version}")
            }
            Self::InvalidMethodToken => {
                formatter.write_str("request method is not a valid HTTP token")
            }
        }
    }
}

impl std::error::Error for StartLineError {}

impl StartLineError {
    /// A rejected start line cannot safely authorize reuse of the connection.
    /// In particular, an unsupported version might define framing semantics
    /// that this HTTP/1 implementation does not understand.
    pub fn can_continue(&self) -> bool {
        false
    }
}

impl From<&StartLineError> for StatusCode {
    fn from(error: &StartLineError) -> Self {
        match error {
            StartLineError::UnsupportedHttpVersion(_) => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_invalid_and_unsupported_versions_to_distinct_statuses() {
        assert_eq!(
            StatusCode::from(&StartLineError::InvalidHttpVersion("HTTPX".to_string())),
            StatusCode::BAD_REQUEST,
        );
        assert_eq!(
            StatusCode::from(&StartLineError::UnsupportedHttpVersion(
                "HTTP/9.9".to_string()
            )),
            StatusCode::HTTP_VERSION_NOT_SUPPORTED,
        );
    }
}
