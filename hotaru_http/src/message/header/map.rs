//! Typed collection of HTTP header name/value pairs.

use std::collections::HashMap;
use std::collections::hash_map;

use super::error::HeaderError;
use super::value::HeaderValue;

/// Collection of HTTP header name/value pairs.
///
/// Wraps `HashMap<String, HeaderValue>` with typed accessors that distinguish
/// present-but-multiple from absent (`get_only`) and give safe iteration over
/// every value for a name (`get_all`).
#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    inner: HashMap<String, HeaderValue>,
}

impl HeaderMap {
    /// Creates an empty `HeaderMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of distinct header names stored.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if no header names are stored.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns a reference to the value(s) stored under `name`, if any.
    pub fn get<Q: AsRef<str>>(&self, name: Q) -> Option<&HeaderValue> {
        self.inner.get(name.as_ref())
    }

    /// Returns the value stored under `name` if it is uniquely present.
    ///
    /// - Absent → `Ok(None)`.
    /// - Present exactly once → `Ok(Some(&str))`.
    /// - Present more than once → `Err(HeaderError::MultipleValues(name))`.
    ///
    /// `name` must be a `&'static str` so the error payload never carries
    /// untrusted bytes from the wire.
    pub fn get_only(&self, name: &'static str) -> Result<Option<&str>, HeaderError> {
        match self.inner.get(name) {
            None => Ok(None),
            Some(HeaderValue::Single(value)) => Ok(Some(value.as_str())),
            Some(HeaderValue::Multiple(values)) if values.len() == 1 => {
                Ok(Some(values[0].as_str()))
            }
            Some(HeaderValue::Multiple(_)) => Err(HeaderError::MultipleValues(name)),
        }
    }

    /// Returns every string value stored under `name`, in insertion order.
    pub fn get_all<Q: AsRef<str>>(&self, name: Q) -> Vec<&str> {
        match self.inner.get(name.as_ref()) {
            None => Vec::new(),
            Some(HeaderValue::Single(value)) => vec![value.as_str()],
            Some(HeaderValue::Multiple(values)) => {
                values.iter().map(String::as_str).collect()
            }
        }
    }

    /// Returns true if any value is stored under `name`.
    pub fn contains_key<Q: AsRef<str>>(&self, name: Q) -> bool {
        self.inner.contains_key(name.as_ref())
    }

    /// Inserts a value, replacing any existing value(s) for the same name.
    /// Returns the previous value, if any.
    pub fn insert(&mut self, name: String, value: HeaderValue) -> Option<HeaderValue> {
        self.inner.insert(name, value)
    }

    /// Removes and returns the value(s) for `name`, if any.
    pub fn remove<Q: AsRef<str>>(&mut self, name: Q) -> Option<HeaderValue> {
        self.inner.remove(name.as_ref())
    }

    /// Extends this map with the entries of `other`, overwriting duplicates.
    pub fn extend(&mut self, other: HeaderMap) {
        self.inner.extend(other.inner);
    }

    /// Borrowing iterator over all name/value pairs.
    pub fn iter(&self) -> hash_map::Iter<'_, String, HeaderValue> {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = (&'a String, &'a HeaderValue);
    type IntoIter = hash_map::Iter<'a, String, HeaderValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl From<HashMap<String, HeaderValue>> for HeaderMap {
    fn from(inner: HashMap<String, HeaderValue>) -> Self {
        Self { inner }
    }
}

impl From<HeaderMap> for HashMap<String, HeaderValue> {
    fn from(map: HeaderMap) -> Self {
        map.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_only_returns_none_when_absent() {
        let map = HeaderMap::new();
        assert!(matches!(map.get_only("content-length"), Ok(None)));
    }

    #[test]
    fn get_only_returns_value_when_singular() {
        let mut map = HeaderMap::new();
        map.insert(
            "content-length".to_string(),
            HeaderValue::Single("42".to_string()),
        );
        assert!(matches!(map.get_only("content-length"), Ok(Some("42"))));
    }

    #[test]
    fn get_only_errors_when_multiple() {
        let mut map = HeaderMap::new();
        map.insert(
            "content-length".to_string(),
            HeaderValue::Multiple(vec!["10".to_string(), "20".to_string()]),
        );
        assert!(matches!(
            map.get_only("content-length"),
            Err(HeaderError::MultipleValues("content-length"))
        ));
    }
}
