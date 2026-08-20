use core::iter::Peekable;

use proc_macro::{Delimiter, Ident, Literal, TokenStream, TokenTree};

use super::{
    groups::expect_group_consume_return_inner,
    tokens::{
        expect_any_ident, expect_punct_consume, expect_string_literal_consume, match_punct_consume,
    },
};

pub fn parse_fields_with<I, V, F>(
    cursor: &mut Peekable<I>,
    mut parse_value: F,
) -> Result<Vec<(Ident, V)>, TokenStream>
where
    I: Iterator<Item = TokenTree>,
    F: FnMut(&mut Peekable<I>) -> Result<V, TokenStream>,
{
    let mut fields = Vec::new();

    while cursor.peek().is_some() {
        let name = expect_any_ident(cursor, "expected field name (identifier)")?;
        expect_punct_consume(cursor, "=", "expected `=` after field name")?;
        let value = parse_value(cursor)?;
        fields.push((name, value));

        if !match_punct_consume(cursor, ",") {
            break;
        }
    }

    Ok(fields)
}

/// Parse a comma-separated list of `name = "value"` fields.
///
/// Values must be ordinary or raw Rust string literals. This is a pure syntax
/// parser: duplicate, unknown, and missing fields are validated separately by
/// `AttrLiteralFields`. Fields remain in source order, and a trailing comma is
/// accepted.
///
/// The parser stops after a field not followed by a comma. Callers parsing a
/// complete attribute must follow this with `expect_end` before constructing
/// `AttrLiteralFields`, so malformed trailing input cannot be ignored.
pub fn parse_string_literal_fields(
    cursor: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Vec<(Ident, Literal)>, TokenStream> {
    parse_fields_with(cursor, |cursor| expect_string_literal_consume(cursor))
}

/// Parse comma-separated fields whose values are bracket-delimited token streams.
///
/// The accepted grammar is `name = [tokens], other = [tokens]`. The square
/// brackets are parsing delimiters and are not included in the returned value,
/// so commas and generic arguments inside a value remain unambiguous. This is a
/// pure syntax parser; semantic field validation belongs to `AttrTokenFields`.
/// Fields remain in source order, and a trailing comma is accepted.
///
/// The parser stops after a field not followed by a comma. Callers parsing a
/// complete attribute must follow this with `expect_end` before constructing
/// `AttrTokenFields`, so malformed trailing input cannot be ignored.
pub fn parse_bracketed_token_fields(
    cursor: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Vec<(Ident, TokenStream)>, TokenStream> {
    parse_fields_with(cursor, |cursor| {
        expect_group_consume_return_inner(
            cursor,
            Delimiter::Bracket,
            "expected token field value enclosed in `[...]`",
        )
    })
}
