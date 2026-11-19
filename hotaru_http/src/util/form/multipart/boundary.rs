//! Validated multipart boundary values.

use core::fmt;

use super::error::MultipartError;

/// A multipart boundary satisfying the syntax in RFC 2046 section 5.1.1.
///
/// Construction checks the boundary value, not collisions with part contents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Boundary(String);

impl Boundary {
    /// Validates and stores a multipart boundary value.
    ///
    /// Returns [`MultipartError::EmptyBoundary`] for an empty value, or
    /// [`MultipartError::InvalidData`] for other invalid boundary syntax.
    pub fn new(value: impl Into<String>) -> Result<Self, MultipartError> {
        let value = value.into();
        if value.is_empty() {
            return Err(MultipartError::EmptyBoundary);
        }
        if value.len() > 70 {
            return Err(MultipartError::InvalidData(
                "multipart boundary exceeds 70 characters".to_string(),
            ));
        }
        if value.ends_with(' ') {
            return Err(MultipartError::InvalidData(
                "multipart boundary cannot end with a space".to_string(),
            ));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"'()+_,-./:=? ".contains(&byte))
        {
            return Err(MultipartError::InvalidData(
                "multipart boundary contains an invalid character".to_string(),
            ));
        }

        Ok(Self(value))
    }

    /// Returns the validated boundary value without permitting mutation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Boundary {
    fn default() -> Self {
        Self("----DefaultBoundary7MA4YWxkTrZu0gW".to_string())
    }
}

impl TryFrom<String> for Boundary {
    type Error = MultipartError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Boundary {
    type Error = MultipartError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for Boundary {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Boundary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_boundary() {
        assert_eq!(Boundary::new(""), Err(MultipartError::EmptyBoundary));
    }

    #[test]
    fn accepts_boundary_character_set_and_internal_spaces() {
        let value = "AZaz09'()+_,-./:=? internal-space";

        assert_eq!(Boundary::new(value).unwrap().as_str(), value);
    }

    #[test]
    fn enforces_boundary_length_limit() {
        assert!(Boundary::new("a").is_ok());
        assert!(Boundary::new("a".repeat(70)).is_ok());
        assert!(matches!(
            Boundary::new("a".repeat(71)),
            Err(MultipartError::InvalidData(_))
        ));
    }

    #[test]
    fn rejects_invalid_boundary_characters_and_trailing_space() {
        for value in ["a\r", "a\n", "a\t", "a\0", "a\"b", "a\\b", "a;b", "é", "a "] {
            assert!(matches!(
                Boundary::new(value),
                Err(MultipartError::InvalidData(_))
            ));
        }
    }
}
