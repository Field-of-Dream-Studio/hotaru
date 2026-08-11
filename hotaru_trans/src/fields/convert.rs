use proc_macro::Ident;

use super::{AttrFields, AttrFieldsError};

pub(super) fn first_duplicate<K, V>(pairs: &[(K, V)]) -> Option<&K>
where
    K: ToString,
{
    pairs.iter().enumerate().find_map(|(index, (key, _))| {
        let name = key.to_string();
        pairs[..index]
            .iter()
            .any(|(previous, _)| previous.to_string() == name)
            .then_some(key)
    })
}

impl<V> TryFrom<Vec<(Ident, V)>> for AttrFields<V> {
    type Error = AttrFieldsError;

    fn try_from(pairs: Vec<(Ident, V)>) -> Result<Self, Self::Error> {
        if let Some(key) = first_duplicate(&pairs) {
            return Err(AttrFieldsError::duplicate(key));
        }

        Ok(Self { pairs })
    }
}

impl<V> From<AttrFields<V>> for Vec<(Ident, V)> {
    fn from(fields: AttrFields<V>) -> Self {
        fields.pairs
    }
}

impl<V> IntoIterator for AttrFields<V> {
    type Item = (Ident, V);
    type IntoIter = <Vec<(Ident, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}
