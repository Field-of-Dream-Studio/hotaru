//! Errors for HTTP header value validation and uniqueness.

use core::fmt;

use crate::message::http_value::StatusCode;

/// Errors raised while parsing one raw HTTP header field line.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderLineError {
    /// The field line did not contain a colon separator.
    MissingColon,
    /// The field name before the colon was empty.
    EmptyName,
    /// The field name was not a valid HTTP token.
    InvalidName,
    /// The field value contained a prohibited control byte.
    InvalidValue,
}

impl fmt::Display for HeaderLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon => formatter.write_str("header line is missing ':'"),
            Self::EmptyName => formatter.write_str("header field name is empty"),
            Self::InvalidName => formatter.write_str("header field name is invalid"),
            Self::InvalidValue => formatter.write_str("header field value is invalid"),
        }
    }
}

impl std::error::Error for HeaderLineError {}

/// Errors raised while validating a header value or its uniqueness.
///
/// The name payload is a `String`; sanitize before surfacing error text to
/// external systems as it may carry wire-supplied bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderError {
    /// A raw header field-line failed syntax validation.
    ParseError(HeaderLineError),
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
            Self::ParseError(error) => fmt::Display::fmt(error, formatter),
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

impl std::error::Error for HeaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<HeaderLineError> for HeaderError {
    fn from(error: HeaderLineError) -> Self {
        Self::ParseError(error)
    }
}

impl HeaderError {
    /// Header field-line parse failures fail closed. Header-value failures
    /// don't lose reader sync — the socket can carry on. Both map to 400.
    pub fn can_continue(&self) -> bool {
        !matches!(self, Self::ParseError(_))
    }
}

impl From<&HeaderError> for StatusCode {
    fn from(_: &HeaderError) -> Self {
        StatusCode::BAD_REQUEST
    }
}
