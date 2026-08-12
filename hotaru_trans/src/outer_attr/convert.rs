use proc_macro::TokenStream;

use crate::helper::outer_attr_path;

use super::{OuterAttr, OuterAttrError};

impl TryFrom<Vec<TokenStream>> for OuterAttr {
    type Error = OuterAttrError;

    fn try_from(attrs: Vec<TokenStream>) -> Result<Self, Self::Error> {
        for attr in &attrs {
            if outer_attr_path(attr).is_none() {
                return Err(OuterAttrError::expected_attribute_path(attr));
            }
        }

        Ok(Self { attrs })
    }
}

impl From<OuterAttr> for Vec<TokenStream> {
    fn from(attrs: OuterAttr) -> Self {
        attrs.attrs
    }
}

impl IntoIterator for OuterAttr {
    type Item = TokenStream;
    type IntoIter = <Vec<TokenStream> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.attrs.into_iter()
    }
}
