use alloc::vec::Vec;
//
pub trait ReadLimitError: core::error::Error + Send + Sync + 'static {
    fn rate_limit_error(data: Vec<u8>) -> Self;

    fn get_read(&self) -> &[u8] {
        &[]
    }
}

#[cfg(feature = "std")]
mod std_impl {
    use super::*;

    #[derive(Debug)]
    struct RateLimitData(Vec<u8>);

    impl core::fmt::Display for RateLimitData {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "read rate limit exceeded: {} bytes read", self.0.len())
        }
    }

    impl core::error::Error for RateLimitData {}

    impl ReadLimitError for std::io::Error {
        fn rate_limit_error(data: Vec<u8>) -> Self {
            std::io::Error::new(std::io::ErrorKind::InvalidData, RateLimitData(data))
        }

        fn get_read(&self) -> &[u8] {
            self.get_ref()
                .and_then(|e| e.downcast_ref::<RateLimitData>())
                .map(|r| r.0.as_slice())
                .unwrap_or(&[])
        }
    }
}

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
    SizeExceeded(Vec<u8>),
}

impl ReadLimitError for HotaruIOError {
    fn rate_limit_error(data: Vec<u8>) -> Self {
        Self::SizeExceeded(data)
    }

    fn get_read(&self) -> &[u8] {
        match self {
            Self::SizeExceeded(data) => data.as_slice(),
            _ => &[],
        }
    }
}

impl core::fmt::Display for HotaruIOError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected EOF before buffer was filled"),
            Self::WriteZero => f.write_str("writer accepted 0 bytes"),
            Self::SizeExceeded(data) => {
                write!(f, "read rate limit exceeded: {} bytes read", data.len())
            }
        }
    }
}

impl core::error::Error for HotaruIOError {}
