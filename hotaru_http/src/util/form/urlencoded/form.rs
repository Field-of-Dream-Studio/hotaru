use core::fmt;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::error::UrlEncodedError;
use super::parse::parse_urlencoded;
use super::serialize::serialize_urlencoded;

/// Represents a parsed `application/x-www-form-urlencoded` form body.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UrlEncodedForm {
    pub data: HashMap<String, String>,
}

impl fmt::Display for UrlEncodedForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&serialize_urlencoded(self))
    }
}

impl UrlEncodedForm {
    /// Creates a new `UrlEncodedForm` with an empty `HashMap`.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Parses raw bytes into a `UrlEncodedForm`.
    ///
    /// # Errors
    ///
    /// Returns a [`UrlEncodedError`] if the body contains invalid UTF-8 or malformed key-value pairs.
    pub fn parse(body: impl AsRef<[u8]>) -> Result<Self, UrlEncodedError> {
        parse_urlencoded(body)
    }

    /// Serializes the form back into a URL-encoded string.
    pub fn to_string(&self) -> String {
        serialize_urlencoded(self)
    }

    /// Inserts a key-value pair into the form.
    pub fn insert(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    /// Gets the value associated with a key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Gets the value associated with a key, or an empty string reference if missing.
    pub fn get_or_default(&self, key: &str) -> &String {
        if let Some(value) = self.data.get(key) {
            return value;
        }
        static EMPTY: Lazy<String> = Lazy::new(|| "".to_string());
        &EMPTY
    }

    /// Gets all key-value pairs in the form.
    pub fn get_all(&self) -> &HashMap<String, String> {
        &self.data
    }
}

impl From<HashMap<String, String>> for UrlEncodedForm {
    fn from(data: HashMap<String, String>) -> Self {
        Self { data }
    }
}
