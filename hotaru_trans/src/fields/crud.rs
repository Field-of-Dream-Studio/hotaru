use proc_macro::Ident;

use super::{AttrFields, AttrFieldsError};

impl<V> AttrFields<V> {
    /// Return the number of stored fields.
    pub fn len(&self) -> usize {
        self.collection.len()
    }

    /// Return `true` when no fields are stored.
    pub fn is_empty(&self) -> bool {
        self.collection.is_empty()
    }

    /// Iterate over fields in source order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&Ident, &V)> + ExactSizeIterator {
        self.collection.iter()
    }

    /// Mutably iterate over values in source order while keeping keys immutable.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (&Ident, &mut V)> + ExactSizeIterator {
        self.collection.iter_mut()
    }

    /// Return whether a field with `name` exists.
    pub fn contains<N>(&self, name: N) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains(name)
    }

    /// Return whether every requested field exists.
    pub fn contains_all<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains_all(names)
    }

    /// Return whether at least one requested field exists.
    pub fn contains_any<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        self.collection.contains_any(names)
    }

    /// Borrow the value stored under `name`.
    pub fn get<N>(&self, name: N) -> Option<&V>
    where
        N: AsRef<str>,
    {
        self.collection.get(name)
    }

    /// Borrow several values in `names` order.
    ///
    /// The returned vector has exactly one entry per requested name.
    pub fn get_many<N>(&self, names: &[N]) -> Vec<Option<&V>>
    where
        N: AsRef<str>,
    {
        self.collection.get_many(names)
    }

    /// Mutably borrow the value stored under `name`.
    pub fn get_mut<N>(&mut self, name: N) -> Option<&mut V>
    where
        N: AsRef<str>,
    {
        self.collection.get_mut(name)
    }

    /// Insert a new field at the end of the collection.
    ///
    /// An existing name is reported as a duplicate at the supplied key's span.
    pub fn insert(&mut self, key: Ident, value: V) -> Result<(), AttrFieldsError> {
        self.collection
            .insert(key, value)
            .map_err(|key| AttrFieldsError::duplicate(&key))
    }

    /// Insert several fields atomically at the end of the collection.
    ///
    /// The complete batch is checked against existing fields and itself before
    /// mutation. On error, `self` is unchanged and the second occurrence is
    /// reported.
    pub fn insert_many(&mut self, pairs: Vec<(Ident, V)>) -> Result<(), AttrFieldsError> {
        self.collection
            .insert_many(pairs)
            .map_err(|key| AttrFieldsError::duplicate(&key))
    }

    /// Replace an existing value without changing its key, span, or position.
    pub fn replace<N>(&mut self, name: N, value: V) -> Option<V>
    where
        N: AsRef<str>,
    {
        self.collection.replace(name, value)
    }

    /// Replace an existing value or append a new field.
    ///
    /// When the name already exists, the original key, span, and position are
    /// preserved; the supplied key is used only when appending.
    pub fn upsert(&mut self, key: Ident, value: V) -> Option<V> {
        self.collection.upsert(key, value)
    }

    /// Remove and return a field value.
    pub fn remove<N>(&mut self, name: N) -> Option<V>
    where
        N: AsRef<str>,
    {
        self.collection.remove(name)
    }

    /// Remove several fields in `names` order.
    ///
    /// The returned vector has exactly one entry per requested name.
    pub fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<V>>
    where
        N: AsRef<str>,
    {
        self.collection.remove_many(names)
    }

    /// Remove every stored field.
    pub fn clear(&mut self) {
        self.collection.clear();
    }
}
