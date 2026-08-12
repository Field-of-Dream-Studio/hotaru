use core::mem;
use proc_macro::TokenStream;

use crate::helper::{outer_attr_is_named, outer_attr_path};

use super::{OuterAttr, OuterAttrError};

impl OuterAttr {
    /// Return the number of stored attributes.
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    /// Return whether no attributes are stored.
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }

    /// Iterate over attribute bodies in source order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &TokenStream> + ExactSizeIterator {
        self.attrs.iter()
    }

    /// Return whether an attribute with the exact path exists.
    pub fn contains<N>(&self, name: N) -> bool
    where
        N: AsRef<str>,
    {
        self.get(name).is_some()
    }

    /// Return whether every requested attribute path exists.
    pub fn contains_all<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().all(|name| self.contains(name.as_ref()))
    }

    /// Return whether at least one requested attribute path exists.
    pub fn contains_any<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().any(|name| self.contains(name.as_ref()))
    }

    /// Count attributes with the exact path.
    pub fn count<N>(&self, name: N) -> usize
    where
        N: AsRef<str>,
    {
        self.get_all(name).count()
    }

    /// Borrow the first attribute with the exact path.
    pub fn get<N>(&self, name: N) -> Option<&TokenStream>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.attrs
            .iter()
            .find(|attr| outer_attr_is_named(attr, name))
    }

    /// Borrow the first match for every requested path in `names` order.
    pub fn get_many<N>(&self, names: &[N]) -> Vec<Option<&TokenStream>>
    where
        N: AsRef<str>,
    {
        names.iter().map(|name| self.get(name.as_ref())).collect()
    }

    /// Borrow every attribute with the exact path in source order.
    pub fn get_all<N>(&self, name: N) -> impl DoubleEndedIterator<Item = &TokenStream> + '_
    where
        N: AsRef<str>,
    {
        let name = name.as_ref().to_owned();
        self.attrs
            .iter()
            .filter(move |attr| outer_attr_is_named(attr, &name))
    }

    /// Append one validated attribute body.
    pub fn push(&mut self, attr: TokenStream) -> Result<(), OuterAttrError> {
        validate_attr(&attr)?;
        self.attrs.push(attr);
        Ok(())
    }

    /// Atomically append several validated attribute bodies.
    pub fn extend(&mut self, attrs: Vec<TokenStream>) -> Result<(), OuterAttrError> {
        for attr in &attrs {
            validate_attr(attr)?;
        }
        self.attrs.extend(attrs);
        Ok(())
    }

    /// Replace the first matching attribute while preserving its position.
    pub fn replace<N>(
        &mut self,
        name: N,
        attr: TokenStream,
    ) -> Result<Option<TokenStream>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        validate_attr(&attr)?;
        let name = name.as_ref();
        let Some(index) = self
            .attrs
            .iter()
            .position(|stored| outer_attr_is_named(stored, name))
        else {
            return Ok(None);
        };

        Ok(Some(mem::replace(&mut self.attrs[index], attr)))
    }

    /// Remove and return the first attribute with the exact path.
    pub fn remove<N>(&mut self, name: N) -> Option<TokenStream>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let index = self
            .attrs
            .iter()
            .position(|attr| outer_attr_is_named(attr, name))?;
        Some(self.attrs.remove(index))
    }

    /// Remove the first match for every requested path in `names` order.
    pub fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<TokenStream>>
    where
        N: AsRef<str>,
    {
        names
            .iter()
            .map(|name| self.remove(name.as_ref()))
            .collect()
    }

    /// Remove every attribute with the exact path, preserving source order.
    pub fn remove_all<N>(&mut self, name: N) -> Vec<TokenStream>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let mut retained = Vec::with_capacity(self.attrs.len());
        let mut removed = Vec::new();

        for attr in mem::take(&mut self.attrs) {
            if outer_attr_is_named(&attr, name) {
                removed.push(attr);
            } else {
                retained.push(attr);
            }
        }

        self.attrs = retained;
        removed
    }

    /// Remove all stored attributes.
    pub fn clear(&mut self) {
        self.attrs.clear();
    }
}

fn validate_attr(attr: &TokenStream) -> Result<(), OuterAttrError> {
    if outer_attr_path(attr).is_some() {
        Ok(())
    } else {
        Err(OuterAttrError::expected_attribute_path(attr))
    }
}
