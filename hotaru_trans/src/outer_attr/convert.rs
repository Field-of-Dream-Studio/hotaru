use proc_macro::TokenStream;

use super::{OuterAttr, OuterAttrError};

impl TryFrom<Vec<TokenStream>> for OuterAttr {
    type Error = OuterAttrError;

    fn try_from(attrs: Vec<TokenStream>) -> Result<Self, Self::Error> {
        let collection = super::collection::OuterAttrCollection::try_from_attrs(attrs)
            .map_err(|attr| OuterAttrError::expected_attribute_name(&attr))?;
        Ok(Self { collection })
    }
}

impl From<OuterAttr> for Vec<TokenStream> {
    fn from(attrs: OuterAttr) -> Self {
        attrs.collection.into_attrs()
    }
}

impl IntoIterator for OuterAttr {
    type Item = TokenStream;
    type IntoIter = <Vec<TokenStream> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.collection.into_attrs().into_iter()
    }
}
