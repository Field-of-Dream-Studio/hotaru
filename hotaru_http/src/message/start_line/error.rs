//! Errors for HTTP start-line parsing.

use core::fmt;

use crate::message::http_value::StatusCode;

/// Errors raised while parsing a request or response start line.
#[derive(Debug)]
#[non_exhaustive]
pub enum StartLineError {
    /// Input line was empty or whitespace-only.
    Empty,
    /// The line could not be parsed as either a request or a response start line.
    Unrecognised,
    /// Response status code was not a parseable number.
    InvalidStatusCode,
    /// The HTTP version field is not syntactically valid, for example `HTTPX`.
    MalformedHttpVersion,
    /// The HTTP version is syntactically valid but unsupported by this parser or protocol implementation.
    UnsupportedHttpVersion,
    /// The request method is not a valid HTTP token.
    InvalidMethodToken,
}

impl fmt::Display for StartLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("start line is empty"),
            Self::Unrecognised => formatter.write_str("start line could not be parsed"),
            Self::InvalidStatusCode => formatter.write_str("response status code is not a number"),
            Self::MalformedHttpVersion => formatter.write_str("HTTP version is malformed"),
            Self::UnsupportedHttpVersion => formatter.write_str("HTTP version is not supported"),
            Self::InvalidMethodToken => {
                formatter.write_str("request method is not a valid HTTP token")
            }
        }
    }
}

impl std::error::Error for StartLineError {}

impl StartLineError {
    /// Start-line failures close the connection after the error response.
    /// The request cannot be dispatched, and retaining the HTTP/1 stream would
    /// make version negotiation and pipelined-request handling ambiguous.
    pub fn can_continue(&self) -> bool {
        false
    }
}

impl From<&StartLineError> for StatusCode {
    fn from(error: &StartLineError) -> Self {
        match error {
            StartLineError::UnsupportedHttpVersion => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
