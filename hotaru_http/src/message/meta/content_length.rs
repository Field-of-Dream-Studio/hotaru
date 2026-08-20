use super::{HeaderValue, HttpMeta};
use crate::connection::error::ConnectionError;

impl HttpMeta {
    /// Gets the content length from the HTTP meta data.
    ///
    /// Returns the cached content length if available, otherwise parses
    /// the content-length header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Result<Option<u64>, ConnectionError>` - The content length, absence, or a parsing error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_content_length().unwrap(), Some(123));
    /// ```
    pub fn get_content_length(&mut self) -> Result<Option<u64>, ConnectionError> {
        if let Some(length) = self.content_length {
            return Ok(Some(length));
        }
        self.parse_content_length()
    }

    /// Parses the Content-Length header from the headers map and stores it in the content_length field.
    ///
    /// # Returns
    ///
    /// * `Result<Option<u64>, ConnectionError>` - The parsed value, absence, or a parsing error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let length = meta.parse_content_length();
    /// assert_eq!(length.unwrap(), Some(123));
    /// assert_eq!(meta.get_content_length().unwrap(), Some(123));
    /// ```
    pub fn parse_content_length(&mut self) -> Result<Option<u64>, ConnectionError> {
        let length = match self.header.get("content-length") {
            None => None,
            Some(HeaderValue::Single(value)) => {
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(ConnectionError::BadRequest(
                        "invalid Content-Length".to_string(),
                    ));
                }

                Some(value.parse::<u64>().map_err(|_| {
                    ConnectionError::BadRequest("Content-Length is too large".to_string())
                })?)
            }
            Some(HeaderValue::Multiple(_)) => {
                return Err(ConnectionError::BadRequest(
                    "multiple Content-Length values".to_string(),
                ));
            }
        };

        self.content_length = length;

        Ok(length)
    }

    /// Sets the content_length field.
    ///
    /// # Arguments
    ///
    /// * `length` - The content length to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(456);
    ///
    /// assert_eq!(meta.get_content_length().unwrap(), Some(456));
    /// ```
    pub fn set_content_length(&mut self, length: usize) {
        self.content_length = Some(length as u64);
    }

    /// Clears the cached content_length field without modifying the header map.
    ///
    /// Note that it will **NOT** clear the value in the HashMap.
    /// To remove both the cached field and the header, use `delete_content_length()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(123);
    /// meta.clear_content_length();
    ///
    /// // The content-length header in the HashMap is still intact
    /// // but the cached value is cleared
    /// ```
    pub fn clear_content_length(&mut self) {
        self.content_length = None;
    }

    /// Deletes the Content-Length header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_header("content-length", "123");
    /// meta.delete_content_length();
    ///
    /// // Both the cached field and the header are now removed
    /// assert!(meta.get_header("content-length").is_none());
    /// ```
    pub fn delete_content_length(&mut self) {
        self.content_length = None;
        self.header.remove("content-length");
    }
}
