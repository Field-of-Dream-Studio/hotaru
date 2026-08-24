use super::*;

/// `+` in a form body must decode back to a literal space — this is what
/// every browser sends for `<form method="POST">`.
#[test]
fn parse_plus_decodes_to_space() {
    let form = UrlEncodedForm::parse(b"greeting=Hello+world").unwrap();
    assert_eq!(form.get("greeting").map(String::as_str), Some("Hello world"));
}

/// `%20` must still decode to a space (the other accepted form-encoding
/// for spaces).
#[test]
fn parse_percent_twenty_decodes_to_space() {
    let form = UrlEncodedForm::parse(b"greeting=Hello%20world").unwrap();
    assert_eq!(form.get("greeting").map(String::as_str), Some("Hello world"));
}

/// A literal `+` typed by the user is sent on the wire as `%2B` and must
/// decode back to `+`, not to a space.
#[test]
fn parse_percent_2b_decodes_to_literal_plus() {
    let form = UrlEncodedForm::parse(b"math=1%2B1").unwrap();
    assert_eq!(form.get("math").map(String::as_str), Some("1+1"));
}

/// Keys are form-encoded too — `+` in the key should become a space.
#[test]
fn parse_decodes_plus_in_key() {
    let form = UrlEncodedForm::parse(b"first+name=Ada").unwrap();
    assert_eq!(form.get("first name").map(String::as_str), Some("Ada"));
}

#[test]
fn parse_multiple_pairs() {
    let form = UrlEncodedForm::parse(b"a=Hello+world&b=foo%20bar&c=1%2B2").unwrap();
    assert_eq!(form.get("a").map(String::as_str), Some("Hello world"));
    assert_eq!(form.get("b").map(String::as_str), Some("foo bar"));
    assert_eq!(form.get("c").map(String::as_str), Some("1+2"));
}

#[test]
fn parse_non_ascii_percent_encoded() {
    // UTF-8 "héllo" — é is 0xC3 0xA9
    let form = UrlEncodedForm::parse(b"name=h%C3%A9llo").unwrap();
    assert_eq!(form.get("name").map(String::as_str), Some("héllo"));
}

#[test]
fn parse_empty_body_returns_empty_form() {
    let form = UrlEncodedForm::parse(b"").unwrap();
    assert!(form.get_all().is_empty());
}

#[test]
fn parse_rejects_invalid_utf8() {
    let err = UrlEncodedForm::parse(vec![0xff, 0xfe]).unwrap_err();
    assert_eq!(err, UrlEncodedError::InvalidUtf8);
}

#[test]
fn parse_rejects_missing_equals() {
    let err = UrlEncodedForm::parse(b"keywithoutvalue").unwrap_err();
    assert!(matches!(err, UrlEncodedError::MalformedPair(_)));
}

#[test]
fn parse_rejects_empty_pair_in_payload() {
    let err = UrlEncodedForm::parse(b"a=1&&b=2").unwrap_err();
    assert!(matches!(err, UrlEncodedError::MalformedPair(_)));
}

#[test]
fn round_trip_serialization() {
    let mut form = UrlEncodedForm::new();
    form.insert("name".to_string(), "Ada Lovelace".to_string());
    form.insert("score".to_string(), "100%".to_string());

    let serialized = form.to_string();
    let parsed = UrlEncodedForm::parse(serialized.as_bytes()).unwrap();

    assert_eq!(parsed.get("name").map(String::as_str), Some("Ada Lovelace"));
    assert_eq!(parsed.get("score").map(String::as_str), Some("100%"));
}
