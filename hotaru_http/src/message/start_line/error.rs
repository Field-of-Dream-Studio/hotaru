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
}

impl fmt::Display for StartLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("start line is empty"),
            Self::Unrecognised => formatter.write_str("start line could not be parsed"),
            Self::InvalidStatusCode => {
                formatter.write_str("response status code is not a number")
            }
        }
    }
}

impl std::error::Error for StartLineError {}

impl StartLineError {
    /// A malformed start line means the request boundary is intact but the
    /// contents are garbage — respond with 400 and keep the socket.
    pub fn can_continue(&self) -> bool {
        true
    }
}

impl From<&StartLineError> for StatusCode {
    fn from(_: &StartLineError) -> Self {
        StatusCode::BAD_REQUEST
    }
}
