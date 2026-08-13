use proc_macro::Ident;

use super::{AttrFields, AttrFieldsError, collection::FieldCollection};

impl<V> TryFrom<Vec<(Ident, V)>> for AttrFields<V> {
    type Error = AttrFieldsError;

    fn try_from(pairs: Vec<(Ident, V)>) -> Result<Self, Self::Error> {
        FieldCollection::try_from_pairs(pairs)
            .map(|collection| Self { collection })
            .map_err(|key| AttrFieldsError::duplicate(&key))
    }
}

impl<V> From<AttrFields<V>> for Vec<(Ident, V)> {
    fn from(fields: AttrFields<V>) -> Self {
        fields.collection.into_pairs()
    }
}

impl<V> IntoIterator for AttrFields<V> {
    type Item = (Ident, V);
    type IntoIter = <Vec<(Ident, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.collection.into_pairs().into_iter()
    }
}
