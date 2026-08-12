use proc_macro::TokenStream;

use crate::helper::{match_outer_attr_list, outer_attr_is_named};

use super::{OuterAttr, OuterAttrError};

impl OuterAttr {
    /// Take an optional list attribute of the form `#[name(...)]`.
    ///
    /// Zero matches returns `None`; a second match is a duplicate error. The
    /// collection remains unchanged when validation fails.
    ///
    /// ```ignore
    /// let middleware = attrs.take_optional_list("middleware")?;
    /// ```
    pub fn take_optional_list<N>(&mut self, name: N) -> Result<Option<TokenStream>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let matching = self
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(index, attr)| outer_attr_is_named(attr, name).then_some(index))
            .collect::<Vec<_>>();

        if let Some(second) = matching.get(1).copied() {
            return Err(OuterAttrError::duplicate(name, &self.attrs[second]));
        }

        let Some(index) = matching.first().copied() else {
            return Ok(None);
        };
        let arguments = match_outer_attr_list(&self.attrs[index], name)
            .ok_or_else(|| OuterAttrError::expected_list(name, &self.attrs[index]))?;
        self.attrs.remove(index);
        Ok(Some(arguments))
    }

    /// Take a required list attribute of the form `#[name(...)]`.
    ///
    /// A missing or duplicate attribute is an error. The collection remains
    /// unchanged when validation fails.
    ///
    /// ```ignore
    /// let url = attrs.take_required_list("url")?;
    /// ```
    pub fn take_required_list<N>(&mut self, name: N) -> Result<TokenStream, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.take_optional_list(name)?
            .ok_or_else(|| OuterAttrError::missing_required(name))
    }

    /// Atomically take several optional list attributes in `names` order.
    pub fn take_optional_lists<N>(
        &mut self,
        names: &[N],
    ) -> Result<Vec<Option<TokenStream>>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let mut candidate = self.clone();
        let values = names
            .iter()
            .map(|name| candidate.take_optional_list(name.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        *self = candidate;
        Ok(values)
    }

    /// Atomically take several required list attributes in `names` order.
    pub fn take_required_lists<N>(
        &mut self,
        names: &[N],
    ) -> Result<Vec<TokenStream>, OuterAttrError>
    where
        N: AsRef<str>,
    {
        let mut candidate = self.clone();
        let values = names
            .iter()
            .map(|name| candidate.take_required_list(name.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        *self = candidate;
        Ok(values)
    }
}
