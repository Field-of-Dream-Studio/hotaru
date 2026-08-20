use super::{HeaderValue, HttpMeta};
use crate::util::cookie::{Cookie, CookieMap};

impl HttpMeta {
    /// Gets the cookies from the HTTP meta data.
    ///
    /// Returns the cached cookies if available, otherwise parses
    /// the cookie header from the headers map.
    ///
    /// # Returns
    ///
    /// A reference to the cookies map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// # use hotaru_http::cookie::{Cookie, CookieMap};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().get_value(), "abc123");
    /// assert_eq!(cookies.get("theme").unwrap().get_value(), "dark");
    /// ```
    pub fn get_cookies(&mut self) -> &CookieMap {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        // Safety: unwrap() is safe here because we ensure cookies is Some() in the lines above.
        // This pattern guarantees cookies.is_some() before calling unwrap().
        self.cookies.as_ref().unwrap()
    }

    /// Gets a specific cookie by key.
    ///
    /// If the cookies haven't been parsed yet, parses them from the headers map.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// * `Option<Cookie>` - The cookie if found, or None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let session_cookie = meta.get_cookie("sessionId");
    /// let theme_cookie = meta.get_cookie("theme");
    /// assert_eq!(session_cookie.unwrap().get_value(), "abc123");
    /// assert_eq!(theme_cookie.unwrap().get_value(), "dark");
    /// ```
    pub fn get_cookie<T: AsRef<str>>(&mut self, key: T) -> Option<Cookie> {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        // Safety: unwrap() is safe here because we ensure cookies is Some() in the lines above.
        // This pattern guarantees cookies.is_some() before calling unwrap().
        self.cookies.as_ref().unwrap().get(key).cloned()
    }

    /// Gets a specific cookie by key, returning a default cookie if not found.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// The cookie if found, or a default empty cookie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Existing cookie
    /// let session_cookie = meta.get_cookie_or_default("sessionId");
    /// assert_eq!(session_cookie.get_value(), "abc123");
    ///
    /// // Non-existent cookie returns default
    /// let nonexistent = meta.get_cookie_or_default("nonexistent");
    /// assert_eq!(nonexistent.get_value(), "");
    /// ```
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.get_cookie(key).unwrap_or_else(|| Cookie::new(""))
    }

    /// Parses cookies from either request Cookie header or response Set-Cookie headers,
    /// depending on the type of HTTP message (request or response).
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // For a request with a Cookie header
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// # use hotaru_http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_request("GET / HTTP/1.1"), headers);
    ///
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    ///
    /// // For a response with Set-Cookie headers
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// # use hotaru_http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("set-cookie".to_string(), HeaderValue::new("sessionId=abc123; Path=/; Secure"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_response("HTTP/1.1 200 OK"), headers);
    ///
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    /// assert_eq!(cookies.get("sessionId").unwrap().get_path(), Some("/".to_string()));
    /// assert_eq!(cookies.get("sessionId").unwrap().get_secure(), Some(true));
    /// ```
    pub fn parse_cookies(&self) -> CookieMap {
        // Check if this is a request or response
        if self.start_line.is_request() {
            self.parse_request_cookies()
        } else {
            self.parse_response_cookies()
        }
    }

    /// Parses cookies from the request Cookie header.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    fn parse_request_cookies(&self) -> CookieMap {
        let cookie_header = self.header.get("cookie");

        match cookie_header {
            Some(header_value) => match header_value {
                HeaderValue::Single(cookie_str) => CookieMap::parse(cookie_str),
                HeaderValue::Multiple(cookie_strs) => {
                    // Combine multiple cookie headers into one map
                    let mut cookie_map = CookieMap::new();
                    for cookie_str in cookie_strs {
                        let parsed = CookieMap::parse(cookie_str);
                        for (k, v) in parsed.0.into_iter() {
                            cookie_map.set(k, v);
                        }
                    }
                    cookie_map
                }
            },
            None => CookieMap::default(),
        }
    }

    /// Parses cookies from the response Set-Cookie headers.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies with their attributes.
    fn parse_response_cookies(&self) -> CookieMap {
        let set_cookie_header = self.header.get("set-cookie");

        match set_cookie_header {
            Some(HeaderValue::Single(s)) => CookieMap::parse_set_cookies([s.as_str()]),
            Some(HeaderValue::Multiple(v)) => {
                CookieMap::parse_set_cookies(v.iter().map(|s| s.as_str()))
            }
            None => CookieMap::default(),
        }
    }

    pub fn set_cookies(&mut self, cookies: CookieMap) {
        self.cookies = Some(cookies);
    }

    /// Add a cookie to the HTTP meta data.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key.
    /// * `cookie` - The cookie to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::cookie::Cookie;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.add_cookie("sessionId", Cookie::new("abc123"));
    /// assert_eq!(meta.get_cookie("sessionId").unwrap().get_value(), "abc123");
    ///
    /// meta.add_cookie("sessionCont", Cookie::new("123"));
    /// assert_eq!(meta.get_cookie("sessionId").unwrap().get_value(), "abc123");
    /// ```
    pub fn add_cookie<T: Into<String>>(&mut self, key: T, cookie: Cookie) {
        if self.cookies.is_none() {
            self.cookies = Some(CookieMap::new());
        }
        if let Some(ref mut cookies) = self.cookies {
            cookies.set(key, cookie);
        }
    }

    /// Clears the cached cookies field without modifying the header map.
    ///
    /// This method invalidates the cached cookies value, which will cause
    /// subsequent calls to `get_cookies()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_cookies()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into the cache
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value(), "abc123");
    ///
    /// // Clear the cache only
    /// meta.clear_cookies();
    ///
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_cookies().get("sessionId").unwrap().value(), "abc123");
    /// ```
    pub fn clear_cookies(&mut self) {
        self.cookies = None;
    }

    /// Deletes the Cookie header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the cookie header from the headers map and
    /// clears the cached cookies value. Subsequent calls to `get_cookies()`
    /// will return an empty cookie map unless new cookies are set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both the cache and header
    /// meta.delete_cookies();
    ///
    /// // The header is gone
    /// assert!(meta.get_header("cookie").is_none());
    ///
    /// // And get_cookies will now return an empty map
    /// assert!(meta.get_cookies().is_empty());
    /// ```
    pub fn delete_cookies(&mut self) {
        self.cookies = None;
        self.header.remove("cookie");
        self.header.remove("set-cookie");
    }
}
