//! Errors for HTTP header value validation and uniqueness.

use core::fmt;

use crate::message::http_value::StatusCode;

/// Errors raised while validating a header value or its uniqueness.
///
/// The name payload is a `String`; sanitize before surfacing error text to
/// external systems as it may carry wire-supplied bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderError {
    /// A header value failed validation for its declared type.
    InvalidHeaderValue(String),
    /// A numeric header value parsed but overflowed its target type.
    HeaderValueOverflow(String),
    /// A header that must appear at most once appeared more than once.
    MultipleValues(String),
    /// A header that was required by the caller was not present.
    Missing(String),
}

impl fmt::Display for HeaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeaderValue(name) => {
                write!(formatter, "invalid {name} header value")
            }
            Self::HeaderValueOverflow(name) => {
                write!(formatter, "{name} header value overflowed")
            }
            Self::MultipleValues(name) => {
                write!(formatter, "multiple {name} header values")
            }
            Self::Missing(name) => {
                write!(formatter, "required {name} header is missing")
            }
        }
    }
}

impl std::error::Error for HeaderError {}

impl HeaderError {
    /// Header-value failures don't lose reader sync — the socket can carry
    /// on. The response is still a 400.
    pub fn can_continue(&self) -> bool {
        true
    }
}

impl From<&HeaderError> for StatusCode {
    fn from(_: &HeaderError) -> Self {
        StatusCode::BAD_REQUEST
    }
}
