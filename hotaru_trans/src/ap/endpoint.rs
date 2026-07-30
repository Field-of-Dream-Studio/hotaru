use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::{config::Config, ctor::gen_ctor, helper::use_core, middleware::MWChain};

use super::{APParts, FinalHandler, ParsedAP, RouteAddress};

/// Parsed `endpoint!` definition.
pub(crate) struct Endpoint {
    parts: APParts,
    handler: FinalHandler,
}

impl Endpoint {
    pub(crate) fn new(address: RouteAddress, handler: FinalHandler) -> Self {
        Self {
            parts: APParts::new(address),
            handler,
        }
    }

    pub(crate) fn from_parsed(parsed: ParsedAP) -> Self {
        let (parts, handler) = parsed.into_parts();
        Self {
            parts,
            handler: FinalHandler::new(handler),
        }
    }

    pub(crate) fn with_middlewares(mut self, middlewares: MWChain) -> Self {
        self.parts = self.parts.with_middlewares(middlewares);
        self
    }

    pub(crate) fn with_config(mut self, config: Config) -> Self {
        self.parts = self.parts.with_config(config);
        self
    }

    pub(crate) fn ap_parts(&self) -> &APParts {
        &self.parts
    }

    pub(crate) fn handler(&self) -> &FinalHandler {
        &self.handler
    }

    pub(crate) fn into_parts(self) -> (APParts, FinalHandler) {
        (self.parts, self.handler)
    }

    /// Compiler-facing entry point for `endpoint! { ... }` (trans DSL).
    pub(crate) fn from_trans_input(input: TokenStream) -> TokenStream {
        let mut input = crate::helper::into_peekable_iter(input);
        match ParsedAP::from_trans_stream(&mut input) {
            Ok(parsed) => Endpoint::from_parsed(parsed).expand(),
            Err(error) => error,
        }
    }

    /// Compiler-facing entry point for `#[endpoint] fn ... { ... }` (semi-trans).
    pub(crate) fn from_semi_trans_input(input: TokenStream) -> TokenStream {
        let mut input = crate::helper::into_peekable_iter(input);
        match ParsedAP::from_semi_trans_stream(&mut input) {
            Ok(parsed) => Endpoint::from_parsed(parsed).expand(),
            Err(error) => error,
        }
    }

    /// Compiler-facing entry point for `#[endpoint(url)] fn ... { ... }` (attr).
    pub(crate) fn from_attr_input(attr: TokenStream, input: TokenStream) -> TokenStream {
        let mut attr = crate::helper::into_peekable_iter(attr);
        let mut input = crate::helper::into_peekable_iter(input);
        match ParsedAP::from_attr_stream(&mut attr, &mut input) {
            Ok(parsed) => Endpoint::from_parsed(parsed).expand(),
            Err(error) => error,
        }
    }

    /// Emit `::hotaru_core::executable::def::Endpoint<PROTO>`.
    fn expand_return_type(&self) -> TokenStream {
        let protocol = self.handler.def().protocol().clone();
        let mut out = TokenStream::new();
        out.extend(use_core(&["executable", "def", "Endpoint"]));
        out.extend([
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(protocol),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]);
        out
    }

