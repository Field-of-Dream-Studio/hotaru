use super::HttpMeta;
use super::error::MetaError;
use crate::message::header::HeaderError;

impl HttpMeta {
    /// Returns the cached Content-Length if set, otherwise parses it from the
    /// header block.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_content_length().unwrap(), Some(123));
    /// ```
    pub fn get_content_length(&mut self) -> Result<Option<u64>, MetaError> {
        if let Some(length) = self.content_length {
            return Ok(Some(length));
        }
        self.parse_content_length()
    }

    /// Parses the `Content-Length` header and caches the result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::header::HeaderValue;
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
    pub fn parse_content_length(&mut self) -> Result<Option<u64>, MetaError> {
        let length = self
            .header
            .get_only_parsed("content-length", |value| {
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(HeaderError::InvalidHeaderValue(
                        "content-length".to_string(),
                    ));
                }
                value
                    .parse::<u64>()
                    .map_err(|_| HeaderError::HeaderValueOverflow("content-length".to_string()))
            })
            .map_err(MetaError::from)?;

        self.content_length = length;
        Ok(length)
    }

    /// Returns the request's declared Content-Length in bytes.
    ///
    /// Absent Content-Length is treated as 0 per RFC 9112 §6.3 (request
    /// without framing headers has zero-length body). Use this on the request
    /// path; response bodies without Content-Length can be close-delimited
    /// and need different handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// assert_eq!(meta.get_content_length_request().unwrap(), 0);
    ///
    /// meta.set_content_length(42);
    /// assert_eq!(meta.get_content_length_request().unwrap(), 42);
    /// ```
    pub fn get_content_length_request(&mut self) -> Result<u64, MetaError> {
        Ok(self.get_content_length()?.unwrap_or(0))
    }

    /// Sets the cached Content-Length.
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

    /// Clears the cached Content-Length without removing the header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(123);
    /// meta.clear_content_length();
    /// ```
    pub fn clear_content_length(&mut self) {
        self.content_length = None;
    }

    /// Deletes both the cached Content-Length and the header entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_attribute("content-length", "123");
    /// meta.delete_content_length();
    ///
    /// assert!(meta.get_header("content-length").is_none());
    /// ```
    pub fn delete_content_length(&mut self) {
        self.content_length = None;
        self.header.remove("content-length");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::header::HeaderValue;

    fn meta_with(name: &str, value: HeaderValue) -> HttpMeta {
        let mut meta = HttpMeta::default();
        meta.header.insert(name.to_string(), value);
        meta
    }

    #[test]
    fn parses_valid_content_length() {
        let mut meta = meta_with("content-length", HeaderValue::Single("42".to_string()));
        assert_eq!(meta.parse_content_length().unwrap(), Some(42));
    }

    #[test]
    fn absent_content_length_is_ok_none() {
        let mut meta = HttpMeta::default();
        assert_eq!(meta.parse_content_length().unwrap(), None);
    }

    #[test]
    fn non_numeric_is_invalid_header_value() {
        let mut meta = meta_with("content-length", HeaderValue::Single("abc".to_string()));
        let err = meta.parse_content_length().unwrap_err();
        assert!(matches!(
            err,
            MetaError::Header(HeaderError::InvalidHeaderValue(ref name)) if name == "content-length"
        ));
    }

    #[test]
    fn empty_value_is_invalid_header_value() {
        let mut meta = meta_with("content-length", HeaderValue::Single(String::new()));
        let err = meta.parse_content_length().unwrap_err();
        assert!(matches!(
            err,
            MetaError::Header(HeaderError::InvalidHeaderValue(ref name)) if name == "content-length"
        ));
    }

    #[test]
    fn overflowing_value_is_header_value_overflow() {
        let mut meta = meta_with(
            "content-length",
            HeaderValue::Single("18446744073709551616".to_string()),
        );
        let err = meta.parse_content_length().unwrap_err();
        assert!(matches!(
            err,
            MetaError::Header(HeaderError::HeaderValueOverflow(ref name)) if name == "content-length"
        ));
    }

    #[test]
    fn get_content_length_request_treats_absent_as_zero() {
        let mut meta = HttpMeta::default();
        assert_eq!(meta.get_content_length_request().unwrap(), 0);

        let mut meta_with_value =
            meta_with("content-length", HeaderValue::Single("42".to_string()));
        assert_eq!(meta_with_value.get_content_length_request().unwrap(), 42);
    }

    #[test]
    fn get_content_length_request_propagates_parse_errors() {
        let mut meta = meta_with("content-length", HeaderValue::Single("abc".to_string()));
        let err = meta.get_content_length_request().unwrap_err();
        assert!(matches!(
            err,
            MetaError::Header(HeaderError::InvalidHeaderValue(ref name)) if name == "content-length"
        ));
    }

    #[test]
    fn multiple_content_length_headers_yields_multiple_values() {
        let mut meta = meta_with(
            "content-length",
            HeaderValue::Multiple(vec!["10".to_string(), "20".to_string()]),
        );
        let err = meta.parse_content_length().unwrap_err();
        assert!(matches!(
            err,
            MetaError::Header(HeaderError::MultipleValues(ref name)) if name == "content-length"
        ));
    }
}
