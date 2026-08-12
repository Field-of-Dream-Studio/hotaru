use proc_macro::TokenStream;

use crate::helper::{emit_outer_attr, outer_attr_is_named};

use super::OuterAttr;

impl OuterAttr {
    /// Emit every stored attribute with its leading `#` and brackets.
    pub fn emit(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for attr in &self.attrs {
            tokens.extend(emit_outer_attr(attr));
        }
        tokens
    }

    /// Emit only attributes controlling whether generated companion items exist.
    pub(crate) fn emit_cfg(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for attr in &self.attrs {
            if outer_attr_is_named(attr, "cfg") || outer_attr_is_named(attr, "cfg_attr") {
                tokens.extend(emit_outer_attr(attr));
            }
        }
        tokens
    }
}
