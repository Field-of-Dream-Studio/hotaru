//! Errors for HTTP transfer- and content-coding operations.

use core::fmt;

/// Errors raised while parsing, validating, or applying a coding.
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
    /// Coding is known but the `compression` feature is off in this build.
    CompressionUnavailable(&'static str),
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
            Self::CompressionUnavailable(coding) => {
                write!(
                    formatter,
                    "content coding `{coding}` is unavailable in this build"
                )
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

impl std::error::Error for EncodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EncodeFailed(reason) | Self::DecodeFailed(reason) => Some(reason),
            _ => None,
        }
    }
}

impl std::error::Error for CompressionFailure {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_propagates_the_wrapped_compression_failure() {
        assert_eq!(
            EncodingError::EncodeFailed(CompressionFailure::Truncated).to_string(),
            "compression failed while encoding: input ended before the frame was complete",
        );
        assert_eq!(
            EncodingError::DecodeFailed(CompressionFailure::InvalidStream).to_string(),
            "compression failed while decoding: input stream is malformed",
        );
    }

    #[test]
    fn source_is_present_only_for_wrapping_variants() {
        assert!(
            std::error::Error::source(&EncodingError::EncodeFailed(
                CompressionFailure::LimitExceeded
            ))
            .is_some()
        );
        assert!(
            std::error::Error::source(&EncodingError::DecodeFailed(CompressionFailure::Truncated))
                .is_some()
        );
        assert!(std::error::Error::source(&EncodingError::DuplicateChunked).is_none());
        assert!(std::error::Error::source(&EncodingError::DecodedBodyTooLarge).is_none());
    }
}
