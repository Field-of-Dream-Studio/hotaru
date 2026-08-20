use proc_macro::TokenStream;

use super::{OuterAttr, OuterAttrError};

impl OuterAttr {
    /// Return the number of stored attributes.
    pub fn len(&self) -> usize {
        self.collection.len()
    }

    /// Return whether no attributes are stored.
    pub fn is_empty(&self) -> bool {
        self.collection.is_empty()
    }

    /// Iterate over attribute bodies in source order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &TokenStream> + ExactSizeIterator {
        self.collection.iter().map(|(_, attr)| attr)
    }

    /// Return whether an attribute with the requested leading name exists.
    pub fn contains<N>(&self, name: N) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains(name)
    }

    /// Return whether every requested attribute name exists.
    pub fn contains_all<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains_all(names)
    }

    /// Return whether at least one requested attribute name exists.
    pub fn contains_any<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains_any(names)
    }

    /// Count attributes with the requested leading name.
    pub fn count<N>(&self, name: N) -> usize
    where
        N: AsRef<str>,
    {
        self.collection.count(name)
    }

    /// Borrow the first attribute with the requested leading name.
    pub fn get<N>(&self, name: N) -> Option<&TokenStream>
    where
        N: AsRef<str>,
    {
        self.collection.get(name)
    }

    /// Borrow the first match for every requested name in `names` order.
    pub fn get_many<N>(&self, names: &[N]) -> Vec<Option<&TokenStream>>
    where
        N: AsRef<str>,
    {
        self.collection.get_many(names)
    }

    /// Borrow every attribute with the requested leading name in source order.
    pub fn get_all<N>(&self, name: N) -> impl DoubleEndedIterator<Item = &TokenStream> + '_
    where
        N: AsRef<str>,
    {
        self.collection.get_all(name)
    }

    /// Append one validated attribute body.
    pub fn push(&mut self, attr: TokenStream) -> Result<(), OuterAttrError> {
        self.collection
            .push(attr)
            .map_err(|attr| OuterAttrError::expected_attribute_name(&attr))
    }

    /// Atomically append several validated attribute bodies.
    pub fn extend(&mut self, attrs: Vec<TokenStream>) -> Result<(), OuterAttrError> {
        self.collection
            .extend(attrs)
            .map_err(|attr| OuterAttrError::expected_attribute_name(&attr))
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
        self.collection
            .replace(name, attr)
            .map_err(|attr| OuterAttrError::expected_attribute_name(&attr))
    }

    /// Remove and return the first attribute with the requested leading name.
    pub fn remove<N>(&mut self, name: N) -> Option<TokenStream>
    where
        N: AsRef<str>,
    {
        self.collection.remove(name)
    }

    /// Remove the first match for every requested name in `names` order.
    pub fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<TokenStream>>
    where
        N: AsRef<str>,
    {
        self.collection.remove_many(names)
    }

    /// Remove every attribute with the requested leading name, preserving order.
    pub fn remove_all<N>(&mut self, name: N) -> Vec<TokenStream>
    where
        N: AsRef<str>,
    {
        self.collection.remove_all(name)
    }

    /// Remove all stored attributes.
    pub fn clear(&mut self) {
        self.collection.clear();
    }
}
