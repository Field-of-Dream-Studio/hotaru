//! Typed collection of HTTP header name/value pairs.

use std::collections::HashMap;
use std::collections::hash_map;

use super::error::HeaderError;
use super::value::HeaderValue;

/// Collection of HTTP header name/value pairs.
///
/// Accessors come in `get_*` / `require_*` pairs: `get_*` treats absence as
/// non-error (`Option`/`Vec`); `require_*` returns `Err(HeaderError::Missing(name))`
/// on absence. All accessors take `impl AsRef<str>`.
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

    /// Like [`get`] but treats absence as [`HeaderError::Missing`].
    ///
    /// [`get`]: HeaderMap::get
    pub fn require<Q: AsRef<str>>(&self, name: Q) -> Result<&HeaderValue, HeaderError> {
        let name = name.as_ref();
        self.inner
            .get(name)
            .ok_or_else(|| HeaderError::Missing(name.to_string()))
    }

    /// Runs `parse` on the raw [`HeaderValue`] under `name`. Absence is
    /// `Ok(None)`; the closure sees `Single` or `Multiple` unchanged.
    pub fn get_parsed<T, Q, F>(&self, name: Q, parse: F) -> Result<Option<T>, HeaderError>
    where
        Q: AsRef<str>,
        F: FnOnce(&HeaderValue) -> Result<T, HeaderError>,
    {
        match self.inner.get(name.as_ref()) {
            None => Ok(None),
            Some(value) => parse(value).map(Some),
        }
    }

    /// Like [`get_parsed`] but returns `Err(Missing)` on absence.
    ///
    /// [`get_parsed`]: HeaderMap::get_parsed
    pub fn require_parsed<T, Q, F>(&self, name: Q, parse: F) -> Result<T, HeaderError>
    where
        Q: AsRef<str>,
        F: FnOnce(&HeaderValue) -> Result<T, HeaderError>,
    {
        let name = name.as_ref();
        self.get_parsed(name, parse)?
            .ok_or_else(|| HeaderError::Missing(name.to_string()))
    }

    /// Returns the unique value under `name`: `Ok(None)` absent,
    /// `Ok(Some(&str))` one value, `Err(MultipleValues)` more than one.
    pub fn get_only<Q: AsRef<str>>(&self, name: Q) -> Result<Option<&str>, HeaderError> {
        let name = name.as_ref();
        match self.inner.get(name) {
            None => Ok(None),
            Some(HeaderValue::Single(value)) => Ok(Some(value.as_str())),
            Some(HeaderValue::Multiple(values)) if values.len() == 1 => {
                Ok(Some(values[0].as_str()))
            }
            Some(HeaderValue::Multiple(_)) => {
                Err(HeaderError::MultipleValues(name.to_string()))
            }
        }
    }

    /// Like [`get_only`] but treats absence as [`HeaderError::Missing`].
    ///
    /// [`get_only`]: HeaderMap::get_only
    pub fn require_only<Q: AsRef<str>>(&self, name: Q) -> Result<&str, HeaderError> {
        let name = name.as_ref();
        self.get_only(name)?
            .ok_or_else(|| HeaderError::Missing(name.to_string()))
    }

    /// Like [`get_only`] but runs `parse` on the value. Closure errors and
    /// `MultipleValues` propagate.
    ///
    /// [`get_only`]: HeaderMap::get_only
    pub fn get_only_parsed<T, Q, F>(
        &self,
        name: Q,
        parse: F,
    ) -> Result<Option<T>, HeaderError>
    where
        Q: AsRef<str>,
        F: FnOnce(&str) -> Result<T, HeaderError>,
    {
        match self.get_only(name)? {
            None => Ok(None),
            Some(value) => parse(value).map(Some),
        }
    }

    /// Like [`get_only_parsed`] but returns `Err(Missing)` on absence.
    ///
    /// [`get_only_parsed`]: HeaderMap::get_only_parsed
    pub fn require_only_parsed<T, Q, F>(
        &self,
        name: Q,
        parse: F,
    ) -> Result<T, HeaderError>
    where
        Q: AsRef<str>,
        F: FnOnce(&str) -> Result<T, HeaderError>,
    {
        let name = name.as_ref();
        self.get_only_parsed(name, parse)?
            .ok_or_else(|| HeaderError::Missing(name.to_string()))
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

    /// Like [`get_all`] but returns `Err(Missing)` on absence or zero values.
    ///
    /// [`get_all`]: HeaderMap::get_all
    pub fn require_all<Q: AsRef<str>>(&self, name: Q) -> Result<Vec<&str>, HeaderError> {
        let name = name.as_ref();
        let values = self.get_all(name);
        if values.is_empty() {
            Err(HeaderError::Missing(name.to_string()))
        } else {
            Ok(values)
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

    /// Like [`remove`] but returns `Err(Missing)` on absence.
    ///
    /// [`remove`]: HeaderMap::remove
    pub fn require_remove<Q: AsRef<str>>(
        &mut self,
        name: Q,
    ) -> Result<HeaderValue, HeaderError> {
        let name = name.as_ref();
        self.inner
            .remove(name)
            .ok_or_else(|| HeaderError::Missing(name.to_string()))
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
        assert_eq!(
            map.get_only("content-length"),
            Err(HeaderError::MultipleValues("content-length".to_string()))
        );
    }

    #[test]
    fn get_only_parsed_invokes_closure_and_propagates_its_error() {
        let mut map = HeaderMap::new();
        map.insert(
            "content-length".to_string(),
            HeaderValue::Single("42".to_string()),
        );

        let ok: Result<Option<u64>, HeaderError> = map.get_only_parsed("content-length", |s| {
            s.parse()
                .map_err(|_| HeaderError::HeaderValueOverflow("content-length".to_string()))
        });
        assert!(matches!(ok, Ok(Some(42))));

        let err: Result<Option<u64>, HeaderError> =
            map.get_only_parsed("content-length", |_| {
                Err(HeaderError::InvalidHeaderValue("content-length".to_string()))
            });
        assert_eq!(
            err,
            Err(HeaderError::InvalidHeaderValue("content-length".to_string()))
        );
    }

    #[test]
    fn require_only_parsed_errors_on_absence() {
        let map = HeaderMap::new();
        let result: Result<String, HeaderError> =
            map.require_only_parsed("host", |s| Ok(s.to_string()));
        assert_eq!(result, Err(HeaderError::Missing("host".to_string())));
    }

    #[test]
    fn require_variants_error_on_absence() {
        let map = HeaderMap::new();
        assert!(matches!(
            map.require("host"),
            Err(HeaderError::Missing(name)) if name == "host"
        ));
        assert_eq!(
            map.require_only("host"),
            Err(HeaderError::Missing("host".to_string()))
        );
        assert_eq!(
            map.require_all("host"),
            Err(HeaderError::Missing("host".to_string()))
        );
    }

    #[test]
    fn get_parsed_sees_full_header_value() {
        let mut map = HeaderMap::new();
        map.insert(
            "set-cookie".to_string(),
            HeaderValue::Multiple(vec!["a=1".to_string(), "b=2".to_string()]),
        );
        let count: Result<Option<usize>, HeaderError> =
            map.get_parsed("set-cookie", |v| Ok(v.len()));
        assert!(matches!(count, Ok(Some(2))));

        let missing: Result<usize, HeaderError> =
            map.require_parsed("host", |v| Ok(v.len()));
        assert_eq!(missing, Err(HeaderError::Missing("host".to_string())));
    }

    #[test]
    fn require_remove_returns_value_when_present_and_errors_when_absent() {
        let mut map = HeaderMap::new();
        map.insert(
            "host".to_string(),
            HeaderValue::Single("example.com".to_string()),
        );

        let removed = map.require_remove("host").unwrap();
        assert!(matches!(removed, HeaderValue::Single(ref s) if s == "example.com"));
        assert!(!map.contains_key("host"));

        assert!(matches!(
            map.require_remove("host"),
            Err(HeaderError::Missing(name)) if name == "host"
        ));
    }

    #[test]
    fn accessors_accept_dynamic_name() {
        let mut map = HeaderMap::new();
        map.insert(
            "content-length".to_string(),
            HeaderValue::Single("42".to_string()),
        );

        let dynamic: String = String::from("content-length");
        assert!(map.get(&dynamic).is_some());
        assert!(matches!(map.get_only(&dynamic), Ok(Some("42"))));
        assert!(map.require(&dynamic).is_ok());
    }
}
