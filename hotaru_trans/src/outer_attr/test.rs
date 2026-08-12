use core::iter::Peekable;

use proc_macro::{TokenStream as ProcTokenStream, TokenTree as ProcTokenTree};
use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};

use super::{OuterAttr, OuterAttrError, parse_outer_attrs};

#[derive(Clone, Debug, Eq, PartialEq)]
enum TestError {
    ExpectedPath,
    Duplicate(String),
    MissingRequired(String),
    ExpectedList(String),
    InnerAttribute,
    ExpectedGroup,
}

#[derive(Clone, Default)]
struct TestOuterAttr {
    attrs: Vec<TokenStream>,
}

impl TestOuterAttr {
    fn try_from(attrs: Vec<TokenStream>) -> Result<Self, TestError> {
        if attrs.iter().any(|attr| attr_path(attr).is_none()) {
            return Err(TestError::ExpectedPath);
        }
        Ok(Self { attrs })
    }

    fn len(&self) -> usize {
        self.attrs.len()
    }

    fn contains(&self, name: impl AsRef<str>) -> bool {
        self.get(name).is_some()
    }

    fn contains_all<N: AsRef<str>>(&self, names: &[N]) -> bool {
        names.iter().all(|name| self.contains(name.as_ref()))
    }

    fn contains_any<N: AsRef<str>>(&self, names: &[N]) -> bool {
        names.iter().any(|name| self.contains(name.as_ref()))
    }

    fn count(&self, name: impl AsRef<str>) -> usize {
        let name = name.as_ref();
        self.attrs
            .iter()
            .filter(|attr| is_named(attr, name))
            .count()
    }

    fn get(&self, name: impl AsRef<str>) -> Option<&TokenStream> {
        let name = name.as_ref();
        self.attrs.iter().find(|attr| is_named(attr, name))
    }

    fn get_many<N: AsRef<str>>(&self, names: &[N]) -> Vec<Option<&TokenStream>> {
        names.iter().map(|name| self.get(name.as_ref())).collect()
    }

    fn push(&mut self, attr: TokenStream) -> Result<(), TestError> {
        validate(&attr)?;
        self.attrs.push(attr);
        Ok(())
    }

    fn extend(&mut self, attrs: Vec<TokenStream>) -> Result<(), TestError> {
        for attr in &attrs {
            validate(attr)?;
        }
        self.attrs.extend(attrs);
        Ok(())
    }

    fn replace(
        &mut self,
        name: impl AsRef<str>,
        attr: TokenStream,
    ) -> Result<Option<TokenStream>, TestError> {
        validate(&attr)?;
        let name = name.as_ref();
        let Some(index) = self.attrs.iter().position(|attr| is_named(attr, name)) else {
            return Ok(None);
        };
        Ok(Some(core::mem::replace(&mut self.attrs[index], attr)))
    }

    fn remove(&mut self, name: impl AsRef<str>) -> Option<TokenStream> {
        let name = name.as_ref();
        let index = self.attrs.iter().position(|attr| is_named(attr, name))?;
        Some(self.attrs.remove(index))
    }

    fn remove_many<N: AsRef<str>>(&mut self, names: &[N]) -> Vec<Option<TokenStream>> {
        names
            .iter()
            .map(|name| self.remove(name.as_ref()))
            .collect()
    }

    fn remove_all(&mut self, name: impl AsRef<str>) -> Vec<TokenStream> {
        let name = name.as_ref();
        let mut retained = Vec::with_capacity(self.attrs.len());
        let mut removed = Vec::new();
        for attr in core::mem::take(&mut self.attrs) {
            if is_named(&attr, name) {
                removed.push(attr);
            } else {
                retained.push(attr);
            }
        }
        self.attrs = retained;
        removed
    }

    fn take_optional_list(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<Option<TokenStream>, TestError> {
        let name = name.as_ref();
        let matching = self
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(index, attr)| is_named(attr, name).then_some(index))
            .collect::<Vec<_>>();
        if matching.len() > 1 {
            return Err(TestError::Duplicate(name.to_owned()));
        }
        let Some(index) = matching.first().copied() else {
            return Ok(None);
        };
        let arguments = match_list(&self.attrs[index], name)
            .ok_or_else(|| TestError::ExpectedList(name.to_owned()))?;
        self.attrs.remove(index);
        Ok(Some(arguments))
    }

