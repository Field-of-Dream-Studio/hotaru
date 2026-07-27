use super::RouteAddress;

use core::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::{
    config::{Cloneable, Config},
    helper::{
        expect_end, expect_punct_consume, generate_compile_error, into_peekable_iter,
        match_punct_consume,
    },
    middleware::MWChain,
    outer_attr::OuterAttr,
};

/// Address, middleware, and config components shared by every AP flavour.
pub(crate) struct APParts {
    address: RouteAddress,
    middlewares: MWChain,
    config: Config,
}

impl APParts {
    /// Construct with the same default middleware chain as core: `[Inherit]`.
    pub(crate) fn new(address: RouteAddress) -> Self {
        Self {
            address,
            middlewares: MWChain::inheriting(),
            config: Config::new(Vec::new(), Cloneable::Yes),
        }
    }

    pub(crate) fn with_middlewares(mut self, middlewares: MWChain) -> Self {
        self.middlewares = middlewares;
        self
    }

    pub(crate) fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub(crate) fn address(&self) -> &RouteAddress {
        &self.address
    }

    pub(crate) fn middlewares(&self) -> &MWChain {
        &self.middlewares
    }

    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) fn into_parts(self) -> (RouteAddress, MWChain, Config) {
        (self.address, self.middlewares, self.config)
    }

    /// Emit the shared builder suffix chained onto every AP constructor call:
    ///
    /// ```ignore
    /// .with_url_mode(::hotaru_core::executable::def::UrlMode::Pattern)
    /// .with_middlewares({ /* MWChain::expand_middleware_chain */ })
    /// .with_config({ /* Config::expand */ })
    /// ```
    pub(crate) fn expand_suffix(self) -> TokenStream {
        let (address, middlewares, config) = self.into_parts();

        let mut out = TokenStream::new();

        // .with_url_mode(<UrlMode variant path>)
        out.extend([
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("with_url_mode", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                address.url_mode().expand(),
            )),
        ]);

        // .with_middlewares({ /* MWChain::expand_middleware_chain */ })
        // `expand_middleware_chain` already returns a `{ ... }` brace group.
        out.extend([
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("with_middlewares", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                middlewares.expand_middleware_chain(),
            )),
        ]);

        // .with_config({ /* Config::expand */ })
        // `Config::expand` also returns a `{ ... }` brace group.
        out.extend([
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("with_config", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, config.expand())),
        ]);

        out
    }

    /// Parses an isolated route address and its following inline clauses. (Trans and Attr)
    pub(crate) fn from_stream(
        address_fragment: TokenStream,
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let mut address_fragment = into_peekable_iter(address_fragment);
        let address = RouteAddress::from_stream(&mut address_fragment)?;
        let mut parts = Self::new(address);
        let mut saw_middleware = false;
        let mut saw_config = false;

        loop {
            while match_punct_consume(tokens, ",") {}

            let (clause, span) = match tokens.peek() {
                Some(TokenTree::Ident(ident))
                    if ident.to_string() == "middleware" || ident.to_string() == "config" =>
                {
                    (ident.to_string(), ident.span())
                }
                _ => break,
            };
            tokens.next();
            expect_punct_consume(tokens, "=", format!("expected `=` after `{clause}`"))?;

            match clause.as_str() {
                "middleware" if !saw_middleware => {
                    saw_middleware = true;
                    parts = parts.with_middlewares(MWChain::from_stream(tokens)?);
                }
                "config" if !saw_config => {
                    saw_config = true;
                    parts = parts.with_config(Config::from_stream(tokens, Cloneable::Yes)?);
                }
                _ => {
                    return Err(generate_compile_error(
                        span,
                        &format!("duplicate `{clause}` clause"),
                    ));
                }
            }
        }

        Ok(parts)
    }

    /// Extracts and parses AP-owned semi-trans attributes.
    pub(crate) fn from_outer_attrs(attrs: &mut OuterAttr) -> Result<Self, TokenStream> {
        let address_fragment = attrs.remove_unique_inner("url")?.ok_or_else(|| {
            generate_compile_error(
                Span::call_site(),
                "missing required `#[url(...)]` attribute",
            )
        })?;
        let middleware_fragment = attrs.remove_unique_inner("middleware")?;
        let config_fragment = attrs.remove_unique_inner("config")?;

        let mut no_inline_clauses = into_peekable_iter(TokenStream::new());
        let mut parts = Self::from_stream(address_fragment, &mut no_inline_clauses)?;

        if let Some(fragment) = middleware_fragment {
            let mut fragment = into_peekable_iter(fragment);
            let value = MWChain::from_stream(&mut fragment)?;
            expect_end(&mut fragment, "unexpected token after the middleware array")?;
            parts = parts.with_middlewares(value);
        }

        if let Some(fragment) = config_fragment {
            let mut fragment = into_peekable_iter(fragment);
            let value = Config::from_stream(&mut fragment, Cloneable::Yes)?;
            expect_end(&mut fragment, "unexpected token after the config array")?;
            parts = parts.with_config(value);
        }

        Ok(parts)
    }
}
