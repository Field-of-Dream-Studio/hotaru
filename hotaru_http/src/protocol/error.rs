use std::fmt;

use hotaru_core::protocol::ProtocolError;

use crate::message::body::BodyError;
use crate::message::header::HeaderError;
use crate::message::http_value::StatusCode;
use crate::message::meta::MetaError;
use crate::message::start_line::StartLineError;
use crate::util::encoding::{CompressionError, EncodingError};
use crate::util::streamed::Streamed;

/// Aggregate HTTP error.
///
/// Wraps component errors (`MetaError`, `BodyError`) and carries only the
/// concerns that belong to no single component (transport, routing, timeout,
/// user-thrown status).
#[derive(Debug)]
#[non_exhaustive]
pub enum HttpError {
    /// A header or framing failure bubbled up from meta parsing.
    Meta(MetaError),
    /// A body-processing failure bubbled up from body parsing or serialization.
    Body(BodyError),
    /// A raw transport failure (usually from a read/write not yet inside a
    /// `Streamed<E>` boundary).
    Io(std::io::Error),

    /// The request's HTTP method is not permitted at the matched route.
    MethodNotAllowed,
    /// No route matched the request path.
    NoRoute(String),
    /// The request exceeded its deadline.
    Timeout,
    /// The handler explicitly returned a status as an error.
    Status(StatusCode),
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meta(error) => fmt::Display::fmt(error, formatter),
            Self::Body(error) => fmt::Display::fmt(error, formatter),
            Self::Io(error) => write!(formatter, "HTTP I/O error: {error}"),
            Self::MethodNotAllowed => formatter.write_str("method not allowed"),
            Self::NoRoute(path) => write!(formatter, "no route matched path: {path}"),
            Self::Timeout => formatter.write_str("request timed out"),
            Self::Status(code) => write!(formatter, "HTTP status error: {code:?}"),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Meta(error) => Some(error),
            Self::Body(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

// ── From: component → HttpError ───────────────────────────────────────

impl From<MetaError> for HttpError {
    fn from(error: MetaError) -> Self {
        Self::Meta(error)
    }
}

impl From<BodyError> for HttpError {
    fn from(error: BodyError) -> Self {
        Self::Body(error)
    }
}

impl From<std::io::Error> for HttpError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<StatusCode> for HttpError {
    fn from(code: StatusCode) -> Self {
        Self::Status(code)
    }
}

// Sub-component conveniences: route to the natural aggregate.

impl From<HeaderError> for HttpError {
    fn from(error: HeaderError) -> Self {
        Self::Meta(MetaError::from(error))
    }
}

impl From<StartLineError> for HttpError {
    fn from(error: StartLineError) -> Self {
        Self::Meta(MetaError::from(error))
    }
}

impl From<EncodingError> for HttpError {
    fn from(error: EncodingError) -> Self {
        Self::Meta(MetaError::from(error))
    }
}

impl From<CompressionError> for HttpError {
    fn from(error: CompressionError) -> Self {
        Self::Body(BodyError::from(error))
    }
}

// ── From: Streamed<E> → HttpError ─────────────────────────────────────
//
// The load-bearing edge: any `Streamed<E>` where `E: Into<HttpError>`
// converts via `?` at the aggregate boundary. Transport (`Streamed::Io`)
// becomes `HttpError::Io`; the domain error takes its natural path.

impl<E> From<Streamed<E>> for HttpError
where
    E: Into<HttpError>,
{
    fn from(streamed: Streamed<E>) -> Self {
        match streamed {
            Streamed::Io(error) => Self::Io(error),
            Streamed::Err(error) => error.into(),
        }
    }
}

// ── ProtocolError: connection lifecycle policy ────────────────────────

impl ProtocolError for HttpError {
    /// Delegates component-level policy to `MetaError` / `BodyError`.
    /// HttpError only decides for its own aggregate-level variants.
    fn can_continue(&self) -> bool {
        match self {
            Self::Meta(error) => error.can_continue(),
            Self::Body(error) => error.can_continue(),
            // Transport is dead.
            Self::Io(_) => false,
            // Everything else: send a response and keep the socket.
            _ => true,
        }
    }
}

// ── HttpError → StatusCode: response mapping ──────────────────────────
//
// Component wrappers delegate; the mapping for each component lives in
// its own error file. HttpError only owns the mapping for its own
// aggregate-level variants (transport, routing, timeout, user-thrown status).

impl From<&HttpError> for StatusCode {
    fn from(error: &HttpError) -> Self {
        match error {
            HttpError::Meta(meta) => StatusCode::from(meta),
            HttpError::Body(body) => StatusCode::from(body),
            HttpError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            HttpError::NoRoute(_) => StatusCode::NOT_FOUND,
            HttpError::Timeout => StatusCode::REQUEST_TIMEOUT,
            HttpError::Status(code) => code.clone(),
        }
    }
}
