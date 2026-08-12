use core::iter::Peekable;
use proc_macro::{TokenStream, TokenTree};

use crate::helper::parse_outer_attr_bodies;

use super::OuterAttr;

/// Parse consecutive outer attributes from the start of a cursor.
///
/// Each captured attribute is stored without its leading `#` and surrounding
/// brackets. The first non-attribute token remains in the cursor.
pub fn parse_outer_attrs(
    cursor: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<OuterAttr, TokenStream> {
    OuterAttr::try_from(parse_outer_attr_bodies(cursor)?).map_err(Into::into)
}
