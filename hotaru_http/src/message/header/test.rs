use super::{HeaderError, HeaderMap, HeaderValue};
use crate::message::http_value::StatusCode;
use crate::message::meta::{HttpMeta, MetaError, StreamedMetaError};
use crate::message::request::HttpRequest;
use crate::protocol::HttpError;
use crate::security::safety::HttpSafety;
use crate::util::streamed::Streamed;
use hotaru_io_tokio::TokioIo;
use std::io::Cursor;
use tokio::io::BufReader;

async fn parse_request_head(input: &[u8]) -> Result<HttpMeta, StreamedMetaError> {
    let cursor = Cursor::new(input.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));

    HttpMeta::from_request_stream(&mut reader, &HttpSafety::default(), false).await
}

fn assert_invalid_header<T: std::fmt::Debug>(result: Result<T, StreamedMetaError>) {
    match result {
        Err(Streamed::Err(error @ MetaError::Header(HeaderError::ParseError(_)))) => {
            assert_eq!(StatusCode::from(&error), StatusCode::BAD_REQUEST);
            assert!(!error.can_continue());
        }
        other => panic!("expected invalid header error, got {other:?}"),
    }
}

#[test]
fn get_only_returns_none_when_absent() {
    let map = HeaderMap::new();
    assert!(matches!(map.get_only("content-length"), Ok(None)));
}

#[test]
fn get_only_returns_value_when_singular() {
    let mut map = HeaderMap::new();
    map.insert(
        "content-length".to_string(),
        HeaderValue::Single("42".to_string()),
    );
    assert!(matches!(map.get_only("content-length"), Ok(Some("42"))));
}

#[test]
fn get_only_errors_for_multiple_variant_regardless_of_count() {
    let mut map = HeaderMap::new();
    map.insert(
        "content-length".to_string(),
        HeaderValue::Multiple(vec!["10".to_string(), "20".to_string()]),
    );
    assert_eq!(
        map.get_only("content-length"),
        Err(HeaderError::MultipleValues("content-length".to_string()))
    );

    map.insert(
        "content-length".to_string(),
        HeaderValue::Multiple(vec!["42".to_string()]),
    );
    assert_eq!(
        map.get_only("content-length"),
        Err(HeaderError::MultipleValues("content-length".to_string()))
    );

    map.insert("content-length".to_string(), HeaderValue::Multiple(vec![]));
    assert_eq!(
        map.get_only("content-length"),
        Err(HeaderError::MultipleValues("content-length".to_string()))
    );
}

#[test]
fn get_only_parsed_invokes_closure_and_propagates_its_error() {
    let mut map = HeaderMap::new();
    map.insert(
        "content-length".to_string(),
        HeaderValue::Single("42".to_string()),
    );

    let ok: Result<Option<u64>, HeaderError> = map.get_only_parsed("content-length", |s| {
        s.parse()
            .map_err(|_| HeaderError::HeaderValueOverflow("content-length".to_string()))
    });
    assert!(matches!(ok, Ok(Some(42))));

    let err: Result<Option<u64>, HeaderError> = map.get_only_parsed("content-length", |_| {
        Err(HeaderError::InvalidHeaderValue("content-length".to_string()))
    });
    assert_eq!(
        err,
        Err(HeaderError::InvalidHeaderValue("content-length".to_string()))
    );
}

#[test]
fn require_only_parsed_errors_on_absence() {
    let map = HeaderMap::new();
    let result: Result<String, HeaderError> = map.require_only_parsed("host", |s| Ok(s.to_string()));
    assert_eq!(result, Err(HeaderError::Missing("host".to_string())));
}

#[test]
fn require_variants_error_on_absence() {
    let map = HeaderMap::new();
    assert!(matches!(
        map.require("host"),
        Err(HeaderError::Missing(name)) if name == "host"
    ));
    assert_eq!(
        map.require_only("host"),
        Err(HeaderError::Missing("host".to_string()))
    );
    assert_eq!(
        map.require_all("host"),
        Err(HeaderError::Missing("host".to_string()))
    );
}

