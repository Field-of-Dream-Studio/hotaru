#![cfg(feature = "trans")]

use hotaru_core::{executable::middleware::AsyncMiddleware, marker::Arc};
use hotaru_http::{HTTP, context::HttpReqCtx};
use hotaru_trans::middleware;

middleware! {
    #[doc = "first ordered attribute"]
    #[doc = "second ordered attribute"]
    #[rustfmt::skip]
    pub AttributeMiddleware<HTTP> {
        next(req).await
    }
}

#[test]
fn non_empty_outer_attributes_survive_real_macro_expansion() {
    let middleware: Arc<dyn AsyncMiddleware<HttpReqCtx>> = Arc::new(AttributeMiddleware);
    let _ = middleware;
}

#[cfg(feature = "auto-reg")]
mod cfg_propagation {
    use hotaru_trans::endpoint;

    endpoint! {
        APP.url("/cfg-disabled"),
        #[cfg(any())]
        pub cfg_disabled<HTTP> {
            unreachable!()
        }
    }

    endpoint! {
        APP.url("/cfg-attr-disabled"),
        #[cfg_attr(all(), cfg(any()))]
        pub cfg_attr_disabled<HTTP> {
            unreachable!()
        }
    }
}
