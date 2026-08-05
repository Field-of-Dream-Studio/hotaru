//! Request-body extraction errors.

use std::fmt;

/// An error produced while decoding, parsing, or extracting an HTTP body.
///
/// This type describes the body failure itself. It does not choose a response
/// format, allowing applications to decide how errors are presented.
#[derive(Debug)]
pub enum BodyError {
    /// A required request body was not supplied.
    Missing,
    /// The body exceeds the configured size limit.
    TooLarge {
        /// Maximum accepted size in bytes.
        limit: usize,
        /// Observed size in bytes, when it is known.
        actual: Option<usize>,
    },
    /// The request did not include a Content-Type header.
    MissingContentType,
    /// The supplied Content-Type does not match the requested extraction.
    /// Contains the normalized Content-Type received from the request.
    UnsupportedContentType(String),
    /// The request uses a content encoding that Hotaru cannot decode.
    UnsupportedContentEncoding(String),
    /// The body could not be decoded using its declared encoding.
    InvalidEncoding,
    /// The body is not valid UTF-8.
    InvalidUtf8,
    /// The body is not valid JSON. Details must not contain body contents.
    InvalidJson(String),
    /// The body is not a valid URL-encoded form. Details must not contain body contents.
    InvalidForm(String),
    /// The body is not valid multipart form data. Details must not contain body contents.
    InvalidMultipart(String),
    /// The body ended before the declared representation was complete.
    IncompleteBody,
    /// An I/O failure occurred while reading the body.
    Io(std::io::Error),
}

impl fmt::Display for BodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("request body is missing"),
            Self::TooLarge {
                limit,
                actual: Some(actual),
            } => write!(
                formatter,
                "request body is too large (limit: {limit} bytes, actual: {actual} bytes)"
            ),
            Self::TooLarge {
                limit,
                actual: None,
            } => write!(
                formatter,
                "request body is too large (limit: {limit} bytes)"
            ),
            Self::MissingContentType => formatter.write_str("request body Content-Type is missing"),
            Self::UnsupportedContentType(actual) => {
                write!(formatter, "unsupported request body Content-Type: {actual}")
            }
            Self::UnsupportedContentEncoding(encoding) => write!(
                formatter,
                "unsupported request body Content-Encoding: {encoding}"
            ),
            Self::InvalidEncoding => formatter.write_str("request body encoding is invalid"),
            Self::InvalidUtf8 => formatter.write_str("request body is not valid UTF-8"),
            Self::InvalidJson(details) => {
                write!(formatter, "request body contains invalid JSON: {details}")
            }
            Self::InvalidForm(details) => write!(
                formatter,
                "request body contains an invalid form: {details}"
            ),
            Self::InvalidMultipart(details) => write!(
                formatter,
                "request body contains invalid multipart data: {details}"
            ),
            Self::IncompleteBody => formatter.write_str("request body is incomplete"),
            Self::Io(error) => write!(formatter, "request body I/O error: {error}"),
        }
    }
}

impl std::error::Error for BodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BodyError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
