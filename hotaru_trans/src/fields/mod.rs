#![allow(dead_code)]

use proc_macro::{Ident, Literal};

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
    pairs: Vec<(Ident, V)>,
}

/// String-literal specialization used with `parse_kv_pairs`.
pub type AttrLiteralFields = AttrFields<Literal>;

#[cfg(test)]
mod test;
