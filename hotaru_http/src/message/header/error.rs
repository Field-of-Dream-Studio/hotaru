//! Errors for HTTP header value validation and uniqueness.

use core::fmt;

/// Errors raised while validating a single header value or the uniqueness of a
/// header name.
///
/// Variants that carry a header name use `&'static str` on purpose: the name
/// is always a compile-time constant chosen by the parser
/// (e.g. `"content-length"`), never a value read from the wire.
#[derive(Debug)]
#[non_exhaustive]
pub enum HeaderError {
    /// A header value failed validation for its declared type.
    InvalidHeaderValue(&'static str),
    /// A numeric header value parsed but overflowed its target type.
    HeaderValueOverflow(&'static str),
    /// A header that must appear at most once appeared more than once.
    MultipleValues(&'static str),
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
        }
    }
}

impl std::error::Error for HeaderError {}
