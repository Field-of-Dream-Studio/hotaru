use core::fmt::{self, Display, Formatter};
use proc_macro::{Span, TokenStream};

use crate::helper::generate_compile_error;

/// Semantic validation error produced by [`super::OuterAttr`].
#[derive(Clone, Debug)]
pub enum OuterAttrError {
    /// An attribute body did not begin with a valid attribute path.
    ExpectedAttributePath { span: Span },
    /// An operation requiring a unique list attribute found a second match.
    Duplicate { name: String, span: Span },
    /// A required list attribute was absent.
    MissingRequired { name: String, span: Span },
    /// A matching attribute was not exactly `#[name(...)]`.
    ExpectedList { name: String, span: Span },
}

impl OuterAttrError {
    pub(super) fn expected_attribute_path(attr: &TokenStream) -> Self {
        Self::ExpectedAttributePath {
            span: attr_span(attr),
        }
    }

    pub(super) fn duplicate(name: &str, attr: &TokenStream) -> Self {
        Self::Duplicate {
            name: name.to_owned(),
            span: attr_span(attr),
        }
    }

    pub(super) fn missing_required(name: &str) -> Self {
        Self::MissingRequired {
            name: name.to_owned(),
            span: Span::call_site(),
        }
    }

    pub(super) fn expected_list(name: &str, attr: &TokenStream) -> Self {
        Self::ExpectedList {
            name: name.to_owned(),
            span: attr_span(attr),
        }
    }

    /// Return the attribute path associated with this error, if present.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::ExpectedAttributePath { .. } => None,
            Self::Duplicate { name, .. }
            | Self::MissingRequired { name, .. }
            | Self::ExpectedList { name, .. } => Some(name),
        }
    }

    /// Return the source span at which this error should be reported.
    pub fn span(&self) -> Span {
        match self {
            Self::ExpectedAttributePath { span }
            | Self::Duplicate { span, .. }
            | Self::MissingRequired { span, .. }
            | Self::ExpectedList { span, .. } => *span,
        }
    }

    /// Convert this semantic error into a `compile_error!` token stream.
    pub fn into_compile_error(self) -> TokenStream {
        generate_compile_error(self.span(), &self.to_string())
    }
}

impl Display for OuterAttrError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedAttributePath { .. } => {
                formatter.write_str("expected an outer attribute path")
            }
            Self::Duplicate { name, .. } => {
                write!(formatter, "duplicate `#[{name}(...)]` attribute")
            }
            Self::MissingRequired { name, .. } => {
                write!(formatter, "missing required `#[{name}(...)]` attribute")
            }
            Self::ExpectedList { name, .. } => write!(formatter, "expected `#[{name}(...)]`"),
        }
    }
}

impl core::error::Error for OuterAttrError {}

impl From<OuterAttrError> for TokenStream {
    fn from(error: OuterAttrError) -> Self {
        error.into_compile_error()
    }
}

fn attr_span(attr: &TokenStream) -> Span {
    attr.clone()
        .into_iter()
        .next()
        .map(|token| token.span())
        .unwrap_or_else(Span::call_site)
}
