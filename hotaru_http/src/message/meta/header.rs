/// Represents a value for an HTTP header, which can be either a single string or multiple values.
///
/// HTTP headers can sometimes have multiple values, which are typically combined with commas,
/// but some special headers like Set-Cookie maintain separate values.
#[derive(Debug, Clone)]
pub enum HeaderValue {
    /// A single header value
    Single(String),
    /// Multiple header values
    Multiple(Vec<String>),
}

impl HeaderValue {
    /// Create a new HeaderValue from a single string.
    ///
    /// # Arguments
    ///
    /// * `value` - A string that represents the header value.
    ///
    /// # Returns
    ///
    /// A new HeaderValue containing a single value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header = HeaderValue::new("application/json");
    /// ```
    pub fn new<T: Into<String>>(value: T) -> Self {
        HeaderValue::Single(value.into())
    }

    /// Append a new value to the HeaderValue.
    ///
    /// If the HeaderValue is a single value, it will convert it to a multiple value.
    /// Values are typically combined with comma separators for standard HTTP headers.
    ///
    /// # Arguments
    ///
    /// * `value` - A string that represents the header value to append.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("charset=UTF-8");
    /// assert_eq!(header_value.as_str(), "text/html, charset=UTF-8");
    /// ```
    pub fn append<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(s) => {
                let mut values = vec![s.clone()];
                values.push(value.into());
                *self = HeaderValue::Multiple(values);
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Convert the HeaderValue to a string representation.
    ///
    /// Multiple values are joined with a comma and space, following HTTP header conventions.
    ///
    /// # Returns
    ///
    /// A string representation of the header value(s).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("application/xhtml+xml");
    /// assert_eq!(header_value.as_str(), "text/html, application/xhtml+xml");
    /// ```
    pub fn as_str(&self) -> String {
        match self {
            HeaderValue::Single(s) => s.clone(),
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }

    /// Returns the number of values in this HeaderValue.
    ///
    /// # Returns
    ///
    /// * `usize` - 1 for a single value, or the count of values for multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.len(), 1);
    ///
    /// header.append("application/json");
    /// assert_eq!(header.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            HeaderValue::Single(_) => 1,
            HeaderValue::Multiple(v) => v.len(),
        }
    }

