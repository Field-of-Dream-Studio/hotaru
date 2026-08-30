//! Parser for one HTTP header field line.

use core::fmt;

/// Errors raised while parsing one raw HTTP header field line.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderLineError {
    /// The field line did not contain a colon separator.
    MissingColon,
    /// The field name before the colon was empty.
    EmptyName,
    /// The field name was not a valid HTTP token.
    InvalidName,
    /// The field value contained a prohibited control byte.
    InvalidValue,
}

impl fmt::Display for HeaderLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon => formatter.write_str("header line is missing ':'"),
            Self::EmptyName => formatter.write_str("header field name is empty"),
            Self::InvalidName => formatter.write_str("header field name is invalid"),
            Self::InvalidValue => formatter.write_str("header field value is invalid"),
        }
    }
}

impl std::error::Error for HeaderLineError {}

/// One parsed HTTP header field line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderLine {
    name: String,
    value: String,
}

impl HeaderLine {
    pub fn parse(line: &str) -> Result<Self, HeaderLineError> {
        let colon_pos = line.find(':').ok_or(HeaderLineError::MissingColon)?;
        let name = &line[..colon_pos];
        let value = &line[colon_pos + 1..];

        if name.is_empty() {
            return Err(HeaderLineError::EmptyName);
        }

        if !name.as_bytes().iter().all(|byte| is_tchar(*byte)) {
            return Err(HeaderLineError::InvalidName);
        }

        if value.as_bytes().iter().any(|byte| {
            matches!(*byte, b'\r' | b'\n' | b'\0' | 0x7f) || (*byte < 0x20 && *byte != b'\t')
        }) {
            return Err(HeaderLineError::InvalidValue);
        }

        Ok(Self {
            name: name.to_ascii_lowercase(),
            value: value
                .trim_matches(|char| char == ' ' || char == '\t')
                .to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn into_parts(self) -> (String, String) {
        (self.name, self.value)
    }
}

fn is_tchar(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
            | b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_header_line() {
        let line = HeaderLine::parse("X-Test: v").unwrap();

        assert_eq!(line.name(), "x-test");
        assert_eq!(line.value(), "v");
    }

    #[test]
    fn trims_only_ows_around_value() {
        let line = HeaderLine::parse("X-Test:\t value \t").unwrap();

        assert_eq!(
            line.into_parts(),
            ("x-test".to_string(), "value".to_string())
        );
    }

    #[test]
    fn rejects_cr_in_value() {
        assert_eq!(
            HeaderLine::parse("X-Test: a\rb"),
            Err(HeaderLineError::InvalidValue)
        );
    }

    #[test]
    fn rejects_nul_in_value() {
        assert_eq!(
            HeaderLine::parse("X-Test: a\0b"),
            Err(HeaderLineError::InvalidValue)
        );
    }

    #[test]
    fn rejects_whitespace_before_colon() {
        assert_eq!(
            HeaderLine::parse("X-Test : v"),
            Err(HeaderLineError::InvalidName)
        );
    }

    #[test]
    fn rejects_missing_colon() {
        assert_eq!(
            HeaderLine::parse("X-Test v"),
            Err(HeaderLineError::MissingColon)
        );
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(HeaderLine::parse(": v"), Err(HeaderLineError::EmptyName));
    }

    #[test]
    fn allows_obs_text_in_value() {
        let line = HeaderLine::parse("X-Test: cafe\u{00e9}").unwrap();

        assert_eq!(line.value(), "cafe\u{00e9}");
    }
}
