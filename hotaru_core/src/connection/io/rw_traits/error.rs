//
pub trait ReadLimitError: core::error::Error + Send + Sync + 'static {
    fn rate_limit_error() -> Self;
}

// ============================================================================
// std::io::Error — tokio / futures / tls backends
// ============================================================================

#[cfg(feature = "std")]
mod std_impl {
    use super::*;

    impl ReadLimitError for std::io::Error {
        fn rate_limit_error() -> Self {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "read rate limit exceeded")
        }
    }
}

// ============================================================================
// HotaruIOError — framework-owned error type
// ============================================================================

/// Framework-owned IO error. Concrete, non-generic, and identical
/// across feature sets. It starts with only the framework-level sentinel
/// conditions Hotaru itself needs to manufacture.
///
/// Do **not** add a catch-all backend variant up front. When a concrete impl
/// needs to surface another backend failure, add a concrete variant and the
/// corresponding conversion at that impl point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotaruIOError {
    /// Reader returned 0 before `read_exact` filled its buffer.
    UnexpectedEof,
    /// Writer accepted 0 bytes before `write_all` drained its buffer.
    WriteZero,
    /// `read_until` / `read_line` hit the rate limit.
    SizeExceeded,
}

impl ReadLimitError for HotaruIOError {
    fn rate_limit_error() -> Self {
        Self::SizeExceeded
    }
}

impl core::fmt::Display for HotaruIOError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected EOF before buffer was filled"),
            Self::WriteZero => f.write_str("writer accepted 0 bytes"),
            Self::SizeExceeded => f.write_str("read rate limit exceeded"),
        }
    }
}

impl core::error::Error for HotaruIOError {}
