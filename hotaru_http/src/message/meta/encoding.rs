use super::HttpMeta;
use crate::util::encoding::HttpEncoding;

impl HttpMeta {
    /// Gets the HTTP encoding (both transfer and content encoding) from the HTTP meta data.
    ///
    /// Returns the cached encoding if available, otherwise parses
    /// the transfer-encoding and content-encoding headers from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<HttpEncoding>` - The HTTP encodings, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("gzip")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let encoding = meta.get_encoding();
    /// assert!(encoding.is_some());
    /// let encoding = encoding.unwrap();
    /// assert!(encoding.transfer().is_chunked());
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn get_encoding(&mut self) -> Option<HttpEncoding> {
        if let Some(ref enc) = self.encoding {
            return Some(enc.clone());
        }
        self.parse_encoding()
    }

    /// Parses the Transfer-Encoding and Content-Encoding headers from the headers map
    /// and stores them in the encoding field.
    ///
    /// # Returns
    ///
    /// * `Option<HttpEncoding>` - The parsed HTTP encodings
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("br")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let encoding = meta.parse_encoding();
    /// assert!(encoding.is_some());
    /// let encoding = encoding.unwrap();
    /// assert!(encoding.transfer().is_chunked());
    /// assert_eq!(encoding.content().to_header(), "br");
    /// ```
    pub fn parse_encoding(&mut self) -> Option<HttpEncoding> {
        // Get header values as comma-separated strings
        let transfer_header = self
            .header
            .get("transfer-encoding")
            .map(|values| values.first());

        let content_header = self
            .header
            .get("content-encoding")
            .map(|values| values.first());

        let encoding = HttpEncoding::from_headers(transfer_header, content_header);
        self.encoding = Some(encoding.clone());
        Some(encoding)
    }

    /// Sets the encoding field with both transfer and content encodings
    ///
    /// # Arguments
    ///
    /// * `encoding` - The HTTP encodings to cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::encoding::HttpEncoding;
    ///
    /// let mut meta = HttpMeta::default();
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     Some("gzip".to_string())
    /// );
    ///
    /// meta.set_encoding(Some(encoding.clone()));
    ///
    /// assert!(meta.get_encoding().unwrap().transfer().is_chunked());
    /// assert!(!meta.get_encoding().unwrap().content().is_identity());
    /// ```
    pub fn set_encoding(&mut self, encoding: Option<HttpEncoding>) {
        self.encoding = encoding;
    }

    /// Clears the cached encoding field without modifying the header map
    ///
    /// Subsequent calls to `get_encoding()` will re-parse the value from headers
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into cache
    /// let _ = meta.get_encoding();
    ///
    /// // Clear the cache only
    /// meta.clear_encoding();
    ///
    /// // Header is still intact and will be re-parsed
    /// assert!(meta.get_encoding().is_some());
    /// ```
    pub fn clear_encoding(&mut self) {
        self.encoding = None;
    }

    /// Deletes both Transfer-Encoding and Content-Encoding headers
    ///
    /// Clears both the cached field and removes headers from the map
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("gzip")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("br")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both cache and headers
    /// meta.delete_encoding();
    ///
    /// // Headers are gone
    /// assert!(meta.get_header("transfer-encoding").is_none());
    /// assert!(meta.get_header("content-encoding").is_none());
    ///
    /// // Encoding is now identity
    /// let encoding = meta.get_encoding().unwrap();
    /// assert!(encoding.transfer().is_identity());
    /// assert!(encoding.content().is_identity());
    /// ```
    pub fn delete_encoding(&mut self) {
        self.encoding = None;
        self.header.remove("transfer-encoding");
        self.header.remove("content-encoding");
    }
}
