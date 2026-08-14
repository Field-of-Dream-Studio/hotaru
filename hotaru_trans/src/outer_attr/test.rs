use core::iter::Peekable;

use proc_macro::{TokenStream, TokenTree};

use crate::helper::parse_outer_attr_bodies;

use super::{OuterAttr, OuterAttrError};

#[test]
fn empty_outer_attr_exercises_runtime_safe_production_methods() {
    let mut attrs = OuterAttr::try_from(Vec::new()).expect("an empty collection is valid");

    assert_eq!(attrs.len(), 0);
    assert!(attrs.is_empty());
    assert_eq!(attrs.iter().count(), 0);
    assert!(!attrs.contains("missing"));
    assert!(attrs.contains_all::<&str>(&[]));
    assert!(!attrs.contains_any::<String>(&[]));
    assert_eq!(attrs.count("missing"), 0);
    assert!(attrs.get("missing").is_none());

    let missing = attrs.get_many(&["missing"]);
    assert_eq!(missing.len(), 1);
    assert!(missing[0].is_none());
    assert_eq!(attrs.get_all("missing").count(), 0);

    assert!(attrs.remove("missing").is_none());
    let missing = attrs.remove_many(&["missing"]);
    assert_eq!(missing.len(), 1);
    assert!(missing[0].is_none());
    assert!(attrs.remove_all("missing").is_empty());

    assert!(matches!(attrs.take_optional("missing"), Ok(None)));
    assert!(
        attrs
            .take_optional_many::<&str>(&[])
            .expect("an empty request succeeds")
            .is_empty()
    );
    assert!(
        attrs
            .take_required_many::<&str>(&[])
            .expect("an empty request succeeds")
            .is_empty()
    );

    attrs.clear();
    assert!(attrs.emit().is_empty());
    assert!(attrs.emit_only(&["cfg", "doc"]).is_empty());
    assert!(attrs.emit_cfg().is_empty());

    let streams: Vec<TokenStream> = attrs.into();
    assert!(streams.is_empty());
    assert_eq!(OuterAttr::default().into_iter().count(), 0);
}

#[test]
fn outer_attr_error_implements_the_standard_error_contract() {
    fn assert_error<E: core::error::Error>() {}
    assert_error::<OuterAttrError>();
}

// Compile the proc-macro-specific surface without constructing proc_macro
// handles, which rustc only permits while a procedural macro is executing.
#[allow(dead_code)]
fn assert_proc_macro_api_compiles<I>(
    cursor: &mut Peekable<I>,
    mut attrs: OuterAttr,
    body: TokenStream,
    error: OuterAttrError,
) -> Result<TokenStream, OuterAttrError>
where
    I: Iterator<Item = TokenTree>,
{
    let _ = parse_outer_attr_bodies(cursor).map(OuterAttr::try_from);
    attrs.push(body.clone())?;
    attrs.extend(vec![body.clone()])?;
    let _ = attrs.replace(String::from("cfg"), body)?;
    let _ = attrs.take_required(String::from("url"))?;
    let _ = error.name();
    let _ = error.span();
    Ok(error.into_compile_error())
}
