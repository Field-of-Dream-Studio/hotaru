//! Boundary error for read-and-parse operations: transport failed, or the
//! domain layer rejected the bytes.

/// Two-armed error: `Io` for transport, `Err` for the domain error `E`.
#[derive(Debug)]
pub enum Streamed<E> {
    /// A read from the underlying reader failed.
    Io(std::io::Error),
    /// The read succeeded but the domain layer rejected the bytes.
    Err(E),
}

impl<E> From<std::io::Error> for Streamed<E> {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl<E: core::fmt::Display> core::fmt::Display for Streamed<E> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "transport read failed: {error}"),
            Self::Err(error) => core::fmt::Display::fmt(error, formatter),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for Streamed<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Err(error) => Some(error),
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DomainError;

    impl core::fmt::Display for DomainError {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str("domain rejected the bytes")
        }
    }

    impl std::error::Error for DomainError {}

    #[test]
    fn question_mark_lifts_io_errors_from_read_calls() {
        fn read() -> Result<(), std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "boom"))
        }

        fn compose() -> Result<(), Streamed<DomainError>> {
            read()?;
            Ok(())
        }

        assert!(matches!(compose(), Err(Streamed::Io(_))));
    }

    #[test]
    fn display_delegates_verbatim_to_the_domain_error() {
        let streamed: Streamed<DomainError> = Streamed::Err(DomainError);

        assert_eq!(streamed.to_string(), "domain rejected the bytes");
    }

    #[test]
    fn source_returns_the_underlying_error_from_both_arms() {
        let io_arm: Streamed<DomainError> =
            Streamed::Io(std::io::Error::new(std::io::ErrorKind::Other, "boom"));
        assert!(std::error::Error::source(&io_arm).is_some());

        let domain_arm: Streamed<DomainError> = Streamed::Err(DomainError);
        assert!(std::error::Error::source(&domain_arm).is_some());
    }
}