    fn take_required_list(&mut self, name: impl AsRef<str>) -> Result<TokenStream, TestError> {
        let name = name.as_ref();
        self.take_optional_list(name)?
            .ok_or_else(|| TestError::MissingRequired(name.to_owned()))
    }

    fn take_optional_lists<N: AsRef<str>>(
        &mut self,
        names: &[N],
    ) -> Result<Vec<Option<TokenStream>>, TestError> {
        let mut candidate = self.clone();
        let values = names
            .iter()
            .map(|name| candidate.take_optional_list(name.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        *self = candidate;
        Ok(values)
    }

    fn take_required_lists<N: AsRef<str>>(
        &mut self,
        names: &[N],
    ) -> Result<Vec<TokenStream>, TestError> {
        let mut candidate = self.clone();
        let values = names
            .iter()
            .map(|name| candidate.take_required_list(name.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        *self = candidate;
        Ok(values)
    }

    fn emit(&self) -> TokenStream {
        self.attrs.iter().flat_map(emit_attr).collect()
    }

    fn emit_cfg(&self) -> TokenStream {
        self.attrs
            .iter()
            .filter(|attr| is_named(attr, "cfg") || is_named(attr, "cfg_attr"))
            .flat_map(emit_attr)
            .collect()
    }
}

fn parse_bodies(
    cursor: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Vec<TokenStream>, TestError> {
    let mut attrs = Vec::new();
    while matches!(cursor.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '#') {
        cursor.next();
        if matches!(cursor.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '!') {
            cursor.next();
            return Err(TestError::InnerAttribute);
        }
        match cursor.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                attrs.push(group.stream());
            }
            _ => return Err(TestError::ExpectedGroup),
        }
    }
    Ok(attrs)
}

fn attr_path(body: &TokenStream) -> Option<String> {
    path_and_end(&body.clone().into_iter().collect::<Vec<_>>()).map(|(path, _)| path)
}

fn is_named(body: &TokenStream, name: impl AsRef<str>) -> bool {
    attr_path(body).is_some_and(|path| path == name.as_ref())
}

fn match_list(body: &TokenStream, name: impl AsRef<str>) -> Option<TokenStream> {
    let tokens = body.clone().into_iter().collect::<Vec<_>>();
    let (path, input_index) = path_and_end(&tokens)?;
    if path != name.as_ref() || tokens.len() != input_index + 1 {
        return None;
    }
    match &tokens[input_index] {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
            Some(group.stream())
        }
        _ => None,
    }
}

fn emit_attr(body: &TokenStream) -> impl Iterator<Item = TokenTree> {
    let hash = TokenTree::Punct(Punct::new('#', Spacing::Alone));
    let group = TokenTree::Group(Group::new(Delimiter::Bracket, body.clone()));
    [hash, group].into_iter()
}

fn path_and_end(tokens: &[TokenTree]) -> Option<(String, usize)> {
    let TokenTree::Ident(first) = tokens.first()? else {
        return None;
    };
    let mut path = first.to_string();
    let mut index = 1;
    while matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == ':') {
        let Some(TokenTree::Punct(second_colon)) = tokens.get(index + 1) else {
            return None;
        };
        let Some(TokenTree::Ident(segment)) = tokens.get(index + 2) else {
            return None;
        };
        if second_colon.as_char() != ':' {
            return None;
        }
        path.push_str("::");
        path.push_str(&segment.to_string());
        index += 3;
    }
    Some((path, index))
}

fn validate(attr: &TokenStream) -> Result<(), TestError> {
    attr_path(attr).map(|_| ()).ok_or(TestError::ExpectedPath)
}

fn ts(input: &str) -> TokenStream {
    input.parse().expect("valid token stream")
}

fn compact(input: impl ToString) -> String {
    input
        .to_string()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn raw_parser_collects_bodies_and_leaves_the_item() {
    let mut cursor = ts("#[doc = \"hello\"] #[rustfmt::skip] #[cfg(test)] fn demo() {}")
        .into_iter()
        .peekable();
    let attrs = parse_bodies(&mut cursor).expect("outer attributes are valid");

    assert_eq!(attrs.len(), 3);
    assert_eq!(attr_path(&attrs[0]).as_deref(), Some("doc"));
    assert_eq!(attr_path(&attrs[1]).as_deref(), Some("rustfmt::skip"));
    assert_eq!(attr_path(&attrs[2]).as_deref(), Some("cfg"));
    assert!(matches!(cursor.next(), Some(TokenTree::Ident(ident)) if ident == "fn"));
}

#[test]
fn raw_parser_rejects_real_inner_attributes_and_missing_groups() {
    let mut inner = ts("#![allow(dead_code)] fn demo() {}")
        .into_iter()
        .peekable();
    assert_eq!(
        parse_bodies(&mut inner).expect_err("inner attributes must fail"),
        TestError::InnerAttribute
    );

    let mut missing_group = ts("# item").into_iter().peekable();
    assert_eq!(
        parse_bodies(&mut missing_group).expect_err("a bracketed group is required"),
        TestError::ExpectedGroup
    );
}

#[test]
fn exact_path_and_list_matching_do_not_accept_prefixes_or_trailing_tokens() {
    let list = ts("tool::route(/users)");
    assert!(is_named(&list, "tool::route"));
    assert!(!is_named(&list, "tool"));
    assert_eq!(compact(match_list(&list, "tool::route").unwrap()), "/users");
    assert!(match_list(&list, "route").is_none());
    assert!(match_list(&ts("tool::route = value"), "tool::route").is_none());
    assert!(match_list(&ts("tool::route(value) extra"), "tool::route").is_none());
}

#[test]
fn construction_validates_paths_but_keeps_ordinary_duplicates() {
    let attrs = TestOuterAttr::try_from(vec![ts("doc = \"one\""), ts("doc = \"two\"")])
        .expect("duplicate outer attributes are legal");
    assert_eq!(attrs.count("doc"), 2);
    assert_eq!(
        TestOuterAttr::try_from(vec![ts("= invalid")]).err(),
        Some(TestError::ExpectedPath)
    );
}

#[test]
fn lookup_and_array_queries_accept_string_like_names() {
    let attrs = TestOuterAttr::try_from(vec![ts("doc = \"hello\""), ts("cfg(test)")])
        .expect("valid attributes");
    let names = [String::from("doc"), String::from("cfg")];

    assert!(attrs.contains(String::from("doc")));
    assert!(attrs.contains_all(&names));
    assert!(attrs.contains_any(&[String::from("missing"), String::from("cfg")]));
    assert_eq!(
        attrs.get_many(&["cfg", "missing"])[0].map(compact),
        Some("cfg(test)".into())
    );
    assert!(attrs.get_many(&["cfg", "missing"])[1].is_none());
}

#[test]
fn crud_operations_preserve_order_and_validate_batches_atomically() {
    let mut attrs = TestOuterAttr::try_from(vec![ts("one"), ts("two"), ts("one(extra)")])
        .expect("valid attributes");
    attrs.push(ts("three")).expect("valid path");
    attrs
        .extend(vec![ts("four"), ts("five")])
        .expect("valid batch");
    let before_invalid_batch = compact(attrs.emit());
    assert_eq!(
        attrs.extend(vec![ts("six"), ts("= invalid")]),
        Err(TestError::ExpectedPath)
    );
    assert_eq!(compact(attrs.emit()), before_invalid_batch);

    let replaced = attrs
        .replace("two", ts("replacement"))
        .expect("replacement is valid")
        .expect("two exists");
    assert_eq!(compact(replaced), "two");
    assert_eq!(
        attrs
            .remove_many(&["three", "missing"])
            .into_iter()
            .map(|value| value.map(compact))
            .collect::<Vec<_>>(),
        vec![Some("three".into()), None]
    );
    assert_eq!(
        attrs
            .remove_all("one")
            .into_iter()
            .map(compact)
            .collect::<Vec<_>>(),
        vec!["one", "one(extra)"]
    );
    assert_eq!(compact(attrs.emit()), "#[replacement]#[four]#[five]");
}

#[test]
fn unique_list_extraction_reports_errors_without_mutating() {
    let mut duplicate = TestOuterAttr::try_from(vec![ts("url(/one)"), ts("url(/two)")])
        .expect("duplicates are valid until unique extraction");
    let before = compact(duplicate.emit());
    assert_eq!(
        duplicate
            .take_optional_list("url")
            .expect_err("duplicate unique attributes must fail"),
        TestError::Duplicate("url".into())
    );
    assert_eq!(compact(duplicate.emit()), before);

    let mut not_a_list = TestOuterAttr::try_from(vec![ts("url = /users")]).unwrap();
    let before = compact(not_a_list.emit());
    assert_eq!(
        not_a_list
            .take_required_list("url")
            .expect_err("non-list input must fail"),
        TestError::ExpectedList("url".into())
    );
    assert_eq!(compact(not_a_list.emit()), before);

    let mut missing = TestOuterAttr::default();
    assert_eq!(
        missing
            .take_required_list("url")
            .expect_err("the required attribute is absent"),
        TestError::MissingRequired("url".into())
    );
}

#[test]
fn list_array_extraction_is_atomic_and_follows_request_order() {
    let mut attrs = TestOuterAttr::try_from(vec![
        ts("url(/users)"),
        ts("middleware([auth])"),
        ts("config([one])"),
        ts("config([two])"),
    ])
    .unwrap();
    let before = compact(attrs.emit());
    assert_eq!(
        attrs
            .take_optional_lists(&["url", "config"])
            .expect_err("the second requested attribute is duplicated"),
        TestError::Duplicate("config".into())
    );
    assert_eq!(compact(attrs.emit()), before);

    attrs.remove("config");
    let values = attrs
        .take_optional_lists(&["middleware", "missing", "url"])
        .expect("remaining requested attributes are unique");
    assert_eq!(
        values
            .into_iter()
            .map(|value| value.map(compact))
            .collect::<Vec<_>>(),
        vec![Some("[auth]".into()), None, Some("/users".into())]
    );
    assert_eq!(compact(attrs.emit()), "#[config([two])]");
}

#[test]
fn required_list_array_extraction_is_atomic_when_a_name_is_missing() {
    let mut attrs =
        TestOuterAttr::try_from(vec![ts("url(/users)"), ts("middleware([auth])")]).unwrap();
    let before = compact(attrs.emit());

    assert_eq!(
        attrs
            .take_required_lists(&["url", "config"])
            .expect_err("config is required but absent"),
        TestError::MissingRequired("config".into())
    );
    assert_eq!(compact(attrs.emit()), before);

    let values = attrs
        .take_required_lists(&["middleware", "url"])
        .expect("both required attributes exist");
    assert_eq!(
        values.into_iter().map(compact).collect::<Vec<_>>(),
        ["[auth]", "/users"]
    );
    assert_eq!(attrs.len(), 0);
}

#[test]
fn emission_reconstructs_attributes_and_cfg_filtering_keeps_cfg_attr() {
    let attrs = TestOuterAttr::try_from(vec![
        ts("doc = \"hello\""),
        ts("cfg(unix)"),
        ts("cfg_attr(test, allow(dead_code))"),
    ])
    .unwrap();

    assert_eq!(
        compact(attrs.emit()),
        compact("#[doc = \"hello\"] #[cfg(unix)] #[cfg_attr(test, allow(dead_code))]")
    );
    assert_eq!(
        compact(attrs.emit_cfg()),
        compact("#[cfg(unix)] #[cfg_attr(test, allow(dead_code))]")
    );
}

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
    assert!(matches!(attrs.take_optional_list("missing"), Ok(None)));
    assert!(
        attrs
            .take_optional_lists::<&str>(&[])
            .expect("an empty request succeeds")
            .is_empty()
    );
    assert!(
        attrs
            .take_required_lists::<&str>(&[])
            .expect("an empty request succeeds")
            .is_empty()
    );
    attrs.clear();
    assert!(attrs.emit().is_empty());
    assert!(attrs.emit_cfg().is_empty());

    let streams: Vec<ProcTokenStream> = attrs.into();
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
    body: ProcTokenStream,
    error: OuterAttrError,
) -> Result<ProcTokenStream, OuterAttrError>
where
    I: Iterator<Item = ProcTokenTree>,
{
    let _ = parse_outer_attrs(cursor);
    attrs.push(body.clone())?;
    attrs.extend(vec![body.clone()])?;
    let _ = attrs.replace(String::from("cfg"), body)?;
    let _ = attrs.take_required_list(String::from("url"))?;
    let _ = error.name();
    let _ = error.span();
    Ok(error.into_compile_error())
}
