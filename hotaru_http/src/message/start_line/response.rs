use crate::message::http_value::*;

use super::error::StartLineError;

/// ResponseStartLine is the first line of the HTTP response, which contains the HTTP version and status code.
#[derive(Debug, Clone)]
pub struct ResponseStartLine {
    pub http_version: HttpVersion,
    pub status_code: StatusCode,
}

impl ResponseStartLine {
    /// Creates a new HTTP response start line.
    ///
    /// # Arguments
    ///
    /// * `http_version` - The HTTP version.
    /// * `status_code` - The response status code.
    ///
    /// # Returns
    ///
    /// A new `ResponseStartLine` object.
    pub fn new(http_version: HttpVersion, status_code: StatusCode) -> Self {
        Self {
            http_version,
            status_code,
        }
    }

    /// Parses a string into a response start line.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the response start line.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` on success, or a [`StartLineError`] describing the failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::start_line::ResponseStartLine;
    /// let response_line = "HTTP/1.1 200 OK";
    /// let start_line = ResponseStartLine::parse(response_line).unwrap();
    /// println!("{}", start_line);
    /// ```
    pub fn parse<T: AsRef<str>>(line: T) -> Result<Self, StartLineError> {
        let line = line.as_ref();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Err(StartLineError::Empty);
        }
        if parts.len() < 2 {
            return Err(StartLineError::Unrecognised);
        }

        let http_version = HttpVersion::from_string(parts[0]);

        let status_code = match parts[1].parse::<u16>() {
            Ok(code) => StatusCode::from(code),
            Err(_) => return Err(StartLineError::InvalidStatusCode),
        };

        Ok(Self::new(http_version, status_code))
    }

    /// Returns a string representation of the response start line.
    ///
    /// # Returns
    ///
    /// A string representation of the ResponseStartLine.
    pub fn represent(&self) -> String {
        format!(
            "{} {}",
            self.http_version.to_string(),
            self.status_code.to_string()
        )
    }
}

impl std::fmt::Display for ResponseStartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.http_version.to_string(),
            self.status_code.to_string()
        )
    }
}

impl Default for ResponseStartLine {
    fn default() -> Self {
        Self::new(HttpVersion::Http11, StatusCode::OK)
    }
}
