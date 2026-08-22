/// A header value: one string, or several kept separately.
///
/// Most HTTP headers combine multiple values with commas, but some
/// (e.g. `Set-Cookie`) must stay as separate values.
#[derive(Debug, Clone)]
pub enum HeaderValue {
    /// A single header value
    Single(String),
    /// Multiple header values
    Multiple(Vec<String>),
}

impl HeaderValue {
    /// Creates a `HeaderValue` holding a single string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let header = HeaderValue::new("application/json");
    /// ```
    pub fn new<T: Into<String>>(value: T) -> Self {
        HeaderValue::Single(value.into())
    }

    /// Appends a value; promotes `Single` to `Multiple` on first append.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Renders the value(s) as a single string; multiples are joined with `, `.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Number of stored values (1 for `Single`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// True if there are no values, or every stored value is the empty string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Returns the value at `index`, or `None` if out of range.
    /// A `Single` accepts only index 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Returns the value at `index`, or an empty string if out of range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get(0), "text/html");
    /// assert_eq!(header.get(1), ""); // Out of bounds returns empty string
    /// ```
    pub fn get(&self, index: usize) -> String {
        self.try_get(index).cloned().unwrap_or_default()
    }

    /// Returns the value at `index`, or the caller-supplied default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get_or(0, "default"), "text/html");
    /// assert_eq!(header.get_or(1, "default"), "default"); // Out of bounds returns default
    /// ```
    pub fn get_or<S: Into<String>>(&self, index: usize, default: S) -> String {
        self.try_get(index)
            .cloned()
            .unwrap_or_else(|| default.into())
    }

    /// Adds a value without combining it into an existing one.
    /// Use this for headers like `Set-Cookie` where every value stays separate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let mut cookies = HeaderValue::new("sessionId=abc123; Path=/");
    /// cookies.add_without_combining("theme=dark; Path=/; Max-Age=3600");
    ///
    /// // Each cookie is kept as a separate value
    /// assert_eq!(cookies.try_get(0), Some(&"sessionId=abc123; Path=/".to_string()));
    /// assert_eq!(cookies.try_get(1), Some(&"theme=dark; Path=/; Max-Age=3600".to_string()));
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

    /// Returns the first value, or `None` if there are none.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Returns the first value, or an empty string if there are none.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first(), "text/html");
    ///
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first(), "");
    /// ```
    pub fn first(&self) -> String {
        self.try_first().cloned().unwrap_or_default()
    }

    /// Returns the first value, or the caller-supplied default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first_or("default"), "text/html");
    ///
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first_or("default"), "default");
    /// ```
    pub fn first_or<S: Into<String>>(&self, default: S) -> String {
        self.try_first().cloned().unwrap_or_else(|| default.into())
    }

    /// Returns references to every value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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

    /// Renders as one or more `Name: value\r\n` header lines.
    /// `Multiple` produces one line per value (e.g. `Set-Cookie`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hotaru_http::header::HeaderValue;
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
                format!("{}: {}\r\n", header_name, v)
            }
            HeaderValue::Multiple(values) => {
                let mut result = String::new();
                for v in values {
                    result.push_str(&format!("{}: {}\r\n", header_name, v));
                }
                result
            }
        }
    }
}

impl From<String> for HeaderValue {
    fn from(value: String) -> Self {
        HeaderValue::new(value)
    }
}

impl From<&str> for HeaderValue {
    fn from(value: &str) -> Self {
        HeaderValue::new(value.to_string())
    }
}

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

impl From<HeaderValue> for Vec<String> {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => vec![s],
            HeaderValue::Multiple(v) => v,
        }
    }
}

impl From<HeaderValue> for String {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => s,
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }
}
