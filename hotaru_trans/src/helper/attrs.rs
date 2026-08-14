use core::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use super::generate_compile_error;

/// Parse consecutive outer attributes from the cursor.
///
/// Each returned stream excludes the leading `#` and surrounding brackets.
/// Parsing stops before the first token that is not the start of an attribute.
/// Inner attributes (`#![...]`) are rejected.
pub fn parse_outer_attr_bodies(
    cursor: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Vec<TokenStream>, TokenStream> {
    let mut attrs = Vec::new();

    while matches!(cursor.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '#') {
        let hash = cursor
            .next()
            .expect("the cursor was just checked for an attribute marker");

        if matches!(cursor.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '!') {
            let bang = cursor
                .next()
                .expect("the cursor was just checked for an inner-attribute marker");
            return Err(generate_compile_error(
                bang.span(),
                "inner attributes (#![...]) are not supported here",
            ));
        }

        match cursor.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                attrs.push(group.stream());
            }
            Some(token) => {
                return Err(generate_compile_error(
                    token.span(),
                    "expected attribute group after `#`",
                ));
            }
            None => {
                return Err(generate_compile_error(
                    hash.span(),
                    "expected attribute group after `#`",
                ));
            }
        }
    }

    Ok(attrs)
}

/// Clone the first identifier in an outer-attribute body.
///
/// For example, this returns `cfg` for `cfg(test)` and `rustfmt` for
/// `rustfmt::skip`.
pub fn outer_attr_name(body: &TokenStream) -> Option<Ident> {
    match body.clone().into_iter().next() {
        Some(TokenTree::Ident(ident)) => Some(ident),
        _ => None,
    }
}

/// Match an outer attribute of the exact form `name(...)`.
///
/// Returns the tokens inside the parentheses. A different name, a non-list
/// input, or tokens following the parenthesized input returns `None`.
pub fn match_outer_attr_list<N>(body: &TokenStream, name: N) -> Option<TokenStream>
where
    N: AsRef<str>,
{
    let mut tokens = body.clone().into_iter();
    let Some(TokenTree::Ident(stored)) = tokens.next() else {
        return None;
    };
    if stored.to_string() != name.as_ref() {
        return None;
    }

    let arguments = match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            group.stream()
        }
        _ => return None,
    };

    tokens.next().is_none().then_some(arguments)
}

/// Add the leading `#` and surrounding brackets to one attribute body.
pub fn emit_outer_attr(body: &TokenStream) -> TokenStream {
    let span = body
        .clone()
        .into_iter()
        .next()
        .map(|token| token.span())
        .unwrap_or_else(Span::call_site);

    let mut hash = Punct::new('#', Spacing::Alone);
    hash.set_span(span);
    let mut group = Group::new(Delimiter::Bracket, body.clone());
    group.set_span(span);

    [TokenTree::Punct(hash), TokenTree::Group(group)]
        .into_iter()
        .collect()
}
