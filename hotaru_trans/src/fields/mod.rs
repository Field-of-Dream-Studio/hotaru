#![allow(dead_code)]

use proc_macro::{Ident, Literal, TokenStream};

mod collection;
mod convert;
mod crud;
mod error;
mod extract;

pub use error::AttrFieldsError;

/// Owned collection of named attribute fields.
///
/// Construct through [`TryFrom<Vec<(Ident, V)>>`]. Conversion rejects duplicate
/// names at the second occurrence, keeping syntax parsing separate from field
/// validation.
pub struct AttrFields<V> {
    collection: collection::FieldCollection<Ident, V>,
}

/// String-literal specialization used with `parse_string_literal_fields`.
pub type AttrLiteralFields = AttrFields<Literal>;

/// Token-stream specialization used with `parse_bracketed_token_fields`.
pub type AttrTokenFields = AttrFields<TokenStream>;

#[cfg(test)]
mod test;