#[test]
fn get_parsed_sees_full_header_value() {
    let mut map = HeaderMap::new();
    map.insert(
        "set-cookie".to_string(),
        HeaderValue::Multiple(vec!["a=1".to_string(), "b=2".to_string()]),
    );
    let count: Result<Option<usize>, HeaderError> = map.get_parsed("set-cookie", |v| Ok(v.len()));
    assert!(matches!(count, Ok(Some(2))));

    let missing: Result<usize, HeaderError> = map.require_parsed("host", |v| Ok(v.len()));
    assert_eq!(missing, Err(HeaderError::Missing("host".to_string())));
}

#[test]
fn require_remove_returns_value_when_present_and_errors_when_absent() {
    let mut map = HeaderMap::new();
    map.insert(
        "host".to_string(),
        HeaderValue::Single("example.com".to_string()),
    );

    let removed = map.require_remove("host").unwrap();
    assert!(matches!(removed, HeaderValue::Single(ref s) if s == "example.com"));
    assert!(!map.contains_key("host"));

    assert!(matches!(
        map.require_remove("host"),
        Err(HeaderError::Missing(name)) if name == "host"
    ));
}

#[test]
fn accessors_accept_dynamic_name() {
    let mut map = HeaderMap::new();
    map.insert(
        "content-length".to_string(),
        HeaderValue::Single("42".to_string()),
    );

    let dynamic: String = String::from("content-length");
    assert!(map.get(&dynamic).is_some());
    assert!(matches!(map.get_only(&dynamic), Ok(Some("42"))));
    assert!(map.require(&dynamic).is_ok());
}

