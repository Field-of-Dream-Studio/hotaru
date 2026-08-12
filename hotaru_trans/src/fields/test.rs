use core::fmt::{self, Display, Formatter};
use proc_macro::{Ident, TokenStream};

use super::{
    AttrFields, AttrFieldsError, AttrLiteralFields, AttrTokenFields,
    convert::first_duplicate,
    crud::{first_candidate_duplicate, position, remove_value, replace_value, upsert_value},
    extract::first_unknown,
};

#[derive(Debug, Eq, PartialEq)]
struct TestKey {
    name: &'static str,
    origin: &'static str,
}

impl TestKey {
    fn new(name: &'static str, origin: &'static str) -> Self {
        Self { name, origin }
    }
}

impl Display for TestKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

fn fields(pairs: Vec<(TestKey, i32)>) -> Vec<(TestKey, i32)> {
    pairs
}

fn get<'a>(fields: &'a [(TestKey, i32)], name: &str) -> Option<&'a i32> {
    position(fields, name).map(|index| &fields[index].1)
}

#[test]
fn duplicate_detection_reports_the_second_occurrence_in_source_order() {
    let pairs = vec![
        (TestKey::new("one", "first one"), 1),
        (TestKey::new("two", "first two"), 2),
        (TestKey::new("one", "second one"), 3),
        (TestKey::new("two", "second two"), 4),
    ];

    let duplicate = first_duplicate(&pairs).expect("one is duplicated");
    assert_eq!(duplicate.name, "one");
    assert_eq!(duplicate.origin, "second one");
}

#[test]
fn batch_duplicate_detection_checks_existing_fields_and_the_batch() {
    let mut fields = fields(vec![
        (TestKey::new("one", "stored one"), 1),
        (TestKey::new("two", "stored two"), 2),
    ]);
    let conflicts_with_existing = vec![
        (TestKey::new("three", "new three"), 3),
        (TestKey::new("two", "new two"), 4),
    ];
    let conflicts_with_batch = vec![
        (TestKey::new("three", "first three"), 3),
        (TestKey::new("four", "new four"), 4),
        (TestKey::new("three", "second three"), 5),
    ];

    let existing_duplicate = first_candidate_duplicate(&fields, &conflicts_with_existing)
        .expect("two conflicts with an existing field");
    assert_eq!(existing_duplicate.origin, "new two");
    let batch_duplicate = first_candidate_duplicate(&fields, &conflicts_with_batch)
        .expect("three occurs twice in the batch");
    assert_eq!(batch_duplicate.origin, "second three");
    assert_eq!(
        fields.len(),
        2,
        "validation must not mutate existing fields"
    );

    let valid = vec![
        (TestKey::new("three", "new three"), 3),
        (TestKey::new("four", "new four"), 4),
    ];
    assert!(first_candidate_duplicate(&fields, &valid).is_none());
    fields.extend(valid);
    assert_eq!(fields.len(), 4);
}

#[test]
fn lookup_accepts_string_like_names_and_preserves_order() {
    let mut fields = fields(vec![
        (TestKey::new("one", "source one"), 10),
        (TestKey::new("two", "source two"), 20),
        (TestKey::new("three", "source three"), 30),
    ]);

    assert_eq!(fields.len(), 3);
    assert!(position(&fields, String::from("one")).is_some());
    assert_eq!(get(&fields, "two"), Some(&20));
    assert_eq!(get(&fields, "missing"), None);
    assert_eq!(
        [String::from("three"), String::from("missing")]
            .iter()
            .map(|name| get(&fields, name))
            .collect::<Vec<_>>(),
        vec![Some(&30), None]
    );

    let two = position(&fields, "two").expect("two exists");
    fields[two].1 = 21;
    for (_, value) in &mut fields {
        *value += 1;
    }

    let ordered = fields
        .iter()
        .map(|(key, value)| (key.name, *value))
        .collect::<Vec<_>>();
    assert_eq!(ordered, vec![("one", 11), ("two", 22), ("three", 31)]);
}

#[test]
fn unknown_detection_reports_the_first_unknown_field() {
    let fields = fields(vec![
        (TestKey::new("known", "known source"), 1),
        (TestKey::new("first_unknown", "first unknown source"), 2),
        (TestKey::new("second_unknown", "second unknown source"), 3),
    ]);

    assert_eq!(
        first_unknown(&fields, &["known"])
            .expect("an unknown field exists")
            .name,
        "first_unknown"
    );
    assert_eq!(
        first_unknown(&fields, &["known", "first_unknown"])
            .expect("an unknown field exists")
            .name,
        "second_unknown"
    );
    assert!(first_unknown(&fields, &["known", "first_unknown", "second_unknown"]).is_none());
}

