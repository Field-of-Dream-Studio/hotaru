use crate::message::http_value::*;

use super::error::StartLineError;

/// RequestStartLine is the first line of the HTTP request, which contains the method, path, and HTTP version.
#[derive(Debug, Clone)]
pub struct RequestStartLine {
    pub http_version: HttpVersion,
    pub method: HttpMethod,
    pub path: String,
    pub url: Option<RequestPath>,
}

impl RequestStartLine {
    /// Creates a new RequestStartLine object.
    ///
    /// # Arguments
    ///
    /// * `http_version` - The HTTP version.
    /// * `method` - The HTTP method.
    /// * `path` - The request path.
    ///
    /// # Returns
    ///
    /// A new `RequestStartLine` object.
    pub fn new(http_version: HttpVersion, method: HttpMethod, path: String) -> Self {
        Self {
            http_version,
            method,
            path,
            url: None,
        }
    }

    /// Converts the RequestStartLine object to a string.
    ///
    /// # Returns
    ///
    /// A string representation of the RequestStartLine.
    pub fn represent(&self) -> String {
        format!(
            "{} {} {}",
            self.method.to_string(),
            self.path,
            self.http_version.to_string(),
        )
    }

    /// Parses a string into a RequestStartLine object.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that contains the request line.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` on success, or a [`StartLineError`] describing the failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::start_line::RequestStartLine;
    /// let request_line = "GET /index.html HTTP/1.1";
    /// let start_line = RequestStartLine::parse(request_line).unwrap();
    /// println!("{}", start_line);
    /// ```
    pub fn parse<T: AsRef<str>>(line: T) -> Result<Self, StartLineError> {
        let line = line.as_ref();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Err(StartLineError::Empty);
        }
        if parts.len() != 3 {
            return Err(StartLineError::Unrecognised);
        }

        let method = HttpMethod::parse(parts[0])?;
        let path = parts[1].to_string();
        let http_version = HttpVersion::parse(parts[2])?;

        Ok(Self::new(http_version, method, path))
    }

    /// Gets the parsed URL, parsing it if not already present.
    ///
    /// # Returns
    ///
    /// The parsed RequestPath.
    pub fn get_url(&mut self) -> RequestPath {
        match &self.url {
            Some(url) => return url.clone(),
            None => self.parse_url(),
        }
    }

    /// Parses the URL from the path.
    ///
    /// # Returns
    ///
    /// The parsed RequestPath.
    pub fn parse_url(&mut self) -> RequestPath {
        let url = RequestPath::from_string(&self.path);
        self.url = Some(url.clone());
        url
    }

    /// Sets the parsed URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The RequestPath to set.
    pub fn set_url(&mut self, url: RequestPath) {
        self.url = Some(url);
    }

    /// Clears the parsed URL.
    pub fn clear_url(&mut self) {
        self.url = None;
    }
}

impl std::fmt::Display for RequestStartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.path, self.http_version)
    }
}

impl Default for RequestStartLine {
    fn default() -> Self {
        Self::new(HttpVersion::Http11, HttpMethod::GET, "/".to_string())
    }
}
