#![allow(dead_code)]

use proc_macro::TokenStream;

mod convert;
mod crud;
mod emit;
mod error;
mod extract;
mod parse;

pub use error::OuterAttrError;
pub use parse::parse_outer_attrs;

/// Ordered collection of outer-attribute bodies.
///
/// Each stored stream excludes the leading `#` and surrounding brackets.
/// Duplicate attribute paths are valid and remain in source order.
#[derive(Clone, Default)]
pub struct OuterAttr {
    attrs: Vec<TokenStream>,
}

#[cfg(test)]
mod test;