#[tokio::test]
async fn test_header_null_byte_injection() {
    let mut meta = HttpMeta::new(Default::default(), HeaderMap::new());
    let safety = HttpSafety::default();

    let headers = b"Host: example.com\0malicious.com\r\n\r\n";
    let cursor = Cursor::new(headers.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = meta
        .append_headers_from_stream(&mut reader, &safety, true)
        .await;
    assert_invalid_header(result);
}

#[tokio::test]
async fn test_header_oversized_header_name() {
    let mut meta = HttpMeta::new(Default::default(), HeaderMap::new());
    let safety = HttpSafety::default().with_max_header_size(1024);

    let long_name = "X-".to_string() + &"A".repeat(2048);
    let headers = format!("{}: value\r\n\r\n", long_name);
    let cursor = Cursor::new(headers.as_bytes().to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = meta
        .append_headers_from_stream(&mut reader, &safety, true)
        .await;
    assert!(result.is_err(), "Should reject oversized header name");
}

#[tokio::test]
async fn test_header_oversized_header_value() {
    let mut meta = HttpMeta::new(Default::default(), HeaderMap::new());
    let safety = HttpSafety::default().with_max_header_size(1024);

    let long_value = "A".repeat(10240);
    let headers = format!("X-Large: {}\r\n\r\n", long_value);
    let cursor = Cursor::new(headers.as_bytes().to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = meta
        .append_headers_from_stream(&mut reader, &safety, true)
        .await;
    assert!(result.is_err(), "Should reject oversized header value");
}

#[tokio::test]
async fn test_header_many_headers_exceeding_limit() {
    let mut meta = HttpMeta::new(Default::default(), HeaderMap::new());
    let safety = HttpSafety::default().with_max_header_size(2048);

    let mut headers = String::new();
    for i in 0..100 {
        headers.push_str(&format!("X-Header-{}: value-{}\r\n", i, i));
    }
    headers.push_str("\r\n");
    let cursor = Cursor::new(headers.as_bytes().to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = meta
        .append_headers_from_stream(&mut reader, &safety, true)
        .await;
    assert!(
        result.is_err(),
        "Should reject too many headers exceeding size limit"
    );
}

#[tokio::test]
async fn test_header_duplicate_content_length() {
    let safety = HttpSafety::default();
    let request = b"POST / HTTP/1.1\r\nContent-Length: 10\r\nContent-Length: 20\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpRequest::parse_lazy(&mut reader, &safety, false).await;

    assert!(matches!(
        result,
        Err(HttpError::Meta(MetaError::Header(HeaderError::MultipleValues(ref name))))
            if name == "content-length"
    ));
}

#[tokio::test]
async fn test_header_duplicate_identical_content_length() {
    let safety = HttpSafety::default();
    let request = b"POST / HTTP/1.1\r\nContent-Length: 10\r\nContent-Length: 10\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpMeta::from_request_stream(&mut reader, &safety, false).await;

    assert!(matches!(
        result,
        Err(Streamed::Err(MetaError::Header(HeaderError::MultipleValues(ref name))))
            if name == "content-length"
    ));
}

#[tokio::test]
async fn test_header_comma_separated_content_length() {
    let safety = HttpSafety::default();
    let request = b"POST / HTTP/1.1\r\nContent-Length: 10, 10\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpMeta::from_request_stream(&mut reader, &safety, false).await;

    assert!(matches!(
        result,
        Err(Streamed::Err(MetaError::Header(HeaderError::InvalidHeaderValue(ref name))))
            if name == "content-length"
    ));
}

#[tokio::test]
async fn test_header_content_length_with_transfer_encoding() {
    let safety = HttpSafety::default();
    let request = b"POST / HTTP/1.1\r\nContent-Length: 10\r\nTransfer-Encoding: chunked\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpMeta::from_request_stream(&mut reader, &safety, false).await;

    assert!(matches!(
        result,
        Err(Streamed::Err(MetaError::ConflictingFraming))
    ));
}

#[test]
fn test_header_content_length_u64_boundaries() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("content-length".to_string(), u64::MAX.to_string().into());
    let mut meta = HttpMeta::new(Default::default(), headers);
    assert_eq!(meta.parse_content_length().unwrap(), Some(u64::MAX));

    let mut headers = std::collections::HashMap::new();
    headers.insert("content-length".to_string(), "18446744073709551616".into());
    let mut meta = HttpMeta::new(Default::default(), headers);
    assert!(meta.parse_content_length().is_err());
}

#[tokio::test]
async fn test_header_line_folding() {
    let safety = HttpSafety::default();

    let request = b"GET / HTTP/1.1\r\nX-Long-Header: part1\r\n part2\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpMeta::from_request_stream(&mut reader, &safety, false).await;

    assert_invalid_header(result);
}

#[tokio::test]
async fn test_header_carriage_return_in_value() {
    let result =
        parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX-Test: a\rb\r\n\r\n").await;

    assert_invalid_header(result);
}

#[tokio::test]
async fn test_header_whitespace_before_colon() {
    let result = parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX : v\r\n\r\n")
        .await;

    assert_invalid_header(result);
}

#[tokio::test]
async fn test_header_obfuscated_transfer_encoding_before_framing() {
    let result = parse_request_head(
        b"POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding : chunked\r\n\r\n0\r\n\r\n",
    )
    .await;

    assert_invalid_header(result);
}

#[tokio::test]
async fn test_header_valid_field_line_is_parsed_normally() {
    let meta = parse_request_head(b"GET / HTTP/1.1\r\nHost: example.test\r\nX-Test: v\r\n\r\n")
        .await
        .unwrap();

    assert_eq!(meta.header.get("x-test").unwrap().first(), "v");
}

#[test]
fn test_header_field_line_appends_duplicate_field_names_without_combining() {
    let mut headers = HeaderMap::new();

    headers.insert_field_line("Accept: text/html").unwrap();
    headers.insert_field_line("Accept: application/json").unwrap();

    assert_eq!(headers.get_all("accept"), vec!["text/html", "application/json"]);
}
