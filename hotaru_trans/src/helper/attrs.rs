use core::iter::Peekable;

use proc_macro::{Delimiter, Group, Punct, Spacing, Span, TokenStream, TokenTree};

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

/// Return the complete path at the start of an outer-attribute body.
///
/// For example, this returns `"cfg"` for `cfg(test)` and
/// `"rustfmt::skip"` for `rustfmt::skip`.
pub fn outer_attr_path(body: &TokenStream) -> Option<String> {
    let tokens = body.clone().into_iter().collect::<Vec<_>>();
    path_and_end(&tokens).map(|(path, _)| path)
}

/// Return whether an outer-attribute body has the exact requested path.
pub fn outer_attr_is_named<N>(body: &TokenStream, name: N) -> bool
where
    N: AsRef<str>,
{
    outer_attr_path(body).is_some_and(|path| path == name.as_ref())
}

/// Match an outer attribute of the exact form `name(...)`.
///
/// Returns the tokens inside the parentheses. A different path, a non-list
/// input, or tokens following the parenthesized input returns `None`.
pub fn match_outer_attr_list<N>(body: &TokenStream, name: N) -> Option<TokenStream>
where
    N: AsRef<str>,
{
    let tokens = body.clone().into_iter().collect::<Vec<_>>();
    let (path, input_index) = path_and_end(&tokens)?;
    if path != name.as_ref() || tokens.len() != input_index + 1 {
        return None;
    }

    match &tokens[input_index] {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
            Some(group.stream())
        }
        _ => None,
    }
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

fn path_and_end(tokens: &[TokenTree]) -> Option<(String, usize)> {
    let TokenTree::Ident(first) = tokens.first()? else {
        return None;
    };

    let mut path = first.to_string();
    let mut index = 1;

    while matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == ':') {
        let Some(TokenTree::Punct(second_colon)) = tokens.get(index + 1) else {
            return None;
        };
        let Some(TokenTree::Ident(segment)) = tokens.get(index + 2) else {
            return None;
        };
        if second_colon.as_char() != ':' {
            return None;
        }

        path.push_str("::");
        path.push_str(&segment.to_string());
        index += 3;
    }

    Some((path, index))
}
