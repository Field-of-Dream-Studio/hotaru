/// Invariant: all keys have unique textual names.
///
/// Construction and every operation that can add a key preserve this
/// invariant.
pub(super) struct FieldCollection<K: ToString, V> {
    pairs: Vec<(K, V)>,
}

impl<K: ToString, V> FieldCollection<K, V> {
    /// Construct a collection from key-value pairs, rejecting duplicates.
    ///
    /// On error, the second duplicate key is returned.
    pub(super) fn try_from_pairs(pairs: Vec<(K, V)>) -> Result<Self, K> {
        let mut collection = Self { pairs: Vec::new() };

        for (key, value) in pairs {
            collection.insert(key, value)?;
        }

        Ok(collection)
    }

    pub(super) fn len(&self) -> usize {
        self.pairs.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub(super) fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> + ExactSizeIterator {
        self.pairs.iter().map(|(key, value)| (key, value))
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (&K, &mut V)> + ExactSizeIterator {
        self.pairs.iter_mut().map(|(key, value)| (&*key, value))
    }

    pub(super) fn contains<N>(&self, name: N) -> bool
    where
        N: AsRef<str>,
    {
        self.position(name).is_some()
    }

    pub(super) fn contains_all<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().all(|name| self.contains(name.as_ref()))
    }

    pub(super) fn contains_any<N>(&self, names: &[N]) -> bool
    where
        N: AsRef<str>,
    {
        names.iter().any(|name| self.contains(name.as_ref()))
    }

    pub(super) fn get<N>(&self, name: N) -> Option<&V>
    where
        N: AsRef<str>,
    {
        self.position(name).map(|index| &self.pairs[index].1)
    }

    pub(super) fn get_many<N>(&self, names: &[N]) -> Vec<Option<&V>>
    where
        N: AsRef<str>,
    {
        names.iter().map(|name| self.get(name.as_ref())).collect()
    }

    pub(super) fn get_mut<N>(&mut self, name: N) -> Option<&mut V>
    where
        N: AsRef<str>,
    {
        let index = self.position(name)?;
        Some(&mut self.pairs[index].1)
    }

    pub(super) fn insert(&mut self, key: K, value: V) -> Result<(), K> {
        if self.check_unique(&key).is_err() {
            return Err(key);
        }

        self.pairs.push((key, value));
        Ok(())
    }

    pub(super) fn insert_many(&mut self, pairs: Vec<(K, V)>) -> Result<(), K> {
        let mut candidates = Self::try_from_pairs(pairs)?;

        for index in 0..candidates.len() {
            if self.check_unique(&candidates.pairs[index].0).is_err() {
                return Err(candidates.pairs.remove(index).0);
            }
        }

        self.pairs.extend(candidates.into_pairs());
        Ok(())
    }

    pub(super) fn replace<N>(&mut self, name: N, value: V) -> Option<V>
    where
        N: AsRef<str>,
    {
        let index = self.position(name)?;
        Some(core::mem::replace(&mut self.pairs[index].1, value))
    }

    pub(super) fn upsert(&mut self, key: K, value: V) -> Option<V> {
        match self.check_unique(&key) {
            Ok(()) => {
                self.pairs.push((key, value));
                None
            }
            Err(index) => Some(core::mem::replace(&mut self.pairs[index].1, value)),
        }
    }

    pub(super) fn remove<N>(&mut self, name: N) -> Option<V>
    where
        N: AsRef<str>,
    {
        let index = self.position(name)?;
        Some(self.pairs.remove(index).1)
    }

    pub(super) fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<V>>
    where
        N: AsRef<str>,
    {
        names
            .iter()
            .map(|name| self.remove(name.as_ref()))
            .collect()
    }

    pub(super) fn clear(&mut self) {
        self.pairs.clear();
    }

    pub(super) fn first_not_in<N>(&self, allowed: &[N]) -> Option<&K>
    where
        N: AsRef<str>,
    {
        self.pairs.iter().find_map(|(key, _)| {
            let name = key.to_string();
            (!allowed.iter().any(|allowed| allowed.as_ref() == name)).then_some(key)
        })
    }

    pub(super) fn into_pairs(self) -> Vec<(K, V)> {
        self.pairs
    }

    fn check_unique(&self, key: &K) -> Result<(), usize> {
        match self.position(key.to_string()) {
            Some(index) => Err(index),
            None => Ok(()),
        }
    }

    fn position<N>(&self, name: N) -> Option<usize>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.pairs
            .iter()
            .position(|(key, _)| key.to_string() == name)
    }
}
