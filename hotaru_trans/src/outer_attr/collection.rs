use core::mem;

use proc_macro::{Ident, TokenStream, TokenTree};

/// Ordered storage for validated outer-attribute bodies.
///
/// Every body is indexed by its first identifier. Duplicate names are valid
/// and remain in source order.
#[derive(Clone, Default)]
pub(super) struct OuterAttrCollection {
    pairs: Vec<(Ident, TokenStream)>,
}

impl OuterAttrCollection {
    /// Construct from validated attribute bodies.
    ///
    /// On error, the first body without a valid leading identifier is returned.
    pub(super) fn try_from_attrs(attrs: Vec<TokenStream>) -> Result<Self, TokenStream> {
        let pairs = pairs_from_attrs(attrs)?;
        Ok(Self { pairs })
    }

    pub(super) fn len(&self) -> usize {
        self.pairs.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub(super) fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (&Ident, &TokenStream)> + ExactSizeIterator {
        self.pairs.iter().map(|(name, attr)| (name, attr))
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

    pub(super) fn count<N>(&self, name: N) -> usize
    where
        N: AsRef<str>,
    {
        self.get_all(name).count()
    }

    pub(super) fn get<N>(&self, name: N) -> Option<&TokenStream>
    where
        N: AsRef<str>,
    {
        self.position(name).map(|index| &self.pairs[index].1)
    }

    pub(super) fn get_many<N>(&self, names: &[N]) -> Vec<Option<&TokenStream>>
    where
        N: AsRef<str>,
    {
        names.iter().map(|name| self.get(name.as_ref())).collect()
    }

    pub(super) fn get_all<N>(&self, name: N) -> impl DoubleEndedIterator<Item = &TokenStream> + '_
    where
        N: AsRef<str>,
    {
        let name = name.as_ref().to_owned();
        self.pairs
            .iter()
            .filter_map(move |(stored, attr)| (stored.to_string() == name).then_some(attr))
    }

    pub(super) fn push(&mut self, attr: TokenStream) -> Result<(), TokenStream> {
        self.pairs.push(pair_from_attr(attr)?);
        Ok(())
    }

    /// Append several bodies atomically.
    pub(super) fn extend(&mut self, attrs: Vec<TokenStream>) -> Result<(), TokenStream> {
        self.pairs.extend(pairs_from_attrs(attrs)?);
        Ok(())
    }

    pub(super) fn replace<N>(
        &mut self,
        name: N,
        attr: TokenStream,
    ) -> Result<Option<TokenStream>, TokenStream>
    where
        N: AsRef<str>,
    {
        let replacement = pair_from_attr(attr)?;
        let Some(index) = self.position(name) else {
            return Ok(None);
        };

        let (_, previous) = mem::replace(&mut self.pairs[index], replacement);
        Ok(Some(previous))
    }

    pub(super) fn remove<N>(&mut self, name: N) -> Option<TokenStream>
    where
        N: AsRef<str>,
    {
        let index = self.position(name)?;
        Some(self.pairs.remove(index).1)
    }

    pub(super) fn remove_many<N>(&mut self, names: &[N]) -> Vec<Option<TokenStream>>
    where
        N: AsRef<str>,
    {
        names
            .iter()
            .map(|name| self.remove(name.as_ref()))
            .collect()
    }

    pub(super) fn remove_all<N>(&mut self, name: N) -> Vec<TokenStream>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let mut retained = Vec::with_capacity(self.pairs.len());
        let mut removed = Vec::new();

        for (stored, attr) in mem::take(&mut self.pairs) {
            if stored.to_string() == name {
                removed.push(attr);
            } else {
                retained.push((stored, attr));
            }
        }

        self.pairs = retained;
        removed
    }

    pub(super) fn clear(&mut self) {
        self.pairs.clear();
    }

    pub(super) fn first_two_matching_indices<N>(&self, name: N) -> (Option<usize>, Option<usize>)
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        let mut first = None;

        for (index, (stored, _)) in self.pairs.iter().enumerate() {
            if stored.to_string() != name {
                continue;
            }

            if first.is_some() {
                return (first, Some(index));
            }

            first = Some(index);
        }

        (first, None)
    }

    pub(super) fn get_at(&self, index: usize) -> &TokenStream {
        &self.pairs[index].1
    }

    pub(super) fn remove_at(&mut self, index: usize) -> TokenStream {
        self.pairs.remove(index).1
    }

    pub(super) fn into_attrs(self) -> Vec<TokenStream> {
        self.pairs.into_iter().map(|(_, attr)| attr).collect()
    }

    fn position<N>(&self, name: N) -> Option<usize>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.pairs
            .iter()
            .position(|(stored, _)| stored.to_string() == name)
    }
}

fn pairs_from_attrs(attrs: Vec<TokenStream>) -> Result<Vec<(Ident, TokenStream)>, TokenStream> {
    attrs.into_iter().map(pair_from_attr).collect()
}

fn pair_from_attr(attr: TokenStream) -> Result<(Ident, TokenStream), TokenStream> {
    match attr.clone().into_iter().next() {
        Some(TokenTree::Ident(name)) => Ok((name, attr)),
        _ => Err(attr),
    }
}
