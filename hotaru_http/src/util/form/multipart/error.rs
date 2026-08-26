//! Typed errors for multipart form parsing.

use core::fmt;

use crate::message::http_value::StatusCode;

/// An error produced while parsing a multipart form body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultipartError {
    /// The multipart boundary is missing or empty.
    EmptyBoundary,
    /// The opening boundary delimiter was not found in the body.
    MissingBoundary,
    /// A multipart section is truncated or missing the closing boundary delimiter.
    IncompleteSection,
    /// A multipart part is missing its header section (no `\r\n\r\n` separator).
    MissingHeaders,
    /// A multipart part header section contains invalid UTF-8 or malformed headers.
    InvalidHeaders,
    /// A multipart part is missing the required `Content-Disposition` header.
    MissingContentDisposition,
    /// A `Content-Disposition` header is missing the required `name` parameter.
    MissingFieldName,
    /// A text field contains invalid UTF-8 data.
    InvalidUtf8,
    /// Malformed or invalid multipart data.
    InvalidData(String),
}

impl fmt::Display for MultipartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBoundary => formatter.write_str("multipart boundary cannot be empty"),
            Self::MissingBoundary => {
                formatter.write_str("multipart boundary delimiter not found in body")
            }
            Self::IncompleteSection => {
                formatter.write_str("multipart section is incomplete or truncated")
            }
            Self::MissingHeaders => {
                formatter.write_str("multipart part is missing header section")
            }
            Self::InvalidHeaders => {
                formatter.write_str("multipart part contains invalid headers")
            }
            Self::MissingContentDisposition => {
                formatter.write_str("multipart part is missing Content-Disposition header")
            }
            Self::MissingFieldName => {
                formatter.write_str("Content-Disposition header is missing name parameter")
            }
            Self::InvalidUtf8 => {
                formatter.write_str("multipart text field contains invalid UTF-8")
            }
            Self::InvalidData(details) => {
                write!(formatter, "invalid multipart data: {details}")
            }
        }
    }
}

impl core::error::Error for MultipartError {}

impl MultipartError {
    /// Body-content parse failures don't lose reader sync — keep the socket.
    pub fn can_continue(&self) -> bool {
        true
    }
}

impl From<&MultipartError> for StatusCode {
    fn from(_: &MultipartError) -> Self {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiFormFieldError {
    ContentTypeError,
    NoFile,
}

impl core::error::Error for MultiFormFieldError {}

impl fmt::Display for MultiFormFieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentTypeError => formatter.write_str("Wrong Content Type!"),
            Self::NoFile => formatter.write_str("404 Not Found!"),
        }
    }
}
