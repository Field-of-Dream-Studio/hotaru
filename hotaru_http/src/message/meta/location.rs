use super::HttpMeta;

impl HttpMeta {
    /// Gets the location header from the HTTP meta data.
    ///
    /// Returns the cached location if available, otherwise parses
    /// the location header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn get_location(&mut self) -> Option<String> {
        if let Some(ref loc) = self.location {
            return Some(loc.clone());
        }
        self.parse_location()
    }

    /// Parses the Location header from the headers map and stores it in the location field.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let location = meta.parse_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    /// ```
    pub fn parse_location(&mut self) -> Option<String> {
        // Try both lowercase and uppercase for backward compatibility
        let location = self.header.get("location").map(|value| value.first());

        self.set_location(location.clone());
        location
    }

    /// Sets the location field.
    ///
    /// # Arguments
    ///
    /// * `location` - The location to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    ///
    /// let mut meta = HttpMeta::default();
    /// meta.set_location(Some("/redirect".to_string()));
    ///
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn set_location(&mut self, location: Option<String>) {
        self.location = location;
    }

    /// Clears the cached location field without modifying the header map.
    ///
    /// This method invalidates the cached location value, which will cause
    /// subsequent calls to `get_location()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_location()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Parse the value into the cache
    /// let location = meta.get_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    ///
    /// // Clear the cache only
    /// meta.clear_location();
    ///
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn clear_location(&mut self) {
        self.location = None;
    }

    /// Deletes the Location header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the location header from the headers map and
    /// clears the cached location value. Subsequent calls to `get_location()`
    /// will return None unless a new location is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HttpMeta;
    /// # use hotaru_http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// // Delete both the cache and header
    /// meta.delete_location();
    ///
    /// // The header is gone
    /// assert!(meta.get_header("location").is_none());
    ///
    /// // And get_location will now return None
    /// assert_eq!(meta.get_location(), None);
    /// ```
    pub fn delete_location(&mut self) {
        self.location = None;
        self.header.remove("location");
    }
}
