use proc_macro::TokenStream;

use crate::helper::emit_outer_attr;

use super::OuterAttr;

impl OuterAttr {
    /// Emit every stored attribute with its leading `#` and brackets.
    pub fn emit(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (_, attr) in self.collection.iter() {
            tokens.extend(emit_outer_attr(attr));
        }
        tokens
    }

    /// Emit attributes whose names occur in `names`.
    ///
    /// Matching uses each attribute's first identifier. Source order and
    /// duplicate attributes are preserved.
    pub fn emit_only<N>(&self, names: &[N]) -> TokenStream
    where
        N: AsRef<str>,
    {
        let mut tokens = TokenStream::new();

        for (name, attr) in self.collection.iter() {
            let name = name.to_string();
            if names.iter().any(|allowed| allowed.as_ref() == name) {
                tokens.extend(emit_outer_attr(attr));
            }
        }

        tokens
    }

    /// Emit only attributes controlling whether generated companion items exist.
    pub(crate) fn emit_cfg(&self) -> TokenStream {
        self.emit_only(&["cfg", "cfg_attr"])
    }
}
