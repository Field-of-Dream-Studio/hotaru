mod send;

use proc_macro::{Ident, Span, TokenStream};

use crate::middleware::MWFunc;
use crate::outer_attr::OuterAttr;

use super::APHandlerDef;

/// Outpoint flavour: the DSL body becomes middleware before core's sender.
pub(crate) struct OutpointMW {
    def: APHandlerDef,
}

impl OutpointMW {
    pub(crate) fn new(def: APHandlerDef) -> Self {
        Self { def }
    }

    pub(crate) fn def(&self) -> &APHandlerDef {
        &self.def
    }

    pub(crate) fn rewritten_body(&self) -> TokenStream {
        send::rewrite_send(self.def.body().clone(), self.def.request())
    }

    pub(crate) fn into_def(self) -> APHandlerDef {
        self.def
    }

    /// Emit the local outpoint-body middleware inside the constructor:
    ///
    /// ```ignore
    /// #[allow(non_camel_case_types)]
    /// struct __HotaruBody;
    ///
    /// const _: () = {
    ///     type _Ctx = <PROTO as ::hotaru_core::protocol::Protocol>::Context;
    ///     impl ::hotaru_core::executable::middleware::AsyncMiddleware<_Ctx>
    ///         for __HotaruBody
    ///     { /* rewritten body */ }
    /// };
    /// ```
    ///
    /// Delegates to `MWFunc::expand` — the same emitter used by the
    /// standalone `middleware!` macro — so the generated `handle` signature
    /// stays in sync with core's `NextFn`/`BoxFuture` aliases across
    /// `spawn_send` / `spawn_local` modes.
    pub(crate) fn expand(&self) -> TokenStream {
        // Fixed name: this struct lives inside the outpoint constructor's own
        // `{}` scope, so no two outpoints collide.
        let mangled_name = Ident::new("__HotaruBody", Span::call_site());

        let mw_func = MWFunc::new(
            false, // private struct
            mangled_name,
            self.def.protocol().clone(),
            self.def.request().clone(),
            self.rewritten_body(), // send; already rewritten
            OuterAttr::new(Vec::new()), // no user attrs on middleware
        );
        mw_func.expand()
    }
}
