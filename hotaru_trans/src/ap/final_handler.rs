use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::use_core;

use super::APHandlerDef;

/// Endpoint flavour: the DSL body becomes core's final handler.
pub(crate) struct FinalHandler {
    def: APHandlerDef,
}

impl FinalHandler {
    pub(crate) fn new(def: APHandlerDef) -> Self {
        Self { def }
    }

    pub(crate) fn def(&self) -> &APHandlerDef {
        &self.def
    }

    pub(crate) fn into_def(self) -> APHandlerDef {
        self.def
    }

    /// Emit `<PROTO as ::hotaru_core::protocol::Protocol>::Context`.
    ///
    /// The `Protocol` path is wrapped in a `Delimiter::None` group so
    /// downstream `extend` keeps it as one logical unit, mirroring the idiom
    /// in `middleware/func.rs`.
    fn expand_context_type(&self) -> TokenStream {
        let protocol = self.def.protocol().clone();
        let mut out = TokenStream::new();
        out.extend([
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(protocol),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::None,
                use_core(&["protocol", "Protocol"]),
            )),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
        ]);
        out
    }

    /// Emit `req: &mut <PROTO as Protocol>::Context`.
    fn expand_body_param(&self) -> TokenStream {
        let request = self.def.request().clone();
        let mut out = TokenStream::new();
        out.extend([
            TokenTree::Ident(request),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
        ]);
        out.extend(self.expand_context_type());
        out
    }

    /// Emit
    /// `-> ::hotaru_core::marker::MaybeSendBoxFuture<'_, impl ::hotaru_core::protocol::EndpointOutcome<Ctx> + 'static + use<>>`.
    fn expand_body_return_type(&self) -> TokenStream {
        let mut out = TokenStream::new();
        out.extend([
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]);
        out.extend(use_core(&["marker", "MaybeSendBoxFuture"]));
        out.extend([
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("impl", Span::call_site())),
        ]);
        out.extend(use_core(&["protocol", "EndpointOutcome"]));
        out.extend([TokenTree::Punct(Punct::new('<', Spacing::Alone))]);
        out.extend(self.expand_context_type());
        out.extend([
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new('+', Spacing::Alone)),
            TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
            TokenTree::Ident(Ident::new("static", Span::call_site())),
            TokenTree::Punct(Punct::new('+', Spacing::Alone)),
            TokenTree::Ident(Ident::new("use", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]);
        out
    }

    /// Emit `{ ::hotaru_core::prelude::Box::pin(async move { <body> }) }`.
    fn expand_body_block(&self) -> TokenStream {
        let mut inner = TokenStream::new();
        inner.extend(use_core(&["prelude", "Box", "pin"]));
        let mut async_arg = TokenStream::new();
        async_arg.extend([
            TokenTree::Ident(Ident::new("async", Span::call_site())),
            TokenTree::Ident(Ident::new("move", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, self.def.body().clone())),
        ]);
        inner.extend([TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            async_arg,
        ))]);
        TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, inner)))
    }

    /// Emit the full local fn item:
    ///
    /// ```ignore
    /// fn __hotaru_body(
    ///     req: &mut <PROTO as ::hotaru_core::protocol::Protocol>::Context,
    /// ) -> ::hotaru_core::marker::MaybeSendBoxFuture<
    ///     '_,
    ///     impl ::hotaru_core::protocol::EndpointOutcome<
    ///         <PROTO as ::hotaru_core::protocol::Protocol>::Context,
    ///     > + 'static + use<>,
    /// > {
    ///     ::hotaru_core::prelude::Box::pin(async move { /* body */ })
    /// }
    /// ```
    ///
    /// The synthesised name `__hotaru_body` is safe because this item lives
    /// inside the outer `pub fn <name>() -> Endpoint<PROTO>` constructor
    /// scope; no two endpoints share it.
    pub(crate) fn expand(&self) -> TokenStream {
        let mut out = TokenStream::new();
        out.extend([
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new("__hotaru_body", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                self.expand_body_param(),
            )),
        ]);
        out.extend(self.expand_body_return_type());
        out.extend(self.expand_body_block());
        out
    }
}
