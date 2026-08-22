use core::iter::Peekable;

use proc_macro::{TokenStream, TokenTree};

use crate::{
    helper::{expect_end, expect_stream_before_comma_consume, parse_outer_attr_bodies},
    outer_attr::OuterAttr,
};

use super::{APHandlerDef, APParts};

/// Fully parsed components shared before endpoint/outpoint wrapping.
pub(crate) struct ParsedAP {
    parts: APParts,
    handler: APHandlerDef,
}

impl ParsedAP {
    pub(crate) fn new(parts: APParts, handler: APHandlerDef) -> Self {
        Self { parts, handler }
    }

    pub(crate) fn into_parts(self) -> (APParts, APHandlerDef) {
        (self.parts, self.handler)
    }

    /// Parses one complete trans-syntax AP envelope.
    pub(crate) fn from_trans_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let address_fragment = expect_stream_before_comma_consume(
            tokens,
            true,
            "expected `,` after the route address",
        )?;
        let parts = APParts::from_stream(address_fragment, tokens)?;
        let handler = APHandlerDef::from_trans_stream(tokens)?;
        Ok(Self::new(parts, handler))
    }

    /// Parses one complete semi-trans function item.
    pub(crate) fn from_semi_trans_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let mut attrs = OuterAttr::try_from(parse_outer_attr_bodies(tokens)?)?;
        let parts = APParts::from_outer_attrs(&mut attrs)?;
        let handler = APHandlerDef::from_fn_stream(tokens, attrs)?;
        Ok(Self::new(parts, handler))
    }

    /// Parses attribute arguments and their complete function item.
    pub(crate) fn from_attr_stream(
        attr: &mut Peekable<impl Iterator<Item = TokenTree>>,
        item: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let address_fragment =
            expect_stream_before_comma_consume(attr, false, "expected a route address")?;
        let parts = APParts::from_stream(address_fragment, attr)?;
        expect_end(attr, "unexpected token after the AP attributes")?;
        let handler = APHandlerDef::from_fn_item_stream(item)?;
        Ok(Self::new(parts, handler))
    }
}
