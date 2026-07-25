use proc_macro::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn use_core<A: AsRef<str>>(path: &[A]) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    #[cfg(feature = "facade")]
    {
        ts.extend([TokenTree::Ident(Ident::new("hotaru", Span::call_site()))]);
        ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
        ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    }
    ts.extend([TokenTree::Ident(Ident::new(
        "hotaru_core",
        Span::call_site(),
    ))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    for (index, segment) in path.iter().enumerate() {
        ts.extend([TokenTree::Ident(Ident::new(
            &segment.as_ref(),
            Span::call_site(),
        ))]);
        if index + 1 < path.len() {
            ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
            ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
        }
    }
    ts
}

/// Emit an absolute path to a symbol re-exported through `hotaru_trans`.
///
/// Mirrors `use_core`: `::hotaru_trans::<path>` in direct mode,
/// `::hotaru::hrt::<path>` in facade mode. Attribute paths in Rust 2018+
/// accept absolute paths (`#[::hotaru_trans::ctor]` is valid), so this
/// helper is usable in both expression and attribute positions.
pub fn use_trans<A: AsRef<str>>(path: &[A]) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
    ]);
    #[cfg(feature = "facade")]
    {
        ts.extend([TokenTree::Ident(Ident::new("hotaru", Span::call_site()))]);
        ts.extend([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        ]);
        ts.extend([TokenTree::Ident(Ident::new("hrt", Span::call_site()))]);
    }
    #[cfg(not(feature = "facade"))]
    {
        ts.extend([TokenTree::Ident(Ident::new(
            "hotaru_trans",
            Span::call_site(),
        ))]);
    }
    for segment in path {
        ts.extend([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        ]);
        ts.extend([TokenTree::Ident(Ident::new(
            segment.as_ref(),
            Span::call_site(),
        ))]);
    }
    ts
}
