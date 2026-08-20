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

/// Errors raised while reading, parsing, or framing HTTP metadata.
///
/// Variants that describe a specific header carry the header name as a
/// `&'static str`. The name is always a compile-time constant chosen by the
/// parser (for example `"content-length"`) — never a value read from the
/// wire — so the payload cannot leak untrusted request bytes.
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

    /// A header value failed validation for its declared type.
    /// Payload is the header name.
    InvalidHeaderValue(&'static str),

    /// A numeric header value parsed but overflowed its target type.
    /// Payload is the header name.
    HeaderValueOverflow(&'static str),

    /// A header appeared more than once when it must appear at most once.
    /// Payload is the header name.
    MultipleValues(&'static str),
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

impl std::error::Error for MetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MetaError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
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
    fn preserves_io_error_during_round_trip() {
        let inner = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read");
        let error = std::io::Error::from(MetaError::from(inner));

        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
        assert_eq!(error.to_string(), "short read");
    }

    #[test]
    fn converts_non_io_variant_to_invalid_data() {
        let error = std::io::Error::from(MetaError::EmptyMessage);

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "empty HTTP message");
    }

    #[test]
    fn source_is_present_for_io_variant() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        let error = MetaError::from(inner);

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn source_is_absent_for_non_io_variants() {
        assert!(std::error::Error::source(&MetaError::EmptyMessage).is_none());
        assert!(std::error::Error::source(&MetaError::InvalidStartLine).is_none());
        assert!(
            std::error::Error::source(&MetaError::InvalidHeaderValue("content-length")).is_none()
        );
    }

    #[test]
    fn display_includes_header_name() {
        assert_eq!(
            MetaError::InvalidHeaderValue("content-length").to_string(),
            "invalid content-length header value",
        );
        assert_eq!(
            MetaError::HeaderValueOverflow("content-length").to_string(),
            "content-length header value overflowed",
        );
        assert_eq!(
            MetaError::MultipleValues("content-length").to_string(),
            "multiple content-length header values",
        );
    }

    #[test]
    fn display_for_size_and_framing_variants() {
        assert_eq!(
            MetaError::HeaderLineTooLong.to_string(),
            "header line too long",
        );
        assert_eq!(
            MetaError::HeadersTooLarge.to_string(),
            "header block too large",
        );
        assert_eq!(MetaError::TooManyHeaders.to_string(), "too many headers");
        assert_eq!(MetaError::InvalidHeader.to_string(), "invalid header line");
    }
}