#[test]
fn replace_and_upsert_preserve_existing_keys_and_positions() {
    let mut fields = fields(vec![
        (TestKey::new("one", "original one"), 1),
        (TestKey::new("two", "original two"), 2),
    ]);

    assert_eq!(replace_value(&mut fields, "one", 10), Some(1));
    assert_eq!(replace_value(&mut fields, "missing", 99), None);
    assert_eq!(
        upsert_value(&mut fields, TestKey::new("two", "replacement two"), 20),
        Some(2)
    );
    assert_eq!(
        upsert_value(&mut fields, TestKey::new("three", "new three"), 30),
        None
    );

    let ordered = fields
        .iter()
        .map(|(key, value)| (key.name, key.origin, *value))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        vec![
            ("one", "original one", 10),
            ("two", "original two", 20),
            ("three", "new three", 30),
        ]
    );
}

#[test]
fn remove_operations_follow_request_order() {
    let mut fields = fields(vec![
        (TestKey::new("one", "source one"), 1),
        (TestKey::new("two", "source two"), 2),
        (TestKey::new("three", "source three"), 3),
    ]);

    assert_eq!(remove_value(&mut fields, "two"), Some(2));
    assert_eq!(
        ["three", "missing", "one", "one"]
            .iter()
            .map(|name| remove_value(&mut fields, name))
            .collect::<Vec<_>>(),
        vec![Some(3), None, Some(1), None]
    );
    assert!(fields.is_empty());
}

#[test]
fn clear_removes_all_fields() {
    let mut fields = fields(vec![
        (TestKey::new("one", "source one"), 1),
        (TestKey::new("two", "source two"), 2),
    ]);

    fields.clear();

    assert!(fields.is_empty());
    assert_eq!(fields.len(), 0);
}

#[test]
fn empty_attr_fields_exercises_the_production_api_without_proc_macro_handles() {
    let mut fields = AttrFields::<i32>::try_from(Vec::new()).expect("empty fields are valid");

    assert!(fields.is_empty());
    assert_eq!(fields.len(), 0);
    assert!(!fields.contains("missing"));
    assert!(fields.contains_all::<&str>(&[]));
    assert!(!fields.contains_any::<String>(&[]));
    assert_eq!(fields.get("missing"), None);
    assert_eq!(fields.get_many(&["missing"]), vec![None]);
    assert_eq!(fields.get_mut("missing"), None);
    assert_eq!(fields.replace("missing", 1), None);
    assert_eq!(fields.remove("missing"), None);
    assert_eq!(fields.remove_many(&["missing"]), vec![None]);
    assert_eq!(fields.take_optional("missing"), None);
    assert_eq!(fields.take_optional_many(&["missing"]), vec![None]);
    assert_eq!(
        fields
            .take_required_many::<&str>(&[])
            .expect("requesting no required fields succeeds"),
        Vec::<i32>::new()
    );
    fields
        .insert_many(Vec::new())
        .expect("empty batch is valid");
    fields.clear();
    fields
        .reject_unknown::<&str>(&[])
        .expect("empty fields contain no unknown names");

    let pairs: Vec<(Ident, i32)> = fields.into();
    assert!(pairs.is_empty());

    let fields = AttrFields::<i32>::try_from(Vec::new()).expect("empty fields are valid");
    assert_eq!(fields.into_iter().count(), 0);

    AttrFields::<i32>::try_from(Vec::new())
        .expect("empty fields are valid")
        .reject_rest()
        .expect("empty fields leave no remainder");
}

#[test]
fn attr_fields_error_implements_the_standard_error_contract() {
    fn assert_error<E: core::error::Error>() {}
    assert_error::<AttrFieldsError>();
}

// Compile the complete proc-macro-specific surface. This function is never
// called because constructing `proc_macro::Ident` is only valid while rustc is
// executing a procedural macro; the generic lookup and mutation mechanics are
// executed by the tests above.
#[allow(dead_code)]
fn assert_proc_macro_api_compiles(
    mut fields: AttrFields<i32>,
    key: Ident,
    pairs: Vec<(Ident, i32)>,
    error: AttrFieldsError,
) -> Result<TokenStream, AttrFieldsError> {
    let _ = fields.iter().count();
    let _ = fields.iter_mut().count();
    fields.insert(key, 1)?;
    fields.insert_many(pairs)?;
    let _ = fields.upsert(Ident::new("field", error.span()), 2);
    let _ = fields.take_required(String::from("field"))?;
    let _ = error.name();
    Ok(error.into_compile_error())
}

#[allow(dead_code)]
fn assert_specializations_compile(
    literal_fields: AttrLiteralFields,
    token_fields: AttrTokenFields,
) {
    let _: AttrFields<proc_macro::Literal> = literal_fields;
    let _: AttrFields<TokenStream> = token_fields;
}
