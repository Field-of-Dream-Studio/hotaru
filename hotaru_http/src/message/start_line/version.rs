use crate::message::http_value::HttpVersion;

use super::StartLineError;

/// Parse the HTTP-version token used by this crate's HTTP/1 wire protocol.
///
/// RFC 9110 defines the token as case-sensitive `HTTP/` followed by one digit,
/// a dot, and one digit. This HTTP/1 implementation accepts 1.0 and 1.1 on the
/// wire; other syntactically valid versions are rejected as unsupported.
pub(super) fn parse_http1_version(token: &str) -> Result<HttpVersion, StartLineError> {
    let bytes = token.as_bytes();
    let valid_syntax = bytes.len() == 8
        && bytes.starts_with(b"HTTP/")
        && bytes[5].is_ascii_digit()
        && bytes[6] == b'.'
        && bytes[7].is_ascii_digit();

    if !valid_syntax {
        return Err(StartLineError::InvalidHttpVersion(token.to_string()));
    }

    match HttpVersion::from_string(token) {
        version @ (HttpVersion::Http10 | HttpVersion::Http11) => Ok(version),
        _ => Err(StartLineError::UnsupportedHttpVersion(token.to_string())),
    }
}
