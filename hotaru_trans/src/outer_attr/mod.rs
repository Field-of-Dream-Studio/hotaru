#![allow(dead_code)]

mod collection;
mod convert;
mod crud;
mod emit;
mod error;
mod extract;

pub use error::OuterAttrError;

/// Ordered collection of outer-attribute bodies.
///
/// Each stored stream excludes the leading `#` and surrounding brackets.
/// Lookup uses the first identifier as the attribute name. Duplicate names are
/// valid and remain in source order.
#[derive(Clone, Default)]
pub struct OuterAttr {
    collection: collection::OuterAttrCollection,
}

#[cfg(test)]
mod test;
