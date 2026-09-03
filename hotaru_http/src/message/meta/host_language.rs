use super::HttpMeta;
use crate::message::header::HeaderError;
use crate::message::http_value::AcceptLang;

fn valid_port(port: Option<&str>) -> bool {
    match port {
        None => true,
        Some("") => false,
        Some(port) => port.parse::<u16>().is_ok(),
    }
}

fn is_valid_host_header(host: &str) -> bool {
    if host.is_empty()
        || host
            .chars()
            .any(|c| c.is_ascii_control() || c.is_whitespace())
    {
        return false;
    }

    if let Some(rest) = host.strip_prefix('[') {
        let Some(close) = rest.find(']') else {
            return false;
        };
        let literal = &rest[..close];
        if literal.is_empty()
            || !literal
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || matches!(b, b'.' | b':'))
        {
            return false;
        }

        let suffix = &rest[close + 1..];
        if suffix.is_empty() {
            return true;
        }

        let Some(port) = suffix.strip_prefix(':') else {
            return false;
        };
        return valid_port(Some(port));
    }

    if host
        .bytes()
        .any(|b| matches!(b, b'/' | b'?' | b'#' | b'@' | b'[' | b']'))
    {
        return false;
    }

    let (name, port) = match host.rsplit_once(':') {
        Some((name, port)) => {
            if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) {
                return false;
            }
            (name, Some(port))
        }
        None => (host, None),
    };

    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
        && valid_port(port)
}

impl HttpMeta {
    /// Gets the host from the HTTP meta data.
    ///
    /// Returns the cached host if available, otherwise parses
    /// the host header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The host, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_host(), Some("example.com".to_string()));
    /// ```
    pub fn get_host(&mut self) -> Option<String> {
        if let Some(ref host) = self.host {
            return Some(host.clone());
        }
        self.parse_host()
    }

    /// Parses the Host header from the headers map and stores it in the host field.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The host value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let host = meta.parse_host();
    /// assert_eq!(host, Some("example.com".to_string()));
    /// ```
    pub fn parse_host(&mut self) -> Option<String> {
        let host = self.header.get("host").map(|value| value.first());

        self.set_host(host.clone());
        host
    }

    /// Requires exactly one syntactically valid Host header for HTTP/1.1 requests.
    pub fn require_valid_request_host(&mut self) -> Result<String, HeaderError> {
        let host = self.header.require_only("host")?.to_string();

        if !is_valid_host_header(&host) {
            return Err(HeaderError::InvalidHeaderValue("host".to_string()));
        }

        self.set_host(Some(host.clone()));
        Ok(host)
    }

    /// Sets the host field.
    ///
    /// # Arguments
    ///
    /// * `host` - The host to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_host(Some("example.com".to_string()));
    ///
    /// assert_eq!(meta.get_host(), Some("example.com".to_string()));
    /// ```
    pub fn set_host(&mut self, host: Option<String>) {
        self.host = host;
    }

    /// Clears the cached host field without modifying the header map.
    ///
    /// This method invalidates the cached host value, which will cause
    /// subsequent calls to `get_host()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.,
    /// To remove both the cached field and the header, use `delete_host()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into the cache
    /// let host = meta.get_host();
    /// assert_eq!(host, Some("example.com".to_string()));
    ///
    /// // Clear the cache only
    /// meta.clear_host();
    ///
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_host(), Some("example.com".to_string()));
    /// ```
    pub fn clear_host(&mut self) {
        self.host = None;
    }

    /// Gets the language preference from the HTTP meta data.
    ///
    /// Returns the cached language if available, otherwise parses
    /// the appropriate language header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<AcceptLang>` - The language preference, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::start_line::HttpStartLine;
    /// # use hotaru_http::http_value::*;
    /// use std::collections::HashMap;
    /// let mut headers = HashMap::new();
    /// headers.insert("accept-language".to_string(), HeaderValue::new("en-US, en;q=0.9"));
    /// headers.insert("content-language".to_string(), HeaderValue::new("zh-TW"));
    /// let mut meta = HttpMeta::new(HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".to_string()), headers.clone());
    ///
    /// let lang = meta.get_lang().unwrap();
    /// assert_eq!(lang.most_preferred(), "en-US");
    ///
    /// let mut meta = HttpMeta::new(HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK), headers);
    /// let lang = meta.get_lang().unwrap();
    /// assert_eq!(lang.most_preferred(), "zh-TW");
    /// ```
    pub fn get_lang(&mut self) -> Option<AcceptLang> {
        if let Some(ref lang) = self.lang {
            return Some(lang.clone());
        }
        self.parse_lang()
    }

    /// Parses the language header from the headers map and stores it in the lang field.
    ///
    /// For requests: Parses "accept-language" header
    /// For responses: Parses "content-language" header
    ///
    /// # Returns
    ///
    /// * `Option<AcceptLang>` - The parsed language value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::start_line::HttpStartLine;
    /// # use hotaru_http::http_value::*;
    /// use std::collections::HashMap;
    /// let mut headers = HashMap::new();
    /// headers.insert("accept-language".to_string(), HeaderValue::new("en-US, en;q=0.9"));
    /// headers.insert("content-language".to_string(), HeaderValue::new("zh-TW"));
    /// let mut meta = HttpMeta::new(HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".to_string()), headers.clone());
    ///
    /// let lang = meta.parse_lang().unwrap();
    /// assert_eq!(lang.most_preferred(), "en-US");
    ///
    /// let mut meta = HttpMeta::new(HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK), headers);
    /// let lang = meta.parse_lang().unwrap();
    /// assert_eq!(lang.most_preferred(), "zh-TW");
    /// ```
    pub fn parse_lang(&mut self) -> Option<AcceptLang> {
        let header_name = if self.start_line.is_request() {
            "accept-language"
        } else {
            "content-language"
        };

        let lang_str = self.header.get(header_name).map(|value| value.as_str());

        let lang = lang_str.as_ref().map(|s| AcceptLang::from_str(s));
        self.lang = lang.clone();
        lang
    }

    /// Sets the lang field.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language preference to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::http_value::AcceptLang;
    /// let mut meta = HttpMeta::default();
    /// meta.set_lang(Some(AcceptLang::from_str("en")));
    /// ```
    pub fn set_lang(&mut self, lang: Option<AcceptLang>) {
        self.lang = lang;
    }

    /// Clears the cached lang field without modifying the header map.
    ///
    /// This method invalidates the cached lang value but preserves
    /// the header in the map. Subsequent calls to `get_lang()` will
    /// re-parse the value from headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// let mut meta = HttpMeta::default();
    /// meta.clear_lang();
    /// ```
    pub fn clear_lang(&mut self) {
        self.lang = None;
    }

    /// Deletes the language header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// For requests: Removes "accept-language" header
    /// For responses: Removes "content-language" header
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// let mut meta = HttpMeta::default();
    /// meta.delete_lang();
    /// ```
    pub fn delete_lang(&mut self) {
        self.lang = None;
        if self.start_line.is_request() {
            self.header.remove("accept-language");
        } else {
            self.header.remove("content-language");
        }
    }

    /// Deletes the Host header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the host header from the headers map and
    /// clears the cached host value. Subsequent calls to `get_host()`
    /// will return None unless a new host is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both the cache and header
    /// meta.delete_host();
    ///
    /// // The header is gone
    /// assert!(meta.get_header("host").is_none());
    ///
    /// // And get_host will now return None
    /// assert_eq!(meta.get_host(), None);
    /// ```
    pub fn delete_host(&mut self) {
        self.host = None;
        self.header.remove("host");
    }
}
