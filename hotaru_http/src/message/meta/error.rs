//! Error type for HTTP metadata parsing and framing.

use core::fmt;

use crate::message::header::HeaderError;
use crate::message::http_value::StatusCode;
use crate::message::start_line::StartLineError;
use crate::util::encoding::EncodingError;
use crate::util::streamed::Streamed;

/// Ergonomic alias for the read+parse boundary type at meta-parsing functions.
pub type StreamedMetaError = Streamed<MetaError>;

/// Errors raised while parsing or framing HTTP metadata.
///
/// The enum is `#[non_exhaustive]`: additional variants may be added in
/// future releases without a source-breaking change. Downstream matches
/// must include a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetaError {
    /// A header line exceeded the per-line size limit.
    HeaderLineTooLong,

    /// The total header block exceeded the byte-size limit.
    HeadersTooLarge,

    /// The number of header lines exceeded the configured maximum.
    TooManyHeaders,

    /// A header line was structurally invalid (no `:`, illegal name, etc.).
    InvalidHeader,

    /// Content-Length and Transfer-Encoding both present — smuggling risk.
    ConflictingFraming,

    /// A start-line parse failure.
    StartLine(StartLineError),

    /// A header-level failure (bad value, overflow, unexpected duplicate).
    Header(HeaderError),

    /// A Transfer-Encoding or Content-Encoding header failed validation.
    Encoding(EncodingError),
}

impl fmt::Display for MetaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderLineTooLong => formatter.write_str("header line too long"),
            Self::HeadersTooLarge => formatter.write_str("header block too large"),
            Self::TooManyHeaders => formatter.write_str("too many headers"),
            Self::InvalidHeader => formatter.write_str("invalid header line"),
            Self::ConflictingFraming => {
                formatter.write_str("Content-Length cannot be combined with Transfer-Encoding")
            }
            Self::StartLine(error) => fmt::Display::fmt(error, formatter),
            Self::Header(error) => fmt::Display::fmt(error, formatter),
            Self::Encoding(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for MetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StartLine(error) => Some(error),
            Self::Header(error) => Some(error),
            Self::Encoding(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StartLineError> for MetaError {
    fn from(error: StartLineError) -> Self {
        Self::StartLine(error)
    }
}

impl From<HeaderError> for MetaError {
    fn from(error: HeaderError) -> Self {
        Self::Header(error)
    }
}

impl From<EncodingError> for MetaError {
    fn from(error: EncodingError) -> Self {
        Self::Encoding(error)
    }
}

impl MetaError {
    /// Whether the connection can continue after this error. Header
    /// boundary-loss variants (owned by MetaError) force `false`; wrapped
    /// component variants delegate to the component's own policy.
    pub fn can_continue(&self) -> bool {
        match self {
            Self::StartLine(error) => error.can_continue(),
            Self::Header(error) => error.can_continue(),
            Self::Encoding(error) => error.can_continue(),
            // Header block boundary lost — cannot trust where the next request starts.
            Self::HeaderLineTooLong | Self::HeadersTooLarge | Self::TooManyHeaders => false,
            // Framing ambiguity — smuggling class.
            Self::ConflictingFraming => false,
            Self::InvalidHeader => true,
        }
    }
}

/// Maps a metadata failure to the HTTP status code the aggregate should
/// serve. Wrapped component variants delegate to the component's own
/// mapping; meta-owned variants map inline.
impl From<&MetaError> for StatusCode {
    fn from(error: &MetaError) -> Self {
        match error {
            MetaError::StartLine(error) => StatusCode::from(error),
            MetaError::Header(error) => StatusCode::from(error),
            MetaError::Encoding(error) => StatusCode::from(error),
            MetaError::HeaderLineTooLong
            | MetaError::HeadersTooLarge
            | MetaError::TooManyHeaders => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            MetaError::InvalidHeader | MetaError::ConflictingFraming => StatusCode::BAD_REQUEST,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_variants_delegate_display_to_the_inner_error() {
        assert_eq!(
            MetaError::from(StartLineError::Unrecognised).to_string(),
            "start line could not be parsed",
        );
        assert_eq!(
            MetaError::from(HeaderError::MultipleValues("content-length".to_string())).to_string(),
            "multiple content-length header values",
        );
        assert_eq!(
            MetaError::from(EncodingError::DuplicateChunked).to_string(),
            "`chunked` appeared more than once in Transfer-Encoding",
        );
    }

    #[test]
    fn source_is_present_only_for_wrapping_variants() {
        assert!(
            std::error::Error::source(&MetaError::from(StartLineError::Unrecognised)).is_some()
        );
        assert!(
            std::error::Error::source(&MetaError::from(HeaderError::MultipleValues(
                "content-length".to_string()
            )))
            .is_some()
        );
        assert!(
            std::error::Error::source(&MetaError::from(EncodingError::DuplicateChunked))
                .is_some()
        );
        assert!(std::error::Error::source(&MetaError::HeadersTooLarge).is_none());
    }
}
