//! Raw HTTP header field-line parsing.

use super::error::{HeaderError, HeaderLineError};
use super::map::HeaderMap;
use super::value::HeaderValue;

impl HeaderMap {
    /// Parses and inserts one field line, appending to existing values with
    /// the same field name without combining them into a comma-delimited string.
    pub fn insert_field_line(&mut self, line: &str) -> Result<(), HeaderError> {
        let (name, value) = Self::parse_field_line(line)?;

        if let Some(mut existing_value) = self.remove(name.as_str()) {
            existing_value.add_without_combining(value);
            self.insert(name, existing_value);
        } else {
            self.insert(name, HeaderValue::new(value));
        }

        Ok(())
    }

    fn parse_field_line(line: &str) -> Result<(String, String), HeaderLineError> {
        let colon_pos = line.find(':').ok_or(HeaderLineError::MissingColon)?;
        let name = &line[..colon_pos];
        let value = &line[colon_pos + 1..];

        if name.is_empty() {
            return Err(HeaderLineError::EmptyName);
        }

        if !name.as_bytes().iter().all(|byte| Self::is_tchar(*byte)) {
            return Err(HeaderLineError::InvalidName);
        }

        if value.as_bytes().iter().any(|byte| {
            matches!(*byte, b'\r' | b'\n' | b'\0' | 0x7f)
                || (*byte < 0x20 && *byte != b'\t')
        }) {
            return Err(HeaderLineError::InvalidValue);
        }

        Ok((
            name.to_ascii_lowercase(),
            value
                .trim_matches(|char| char == ' ' || char == '\t')
                .to_string(),
        ))
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
}
