use core::iter::Peekable;

use proc_macro::{Delimiter, Span, TokenStream, TokenTree};

use crate::generate_compile_error;

use super::{Cloneable, Config};

impl Config {
    /// Parse the bracketed value after a configuration clause's separator.
    ///
    /// The caller owns the preceding `config` keyword and separator, as well as
    /// any tokens following the bracketed list.
    pub(crate) fn from_stream(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        cloneable: Cloneable,
    ) -> Result<Self, TokenStream> {
        // TODO: Parse commas inside ungrouped generic arguments correctly
        // (for example, `Factory::<A, B>::new()`).
        let entries = split_config_entries(stream, "Expected an array for config")?;
        Ok(Self::new(entries, cloneable))
    }
}

// TODO: also consider the edge case: "->"
fn split_config_entries<T: AsRef<str>>(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    error: T,
) -> Result<Vec<TokenStream>, TokenStream> {
    match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
            let mut array = Vec::new();
            let mut current = TokenStream::new();
            let mut inside_tokens = group.stream().into_iter().peekable();
            let mut angle_depth: usize = 0;
            loop {
                match inside_tokens.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                        match inside_tokens.peek() {
                            Some(TokenTree::Punct(p)) if p.as_char() == '=' => {},
                            _ => angle_depth += 1,
                        }
                        current.extend(core::iter::once(punct))
                    }
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                        match inside_tokens.peek() {
                            Some(TokenTree::Punct(p)) if p.as_char() == '=' => {},
                            _ => angle_depth -= 1,
                        }
                        current.extend(core::iter::once(punct))
                    }
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ',' && angle_depth == 0 => {
                        if current.is_empty() {
                            return Err(generate_compile_error(punct.span(), error.as_ref()));
                        }
                        array.push(current);
                        current = TokenStream::new();
                    }
                    Some(token) => current.extend(core::iter::once(token)),
                    None => {
                        if !current.is_empty() {
                            array.push(current);
                        }
                        break;
                    }
                }
            }
            Ok(array)
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}
