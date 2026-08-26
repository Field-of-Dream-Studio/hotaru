//! Errors for HTTP transfer- and content-coding operations.

use core::fmt;

use crate::message::http_value::StatusCode;

/// Errors from parsing or validating a Transfer-Encoding / Content-Encoding header.
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodingError {
    /// `chunked` appeared more than once in Transfer-Encoding.
    DuplicateChunked,
    /// A coding was listed after `chunked` in Transfer-Encoding.
    CodingAfterChunked,
    /// Transfer coding not implemented. Payload is the token from the header.
    UnsupportedTransferCoding(String),
    /// Content coding not implemented. Payload is the token from the header.
    UnsupportedContentCoding(String),
}

/// Errors from applying a content coding to a body payload.
#[derive(Debug)]
#[non_exhaustive]
pub enum CompressionError {
    /// Coding is recognised but not usable in this build — either the
    /// `compression` feature is off, or the coding is one we never implement
    /// (e.g. `compress`).
    Unavailable(&'static str),
    /// Coding token from the wire is not implemented — reached apply-time
    /// because header parsing was lenient. Payload is the token as sent.
    UnsupportedCoding(String),
    /// Compressor failed while encoding.
    EncodeFailed(CompressionFailure),
    /// Decompressor failed while decoding.
    DecodeFailed(CompressionFailure),
    /// Decoded payload would exceed the per-operation size cap.
    DecodedBodyTooLarge,
}

/// Why a compression or decompression operation failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum CompressionFailure {
    /// Input ended before the frame was complete.
    Truncated,
    /// Input bytes are malformed.
    InvalidStream,
    /// Output exceeded the operation limit.
    LimitExceeded,
}

impl fmt::Display for EncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateChunked => {
                formatter.write_str("`chunked` appeared more than once in Transfer-Encoding")
            }
            Self::CodingAfterChunked => {
                formatter.write_str("a coding was listed after `chunked` in Transfer-Encoding")
            }
            Self::UnsupportedTransferCoding(token) => {
                write!(formatter, "unsupported transfer coding: {token}")
            }
            Self::UnsupportedContentCoding(token) => {
                write!(formatter, "unsupported content coding: {token}")
            }
        }
    }
}

impl fmt::Display for CompressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(coding) => {
                write!(formatter, "content coding `{coding}` is unavailable")
            }
            Self::UnsupportedCoding(token) => {
                write!(formatter, "unsupported content coding: {token}")
            }
            Self::EncodeFailed(reason) => {
                write!(formatter, "compression failed while encoding: {reason}")
            }
            Self::DecodeFailed(reason) => {
                write!(formatter, "compression failed while decoding: {reason}")
            }
            Self::DecodedBodyTooLarge => {
                formatter.write_str("decoded payload would exceed the configured size cap")
            }
        }
    }
}

impl fmt::Display for CompressionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => {
                formatter.write_str("input ended before the frame was complete")
            }
            Self::InvalidStream => formatter.write_str("input stream is malformed"),
            Self::LimitExceeded => formatter.write_str("output exceeded the operation limit"),
        }
    }
}

impl std::error::Error for EncodingError {}

impl std::error::Error for CompressionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EncodeFailed(reason) | Self::DecodeFailed(reason) => Some(reason),
            _ => None,
        }
    }
}

impl std::error::Error for CompressionFailure {}

impl EncodingError {
    /// Chunked framing violations force the socket to close (smuggling
    /// class per RFC 9112 §6.1). Other coding rejections are safe to
    /// respond to and keep the socket.
    pub fn can_continue(&self) -> bool {
        match self {
            Self::DuplicateChunked | Self::CodingAfterChunked => false,
            _ => true,
        }
    }
}

impl From<&EncodingError> for StatusCode {
    fn from(error: &EncodingError) -> Self {
        match error {
            EncodingError::UnsupportedTransferCoding(_) => StatusCode::NOT_IMPLEMENTED,
            EncodingError::UnsupportedContentCoding(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl CompressionError {
    /// Compression-apply failures happen after body framing is intact 
    /// respond and keep the socket.
    pub fn can_continue(&self) -> bool {
        true
    }
}

impl From<&CompressionError> for StatusCode {
    fn from(error: &CompressionError) -> Self {
        match error {
            CompressionError::DecodedBodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            CompressionError::Unavailable(_) | CompressionError::UnsupportedCoding(_) => {
                StatusCode::UNSUPPORTED_MEDIA_TYPE
            }
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_propagates_the_wrapped_compression_failure() {
        assert_eq!(
            CompressionError::EncodeFailed(CompressionFailure::Truncated).to_string(),
            "compression failed while encoding: input ended before the frame was complete",
        );
        assert_eq!(
            CompressionError::DecodeFailed(CompressionFailure::InvalidStream).to_string(),
            "compression failed while decoding: input stream is malformed",
        );
    }

    #[test]
    fn source_is_present_only_for_wrapping_variants() {
        assert!(
            std::error::Error::source(&CompressionError::EncodeFailed(
                CompressionFailure::LimitExceeded
            ))
            .is_some()
        );
        assert!(
            std::error::Error::source(&CompressionError::DecodeFailed(
                CompressionFailure::Truncated
            ))
            .is_some()
        );
        assert!(std::error::Error::source(&CompressionError::Unavailable("gzip")).is_none());
        assert!(std::error::Error::source(&CompressionError::DecodedBodyTooLarge).is_none());
    }
}
