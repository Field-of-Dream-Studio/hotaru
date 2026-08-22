use proc_macro::TokenStream;

use crate::helper::match_outer_attr_list;

use super::{OuterAttr, OuterAttrError};

impl OuterAttr {
    /// Take an optional parenthesized attribute of the form `#[name(...)]`.
    ///
    /// Zero matches returns `None`; a second match is a duplicate error. The
    /// collection remains unchanged when validation fails.
    ///
    /// ```ignore
    /// let middleware = attrs.take_optional("middleware")?;
    /// ```
    pub fn take_optional<N>(&mut self, name: N) -> Result<Option<TokenStream>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let (first, second) = self.collection.first_two_matching_indices(name);

        if let Some(second) = second {
            return Err(OuterAttrError::duplicate(
                name,
                self.collection.get_at(second),
            ));
        }

        let Some(index) = first else {
            return Ok(None);
        };
        let attr = self.collection.get_at(index);
        let arguments = match_outer_attr_list(attr, name)
            .ok_or_else(|| OuterAttrError::expected_list(name, attr))?;
        self.collection.remove_at(index);
        Ok(Some(arguments))
    }

    /// Take a required parenthesized attribute of the form `#[name(...)]`.
    ///
    /// A missing or duplicate attribute is an error. The collection remains
    /// unchanged when validation fails.
    ///
    /// ```ignore
    /// let url = attrs.take_required("url")?;
    /// ```
    pub fn take_required<N>(&mut self, name: N) -> Result<TokenStream, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.take_optional(name)?
            .ok_or_else(|| OuterAttrError::missing_required(name))
    }

    /// Atomically take several optional parenthesized attributes in `names` order.
    pub fn take_optional_many<N>(
        &mut self,
        names: &[N],
    ) -> Result<Vec<Option<TokenStream>>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let mut candidate = self.clone();
        let mut values = Vec::with_capacity(names.len());

        for name in names {
            values.push(candidate.take_optional(name.as_ref())?);
        }

        *self = candidate;
        Ok(values)
    }

    /// Atomically take several required parenthesized attributes in `names` order.
    pub fn take_required_many<N>(&mut self, names: &[N]) -> Result<Vec<TokenStream>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let mut candidate = self.clone();
        let mut values = Vec::with_capacity(names.len());

        for name in names {
            values.push(candidate.take_required(name.as_ref())?);
        }

        *self = candidate;
        Ok(values)
    }
}
