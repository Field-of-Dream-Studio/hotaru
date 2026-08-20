use super::{AttrFields, AttrFieldsError};

impl<V> AttrFields<V> {
    /// Reject the first field whose name is not in `allowed`.
    ///
    /// This check does not consume any fields, so callers can validate the
    /// accepted grammar before extracting required and optional values.
    pub fn reject_unknown<N>(&self, allowed: &[N]) -> Result<(), AttrFieldsError>
    where
        N: AsRef<str>,
    {
        if let Some(key) = self.collection.first_not_in(allowed) {
            return Err(AttrFieldsError::unknown(key));
        }

        Ok(())
    }

    /// Consume and return a required field.
    ///
    /// A missing field is reported at the macro call site because no field
    /// token exists from which to obtain a more precise span.
    pub fn take_required<N>(&mut self, name: N) -> Result<V, AttrFieldsError>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.take_optional(name)
            .ok_or_else(|| AttrFieldsError::missing_required(name))
    }

    /// Consume and return an optional field, or `None` when it is absent.
    pub fn take_optional<N>(&mut self, name: N) -> Option<V>
    where
        N: AsRef<str>,
    {
        self.remove(name)
    }

    /// Consume and return several required fields in `names` order.
    ///
    /// If a field is missing, values requested earlier in the slice have
    /// already been removed. Parser callers should return the error and discard
    /// this collection.
    pub fn take_required_many<N>(&mut self, names: &[N]) -> Result<Vec<V>, AttrFieldsError>
    where
        N: AsRef<str>,
    {
        let mut values = Vec::with_capacity(names.len());

        for name in names {
            values.push(self.take_required(name.as_ref())?);
        }

        Ok(values)
    }

    /// Consume several optional fields in `names` order.
    ///
    /// The returned vector has exactly one entry per requested name.
    pub fn take_optional_many<N>(&mut self, names: &[N]) -> Vec<Option<V>>
    where
        N: AsRef<str>,
    {
        self.remove_many(names)
    }

    /// Reject the first unconsumed field.
    ///
    /// Consuming `self` prevents extraction from continuing after this final
    /// exhaustiveness check.
    pub fn reject_rest(self) -> Result<(), AttrFieldsError> {
        if let Some((key, _)) = self.collection.into_pairs().into_iter().next() {
            return Err(AttrFieldsError::unknown(&key));
        }

        Ok(())
    }
}
