use core::fmt::{self, Display, Formatter};
use proc_macro::{Ident, Span, TokenStream};

use crate::helper::generate_compile_error;

/// Semantic validation error produced by [`super::AttrFields`].
#[derive(Clone, Debug)]
pub enum AttrFieldsError {
    /// A field name appeared more than once.
    Duplicate { name: String, span: Span },
    /// A field was not part of the accepted grammar.
    Unknown { name: String, span: Span },
    /// A required field was absent.
    MissingRequired { name: String, span: Span },
}

impl AttrFieldsError {
    pub(super) fn duplicate(key: &Ident) -> Self {
        Self::Duplicate {
            name: key.to_string(),
            span: key.span(),
        }
    }

    pub(super) fn unknown(key: &Ident) -> Self {
        Self::Unknown {
            name: key.to_string(),
            span: key.span(),
        }
    }

    pub(super) fn missing_required(name: &str) -> Self {
        Self::MissingRequired {
            name: name.to_owned(),
            span: Span::call_site(),
        }
    }

    /// Return the field name associated with this error.
    pub fn name(&self) -> &str {
        match self {
            Self::Duplicate { name, .. }
            | Self::Unknown { name, .. }
            | Self::MissingRequired { name, .. } => name,
        }
    }

    /// Return the source span at which this error should be reported.
    pub fn span(&self) -> Span {
        match self {
            Self::Duplicate { span, .. }
            | Self::Unknown { span, .. }
            | Self::MissingRequired { span, .. } => *span,
        }
    }

    /// Convert this semantic error into a `compile_error!` token stream.
    pub fn into_compile_error(self) -> TokenStream {
        generate_compile_error(self.span(), &self.to_string())
    }
}

impl Display for AttrFieldsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { name, .. } => write!(formatter, "duplicate field `{name}`"),
            Self::Unknown { name, .. } => write!(formatter, "unknown field `{name}`"),
            Self::MissingRequired { name, .. } => {
                write!(formatter, "missing required field `{name}`")
            }
        }
    }
}

impl core::error::Error for AttrFieldsError {}

impl From<AttrFieldsError> for TokenStream {
    fn from(error: AttrFieldsError) -> Self {
        error.into_compile_error()
    }
}
