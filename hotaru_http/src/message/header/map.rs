//! Typed collection of HTTP header name/value pairs.
//! Relies on the `HeaderValue` invariant: `Multiple` means multi-value regardless of element count.

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
    /// `Ok(Some(&str))` when stored as `Single`, `Err(MultipleValues)` when
    /// stored as `Multiple` (regardless of element count).
    pub fn get_only<Q: AsRef<str>>(&self, name: Q) -> Result<Option<&str>, HeaderError> {
        let name = name.as_ref();
        match self.inner.get(name) {
            None => Ok(None),
            Some(HeaderValue::Single(value)) => Ok(Some(value.as_str())),
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