    /// Emit `<attrs>{pub} fn <name>() -> <return_type>`.
    fn expand_signature(&self) -> TokenStream {
        let def = self.handler.def();
        let mut out = TokenStream::new();
        out.extend(def.attrs().reform());
        if def.is_pub() {
            out.extend([TokenTree::Ident(Ident::new("pub", Span::call_site()))]);
        }
        out.extend([
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(def.name().clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]);
        out.extend(self.expand_return_type());
        out
    }

    /// Emit `::hotaru_core::executable::def::Endpoint::endpoint("<url>", "<name>", __hotaru_body)`.
    fn expand_ctor_call(&self) -> TokenStream {
        let url = self.parts.address().url().clone();
        let name = self.handler.def().name();
        let mut args = TokenStream::new();
        args.extend([
            TokenTree::Literal(url),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Literal(Literal::string(&name.to_string())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("__hotaru_body", Span::call_site())),
        ]);
        let mut out = TokenStream::new();
        out.extend(use_core(&["executable", "def", "Endpoint", "endpoint"]));
        out.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, args))]);
        out
    }

    /// Emit the full endpoint constructor:
    ///
    /// ```ignore
    /// <attrs>{pub} fn <name>() -> Endpoint<PROTO> {
    ///     fn __hotaru_body(...) -> ... { Box::pin(async move { <body> }) }
    ///
    ///     Endpoint::endpoint("<url>", "<name>", __hotaru_body)
    ///         .with_url_mode(...).with_middlewares({...}).with_config({...})
    /// }
    /// ```
    pub(crate) fn expand(self) -> TokenStream {
        // Collect all `&self`-visible parts before moving out of self.
        let signature_tokens = self.expand_signature();
        let ctor_call_tokens = self.expand_ctor_call();
        let body_fn_tokens = self.handler.expand();
        let hook_tokens = self.expand_hook();

        // Consume parts for the suffix (MWChain/Config emitters are consume-only).
        let (parts, _handler) = self.into_parts();
        let suffix_tokens = parts.expand_suffix();

        // Body: <body_fn> <ctor_call> <suffix>
        let mut body = TokenStream::new();
        body.extend(body_fn_tokens);
        body.extend(ctor_call_tokens);
        body.extend(suffix_tokens);

        // Wrapper: <signature> { <body> } <hook>
        let mut out = signature_tokens;
        out.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
        out.extend(hook_tokens);
        out
    }
}

#[cfg(feature = "auto-reg")]
impl Endpoint {
    /// Emit `<cfg-attrs> #[<ctor-path>] fn __register_<name>()`.
    fn expand_hook_signature(&self) -> TokenStream {
        let def = self.handler.def();
        let register_name = Ident::new(
            &format!("__register_{}", def.name()),
            Span::call_site(),
        );

        let mut out = TokenStream::new();
        // #[cfg(...)] copied only if the user's attrs contain cfg/cfg_attr.
        out.extend(def.attrs().reform_cfg());
        // #[ctor] attribute for the current build shape.
        out.extend(gen_ctor());
        // fn __register_<name>()
        out.extend([
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(register_name),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        ]);
        out
    }

    /// Emit `{ if let Err(error) = <app>.bind(<name>) { debug_warn!(...); } }`.
    fn expand_hook_body(&self) -> TokenStream {
        let app = self.parts.address().app().clone();
        let name = self.handler.def().name().clone();

        // <app>.bind(<name>)
        let mut bind_call = TokenStream::new();
        bind_call.extend([
            TokenTree::Ident(app),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("bind", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from(TokenTree::Ident(name)),
            )),
        ]);

        // Err(error) = <bind_call>
        let mut err_pat = TokenStream::new();
        err_pat.extend([
            TokenTree::Ident(Ident::new("Err", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from(TokenTree::Ident(Ident::new(
                    "error",
                    Span::call_site(),
                ))),
            )),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        ]);
        err_pat.extend(bind_call);

        // ::hotaru_core::debug_warn!("hotaru auto-reg failed: {}", error);
        let mut warn_args = TokenStream::new();
        warn_args.extend([
            TokenTree::Literal(Literal::string("hotaru auto-reg failed: {}")),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("error", Span::call_site())),
        ]);
        let mut warn_call = TokenStream::new();
        warn_call.extend(use_core(&["debug_warn"]));
        warn_call.extend([
            TokenTree::Punct(Punct::new('!', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, warn_args)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        // if let <err_pat> { <warn_call> }
        let mut if_let = TokenStream::new();
        if_let.extend([
            TokenTree::Ident(Ident::new("if", Span::call_site())),
            TokenTree::Ident(Ident::new("let", Span::call_site())),
        ]);
        if_let.extend(err_pat);
        if_let.extend([TokenTree::Group(Group::new(Delimiter::Brace, warn_call))]);

        TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, if_let)))
    }

    /// Emit the full registration hook item (empty when `auto-reg` is off).
    fn expand_hook(&self) -> TokenStream {
        let mut out = self.expand_hook_signature();
        out.extend(self.expand_hook_body());
        out
    }
}

#[cfg(not(feature = "auto-reg"))]
impl Endpoint {
    /// Empty hook when `auto-reg` is disabled.
    fn expand_hook(&self) -> TokenStream {
        TokenStream::new()
    }
}
