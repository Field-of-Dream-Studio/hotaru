use proc_macro::Ident;

use super::{AttrFields, AttrFieldsError};

pub(super) fn position<K, V, N>(pairs: &[(K, V)], name: N) -> Option<usize>
where
    K: ToString,
    N: AsRef<str>,
{
    let name = name.as_ref();
    pairs.iter().position(|(key, _)| key.to_string() == name)
}

pub(super) fn first_candidate_duplicate<'a, K, V>(
    existing: &[(K, V)],
    candidates: &'a [(K, V)],
) -> Option<&'a K>
where
    K: ToString,
{
    candidates.iter().enumerate().find_map(|(index, (key, _))| {
        let name = key.to_string();
        (position(existing, name.as_str()).is_some()
            || candidates[..index]
                .iter()
                .any(|(previous, _)| previous.to_string() == name))
        .then_some(key)
    })
}

pub(super) fn replace_value<K, V, N>(pairs: &mut [(K, V)], name: N, value: V) -> Option<V>
where
    K: ToString,
    N: AsRef<str>,
{
    let index = position(pairs, name)?;
    Some(core::mem::replace(&mut pairs[index].1, value))
}

pub(super) fn upsert_value<K, V>(pairs: &mut Vec<(K, V)>, key: K, value: V) -> Option<V>
where
    K: ToString,
{
    let name = key.to_string();
    if let Some(index) = position(pairs, name.as_str()) {
        Some(core::mem::replace(&mut pairs[index].1, value))
    } else {
        pairs.push((key, value));
        None
    }
}

pub(super) fn remove_value<K, V, N>(pairs: &mut Vec<(K, V)>, name: N) -> Option<V>
where
    K: ToString,
    N: AsRef<str>,
{
    let index = position(pairs, name)?;
    Some(pairs.remove(index).1)
}

impl<V> AttrFields<V> {
    /// Return the number of stored fields.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Return `true` when no fields are stored.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Iterate over fields in source order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&Ident, &V)> + ExactSizeIterator {
        self.pairs.iter().map(|(key, value)| (key, value))
    }

    /// Mutably iterate over values in source order while keeping keys immutable.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (&Ident, &mut V)> + ExactSizeIterator {
        self.pairs.iter_mut().map(|(key, value)| (&*key, value))
    }

    /// Return whether a field with `name` exists.
    pub fn contains<N>(&self, name: N) -> bool
    where
        N: AsRef<str>,
    {
        position(&self.pairs, name).is_some()
    }

    /// Return whether every requested field exists.
    pub fn contains_all<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().all(|name| self.contains(name.as_ref()))
    }

    /// Return whether at least one requested field exists.
    pub fn contains_any<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().any(|name| self.contains(name.as_ref()))
    }

    /// Borrow the value stored under `name`.
    pub fn get<N>(&self, name: N) -> Option<&V>
    where
        N: AsRef<str>,
    {
        position(&self.pairs, name).map(|index| &self.pairs[index].1)
    }

    /// Borrow several values in `names` order.
    ///
    /// The returned vector has exactly one entry per requested name.
    pub fn get_many<N>(&self, names: &[N]) -> Vec<Option<&V>>
    where
        N: AsRef<str>,
    {
        names.iter().map(|name| self.get(name.as_ref())).collect()
    }

    /// Mutably borrow the value stored under `name`.
    pub fn get_mut<N>(&mut self, name: N) -> Option<&mut V>
    where
        N: AsRef<str>,
    {
        let index = position(&self.pairs, name)?;
        Some(&mut self.pairs[index].1)
    }

    /// Insert a new field at the end of the collection.
    ///
    /// An existing name is reported as a duplicate at the supplied key's span.
    pub fn insert(&mut self, key: Ident, value: V) -> Result<(), AttrFieldsError> {
        let name = key.to_string();
        if position(&self.pairs, name.as_str()).is_some() {
            return Err(AttrFieldsError::duplicate(&key));
        }

        self.pairs.push((key, value));
        Ok(())
    }

    /// Insert several fields atomically at the end of the collection.
    ///
    /// The complete batch is checked against existing fields and itself before
    /// mutation. On error, `self` is unchanged and the second occurrence is
    /// reported.
    pub fn insert_many(&mut self, pairs: Vec<(Ident, V)>) -> Result<(), AttrFieldsError> {
        if let Some(key) = first_candidate_duplicate(&self.pairs, &pairs) {
            return Err(AttrFieldsError::duplicate(key));
        }

        self.pairs.extend(pairs);
        Ok(())
    }

    /// Replace an existing value without changing its key, span, or position.
    pub fn replace<N>(&mut self, name: N, value: V) -> Option<V>
    where
        N: AsRef<str>,
    {
        replace_value(&mut self.pairs, name, value)
    }

    /// Replace an existing value or append a new field.
    ///
    /// When the name already exists, the original key, span, and position are
    /// preserved; the supplied key is used only when appending.
    pub fn upsert(&mut self, key: Ident, value: V) -> Option<V> {
        upsert_value(&mut self.pairs, key, value)
    }

    /// Remove and return a field value.
    pub fn remove<N>(&mut self, name: N) -> Option<V>
    where
        N: AsRef<str>,
    {
        remove_value(&mut self.pairs, name)
    }

    /// Remove several fields in `names` order.
    ///
    /// The returned vector has exactly one entry per requested name.
    pub fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<V>>
    where
        N: AsRef<str>,
    {
        names
            .iter()
            .map(|name| self.remove(name.as_ref()))
            .collect()
    }

    /// Remove every stored field.
    pub fn clear(&mut self) {
        self.pairs.clear();
    }
}