    /// Checks if the HeaderValue is empty.
    ///
    /// A HeaderValue is considered empty if it contains no values or only empty strings.
    ///
    /// # Returns
    ///
    /// `true` if the header value is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let empty_header = HeaderValue::new("");
    /// assert!(empty_header.is_empty());
    ///
    /// let header = HeaderValue::new("application/json");
    /// assert!(!header.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            HeaderValue::Single(s) => s.is_empty(),
            HeaderValue::Multiple(v) => v.is_empty() || v.iter().all(|s| s.is_empty()),
        }
    }

    /// Attempts to get a value at the specified index.
    ///
    /// For a single value, only index 0 is valid.
    /// For multiple values, any valid index within the range of values is accepted.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<&String>` - The value at the specified index, or None if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.try_get(0), Some(&"text/html".to_string()));
    /// assert_eq!(header.try_get(1), None);
    ///
    /// header.append("application/json");
    /// assert_eq!(header.try_get(1), Some(&"application/json".to_string()));
    /// ```
    pub fn try_get(&self, index: usize) -> Option<&String> {
        match self {
            HeaderValue::Single(s) if index == 0 => Some(s),
            HeaderValue::Single(_) => None,
            HeaderValue::Multiple(v) => v.get(index),
        }
    }

    /// Gets a value at the specified index, or returns an empty string if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to retrieve.
    ///
    /// # Returns
    ///
    /// The string at the specified index, or an empty string if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get(0), "text/html");
    /// assert_eq!(header.get(1), ""); // Out of bounds returns empty string
    /// ```
    pub fn get(&self, index: usize) -> String {
        self.try_get(index).cloned().unwrap_or_default()
    }

    /// Gets a value at the specified index, or returns the provided default if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the value to retrieve.
    /// * `default` - The default value to return if the index is out of bounds.
    ///
    /// # Returns
    ///
    /// The string at the specified index, or the default if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get_or(0, "default"), "text/html");
    /// assert_eq!(header.get_or(1, "default"), "default"); // Out of bounds returns default
    /// ```
    pub fn get_or<S: Into<String>>(&self, index: usize, default: S) -> String {
        self.try_get(index)
            .cloned()
            .unwrap_or_else(|| default.into())
    }

    /// Add a value to the header without combining it with existing values.
    ///
    /// This is useful for headers like Set-Cookie where each value should be treated
    /// as a separate header instance rather than being combined with commas.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut cookies = HeaderValue::new("sessionId=abc123; Path=/");
    /// cookies.add_without_combining("theme=dark; Path=/; Max-Age=3600");
    ///
    /// // Each cookie is kept as a separate value
    /// assert_eq!(cookies.try_get(0), Some(&"sessionId=abc123; Path=/".to_string()));
    /// assert_eq!(cookies.try_get(1), Some(&"theme=dark; Path=/; Max-Age=3600".to_string()));
    ///
    /// // When we use as_str() they'll still be combined with commas for API consistency
    /// // but should be treated separately when used with headers like Set-Cookie
    /// ```
    pub fn add_without_combining<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(_) => {
                let original = std::mem::replace(self, HeaderValue::Multiple(Vec::new()));
                if let HeaderValue::Single(s) = original {
                    *self = HeaderValue::Multiple(vec![s, value.into()]);
                }
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Attempts to get the first value in this HeaderValue.
    ///
    /// # Returns
    ///
    /// * `Option<&String>` - The first value, or None if there are no values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    /// assert_eq!(header.try_first(), Some(&"text/html".to_string()));
    /// ```
    pub fn try_first(&self) -> Option<&String> {
        match self {
            HeaderValue::Single(value) => Some(value),
            HeaderValue::Multiple(values) if !values.is_empty() => Some(&values[0]),
            _ => None,
        }
    }

    /// Gets the first value in this HeaderValue, or an empty string if there are no values.
    ///
    /// # Returns
    ///
    /// The first value, or an empty string if there are no values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first(), "text/html");
    ///
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first(), "");
    /// ```
    pub fn first(&self) -> String {
        self.try_first().cloned().unwrap_or_default()
    }

    /// Gets the first value in this HeaderValue, or the provided default if there are no values.
    ///
    /// # Arguments
    ///
    /// * `default` - The default value to return if there are no values.
    ///
    /// # Returns
    ///
    /// The first value, or the default if there are no values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first_or("default"), "text/html");
    ///
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first_or("default"), "default");
    /// ```
    pub fn first_or<S: Into<String>>(&self, default: S) -> String {
        self.try_first().cloned().unwrap_or_else(|| default.into())
    }

    /// Gets all values as a vector of string references.
    ///
    /// # Returns
    ///
    /// A vector containing references to all values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    ///
    /// let values = header.values();
    /// assert_eq!(values.len(), 2);
    /// assert_eq!(values[0], &"text/html".to_string());
    /// assert_eq!(values[1], &"application/json".to_string());
    /// ```
    pub fn values(&self) -> Vec<&String> {
        match self {
            HeaderValue::Single(value) => vec![value],
            HeaderValue::Multiple(values) => values.iter().collect(),
        }
    }

    /// Converts the HeaderValue into a string suitable f or use in HTTP headers.
    /// This method formats the header value according to HTTP standards, ensuring
    /// that single values are represented as a single line and multiple values are
    /// each represented on their own line.
    ///
    /// # Arguments
    /// * `header_name` - The name of the header to use in the formatted string.
    ///
    /// # Returns
    /// A string formatted as an HTTP header line or lines, ready to be sent in a request or response.
    ///
    /// # Examples
    /// ```rust
    /// # use hotaru_http::meta::HeaderValue;
    /// let header_value = HeaderValue::new("text/html");
    /// let header_string = header_value.into_header_string("Content-Type");
    /// assert_eq!(header_string, "Content-Type: text/html\r\n");
    /// let mut multi_header = HeaderValue::new("text/html");
    /// multi_header.append("application/json");
    /// let multi_header_string = multi_header.into_header_string("Accept");
    /// assert_eq!(multi_header_string, "Accept: text/html\r\nAccept: application/json\r\n");
    /// ```
    pub fn into_header_string(&self, header_name: &str) -> String {
        match self {
            HeaderValue::Single(v) => {
                // Single values get a single header line
                format!("{}: {}\r\n", header_name, v)
            }
            HeaderValue::Multiple(values) => {
                // Multiple values each get their own header line
                let mut result = String::new();
                for v in values {
                    result.push_str(&format!("{}: {}\r\n", header_name, v));
                }
                result
            }
        }
    }
}

/// Implements conversion from a string to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
///
/// ```rust
/// # use hotaru_http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".to_string().into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<String> for HeaderValue {
    fn from(value: String) -> Self {
        HeaderValue::new(value)
    }
}

/// Implements conversion from a string slice to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
///
/// ```rust
/// # use hotaru_http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<&str> for HeaderValue {
    fn from(value: &str) -> Self {
        HeaderValue::new(value.to_string())
    }
}

/// Implements iterator for HeaderValue to easily iterate over all values.
///
/// # Examples
///
/// ```rust
/// # use hotaru_http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
///
/// let mut values = Vec::new();
/// for value in header {
///     values.push(value);
/// }
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl IntoIterator for HeaderValue {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HeaderValue::Single(s) => vec![s].into_iter(),
            HeaderValue::Multiple(v) => v.into_iter(),
        }
    }
}

/// Implements conversion from HeaderValue to a vector of strings.
///
/// # Examples
///
/// ```rust
/// # use hotaru_http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
///
/// let values: Vec<String> = header.into();
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl From<HeaderValue> for Vec<String> {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => vec![s],
            HeaderValue::Multiple(v) => v,
        }
    }
}

/// Implements conversion from HeaderValue to a string.
///
/// Multiple values are joined with commas and spaces.
///
/// # Examples
///
/// ```rust
/// # use hotaru_http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
///
/// let value: String = header.into();
/// assert_eq!(value, "text/html, application/json");
/// ```
impl From<HeaderValue> for String {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => s,
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }
}
