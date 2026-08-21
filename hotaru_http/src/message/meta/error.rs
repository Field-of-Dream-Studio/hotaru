//! Error type for HTTP metadata parsing and framing.
//!
//! Provides [`MetaError`], the typed component error raised while reading,
//! parsing, or validating the start line and header block of an HTTP message.
//!
//! The type is defined here but not yet returned from any parser call site:
//! `stream.rs`, `content_length.rs`, and the other split modules continue to
//! return `ConnectionError` in their public signatures. Threading `MetaError`
//! through those signatures is a separate, source-breaking step tracked in
//! `ISSUE_HTTP_MESSAGE_MODULES_AND_ERRORS.md`.

use core::fmt;

use crate::message::header::HeaderError;

/// Errors raised while reading, parsing, or framing HTTP metadata.
///
/// The enum is `#[non_exhaustive]`: additional variants may be added in
/// future releases without a source-breaking change. Downstream matches
/// must include a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetaError {
    /// Underlying I/O failure while reading the header block.
    Io(std::io::Error),

    /// Peer closed the connection before any request bytes arrived.
    EmptyMessage,

    /// Start line was malformed (method, request target, or version).
    InvalidStartLine,

    /// A header line exceeded the per-line size limit.
    HeaderLineTooLong,

    /// The total header block exceeded the byte-size limit.
    HeadersTooLarge,

    /// The number of header lines exceeded the configured maximum.
    TooManyHeaders,

    /// A header line was structurally invalid (no `:`, illegal name, etc.).
    InvalidHeader,

    /// A header-level failure (bad value, overflow, unexpected duplicate).
    Header(HeaderError),
}

impl fmt::Display for MetaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error while reading headers: {error}"),
            Self::EmptyMessage => formatter.write_str("empty HTTP message"),
            Self::InvalidStartLine => formatter.write_str("malformed start line"),
            Self::HeaderLineTooLong => formatter.write_str("header line too long"),
            Self::HeadersTooLarge => formatter.write_str("header block too large"),
            Self::TooManyHeaders => formatter.write_str("too many headers"),
            Self::InvalidHeader => formatter.write_str("invalid header line"),
            Self::Header(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for MetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Header(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MetaError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<HeaderError> for MetaError {
    fn from(error: HeaderError) -> Self {
        Self::Header(error)
    }
}

impl From<MetaError> for std::io::Error {
    fn from(error: MetaError) -> Self {
        match error {
            MetaError::Io(error) => error,
            error => Self::new(std::io::ErrorKind::InvalidData, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_variant_round_trips_through_io_error() {
        let inner = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read");
        let error = std::io::Error::from(MetaError::from(inner));

        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
        assert_eq!(error.to_string(), "short read");
    }

    #[test]
    fn non_io_variant_becomes_invalid_data() {
        let error = std::io::Error::from(MetaError::EmptyMessage);

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "empty HTTP message");
    }

    #[test]
    fn header_error_wraps_and_display_delegates() {
        let error = MetaError::from(HeaderError::MultipleValues("content-length"));

        assert!(matches!(&error, MetaError::Header(HeaderError::MultipleValues(_))));
        assert_eq!(error.to_string(), "multiple content-length header values");
    }

    #[test]
    fn source_present_for_wrapping_variants() {
        let io_error = MetaError::from(std::io::Error::new(std::io::ErrorKind::Other, "boom"));
        assert!(std::error::Error::source(&io_error).is_some());

        let header_error = MetaError::from(HeaderError::MultipleValues("content-length"));
        assert!(std::error::Error::source(&header_error).is_some());

        assert!(std::error::Error::source(&MetaError::EmptyMessage).is_none());
    }
}
