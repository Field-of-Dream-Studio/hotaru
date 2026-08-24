use super::HttpMeta;
use crate::message::http_value::HttpContentType;

impl HttpMeta {
    /// Gets the content type from the HTTP meta data.
    ///
    /// Returns the cached content type if available, otherwise parses
    /// the content-type header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The content type, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html; charset=UTF-8"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml()));
    /// ```
    pub fn get_content_type(&mut self) -> Option<HttpContentType> {
        if let Some(ref content_type) = self.content_type {
            return Some(content_type.clone());
        }
        self.parse_content_type()
    }

    /// Parses the Content-Type header from the headers map and stores it in the content_type field.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The parsed Content-Type value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html; charset=UTF-8"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let content_type = meta.parse_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml()));
    /// ```
    pub fn parse_content_type(&mut self) -> Option<HttpContentType> {
        // Try lowercase first, then uppercase for backward compatibility
        let content_type_str = self.header.get("content-type").map(|value| value.first())?;

        let content_type = HttpContentType::from_str(&content_type_str);
        self.set_content_type(content_type.clone());
        Some(content_type)
    }

    /// Sets the content_type field.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The content type to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::http_value::HttpContentType;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_type(HttpContentType::ApplicationJson());
    ///
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::ApplicationJson()));
    /// ```
    pub fn set_content_type(&mut self, content_type: HttpContentType) {
        self.content_type = Some(content_type);
    }

    /// Clears the cached content_type field without modifying the header map.
    ///
    /// This method invalidates the cached content_type value, which will cause
    /// subsequent calls to `get_content_type()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_content_type()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html; charset=UTF-8"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into the cache
    /// let content_type = meta.get_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml()));
    ///
    /// // Clear the cache only
    /// meta.clear_content_type();
    ///
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml()));
    /// ```
    pub fn clear_content_type(&mut self) {
        self.content_type = None;
    }

    /// Deletes the Content-Type header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the content-type header from the headers map and
    /// clears the cached content_type value. Subsequent calls to `get_content_type()`
    /// will return a default value unless a new content-type is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// # use hotaru_http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html; charset=UTF-8"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both the cache and header
    /// meta.delete_content_type();
    ///
    /// // The header is gone
    /// assert!(meta.get_header("content-type").is_none());
    ///
    /// // And get_content_type will now return None until a new one is set
    /// assert_eq!(meta.get_content_type(), None);
    /// ```
    pub fn delete_content_type(&mut self) {
        self.content_type = None;
        self.header.remove("content-type");
    }
}
