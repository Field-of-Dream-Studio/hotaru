//! HTTP-body processing errors.

use core::fmt;

use crate::message::http_value::StatusCode;
use crate::util::encoding::{CompressionError, EncodingError};
use crate::util::streamed::Streamed;

/// Errors raised while decoding a chunked-transfer body at read time.
///
/// The apply-time counterpart of `EncodingError`'s parse-time chunked
/// framing variants (`DuplicateChunked`, `CodingAfterChunked`).
#[derive(Debug)]
#[non_exhaustive]
pub enum ChunkingError {
    /// Chunk size line exceeded the configured per-line limit.
    LineTooLong,
    /// Chunk size was not valid hex (per RFC 9112 §7.1).
    InvalidSize,
    /// Chunk data was not terminated by CRLF.
    InvalidTerminator,
}

impl fmt::Display for ChunkingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LineTooLong => formatter.write_str("chunk size line exceeds maximum length"),
            Self::InvalidSize => formatter.write_str("invalid chunk size"),
            Self::InvalidTerminator => formatter.write_str("invalid chunk terminator"),
        }
    }
}

impl std::error::Error for ChunkingError {}

impl ChunkingError {
    /// Chunked framing violations force the socket to close — once the byte
    /// stream is desynchronised we can no longer trust the next request boundary.
    pub fn can_continue(&self) -> bool {
        false
    }
}

impl From<&ChunkingError> for StatusCode {
    fn from(_: &ChunkingError) -> Self {
        StatusCode::BAD_REQUEST
    }
}

/// Ergonomic alias for the read+parse boundary type at body-parsing functions.
pub type StreamedBodyError = Streamed<BodyError>;

/// An error produced while parsing or extracting an HTTP body.
///
/// Pure semantic type: no `Io` variant. Transport failures at read+parse
/// boundaries live in [`Streamed<BodyError>`](crate::util::streamed::Streamed).
///
/// Variants that carry a `String` payload are placeholders for typed component
/// errors that do not exist in this branch yet:
/// - `InvalidJson` awaits a dedicated `JsonError` type.
/// - `InvalidForm` / `InvalidMultipart` await PR #55, which introduces
///   `UrlEncodedError` / `MultipartError`. After PR #55 merges, these variants
///   should be replaced by `Form(UrlEncodedError)` / `Multipart(MultipartError)`.
#[derive(Debug)]
#[non_exhaustive]
pub enum BodyError {
    /// A required request body was not supplied.
    Missing,
    /// The body exceeds the configured size limit.
    TooLarge,
    /// The request did not include a Content-Type header.
    MissingContentType,
    /// The supplied Content-Type does not match the requested extraction.
    /// Contains the normalized Content-Type received from the request.
    UnsupportedContentType(String),
    /// The body is not valid UTF-8.
    InvalidUtf8,
    /// The body is not valid JSON. Details must not contain body contents.
    // TODO: replace with `Json(JsonError)` once a dedicated `JsonError` type exists.
    InvalidJson(String),
    /// The body is not a valid URL-encoded form. Details must not contain body contents.
    // TODO: replace with `Form(UrlEncodedError)` after PR #55 merges.
    InvalidForm(String),
    /// The body is not valid multipart form data. Details must not contain body contents.
    // TODO: replace with `Multipart(MultipartError)` after PR #55 merges.
    InvalidMultipart(String),
    /// The body ended before the declared representation was complete.
    IncompleteBody,
    /// A Transfer-Encoding / Content-Encoding header failed validation.
    Encoding(EncodingError),
    /// Applying a content coding to the body payload failed.
    Compression(CompressionError),
    /// Chunked transfer-encoding framing was violated at body-read time.
    Chunking(ChunkingError),
}

impl fmt::Display for BodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("request body is missing"),
            Self::TooLarge => formatter.write_str("request body is too large"),
            Self::MissingContentType => formatter.write_str("request body Content-Type is missing"),
            Self::UnsupportedContentType(actual) => {
                write!(formatter, "unsupported request body Content-Type: {actual}")
            }
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
            Self::Encoding(error) => fmt::Display::fmt(error, formatter),
            Self::Compression(error) => fmt::Display::fmt(error, formatter),
            Self::Chunking(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for BodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encoding(error) => Some(error),
            Self::Compression(error) => Some(error),
            Self::Chunking(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EncodingError> for BodyError {
    fn from(error: EncodingError) -> Self {
        Self::Encoding(error)
    }
}

impl From<CompressionError> for BodyError {
    fn from(error: CompressionError) -> Self {
        Self::Compression(error)
    }
}

impl From<ChunkingError> for BodyError {
    fn from(error: ChunkingError) -> Self {
        Self::Chunking(error)
    }
}

impl BodyError {
    /// Whether the connection can continue after this error. Wrapped
    /// component variants delegate; `IncompleteBody` forces `false` because
    /// the reader is mid-stream and next-request boundary is unknown.
    pub fn can_continue(&self) -> bool {
        match self {
            Self::Encoding(error) => error.can_continue(),
            Self::Compression(error) => error.can_continue(),
            Self::Chunking(error) => error.can_continue(),
            // Body too large: RFC recommends closing since the client will keep
            // sending body bytes. IncompleteBody: reader is mid-stream.
            Self::TooLarge | Self::IncompleteBody => false,
            _ => true,
        }
    }
}

/// Maps a body failure to the HTTP status code the aggregate should serve.
/// Wrapped component variants delegate; body-owned variants map inline.
impl From<&BodyError> for StatusCode {
    fn from(error: &BodyError) -> Self {
        match error {
            BodyError::Encoding(error) => StatusCode::from(error),
            BodyError::Compression(error) => StatusCode::from(error),
            BodyError::Chunking(error) => StatusCode::from(error),
            BodyError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            BodyError::MissingContentType | BodyError::UnsupportedContentType(_) => {
                StatusCode::UNSUPPORTED_MEDIA_TYPE
            }
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
