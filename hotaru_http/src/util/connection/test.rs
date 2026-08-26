use std::io::Cursor;

use hotaru_io_tokio::TokioIo;
use tokio::io::BufReader;

use super::{ConnectionError, ConnectionOptions, ConnectionToken};
use crate::message::header::{HeaderMap, HeaderValue};
use crate::message::http_value::{HttpVersion, StatusCode};
use crate::message::meta::{HttpMeta, MetaError};
use crate::message::request::HttpRequest;
use crate::message::response::HttpResponse;
use crate::security::safety::HttpSafety;
use crate::util::streamed::Streamed;

#[test]
fn known_tokens_are_case_insensitive() {
    assert_eq!("Close".parse(), Ok(ConnectionToken::Close));
    assert_eq!("KEEP-ALIVE".parse(), Ok(ConnectionToken::KeepAlive));
    assert_eq!("Upgrade".parse(), Ok(ConnectionToken::Upgrade));
}

#[test]
fn extension_tokens_are_normalized_and_preserved() {
    assert_eq!(
        "X-Custom".parse(),
        Ok(ConnectionToken::Other("x-custom".into()))
    );
}

#[test]
fn invalid_token_characters_are_rejected() {
    assert!(matches!(
        "not a token".parse::<ConnectionToken>(),
        Err(ConnectionError::InvalidToken(_))
    ));
}

#[test]
fn parses_comma_lists_across_multiple_field_values() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "connection".to_string(),
        HeaderValue::Multiple(vec![
            "keep-alive, upgrade".to_string(),
            "X-Hop, close".to_string(),
        ]),
    );

    let options = ConnectionOptions::from_headers(&headers).unwrap();
    assert!(options.contains_keep_alive());
    assert!(options.contains_close());
    assert_eq!(
        options.to_header().as_deref(),
        Some("keep-alive, upgrade, x-hop, close")
    );
}

#[test]
fn ignores_empty_list_elements() {
    let mut headers = HeaderMap::new();
    headers.insert("connection".to_string(), HeaderValue::new(", close,,"));

    let options = ConnectionOptions::from_headers(&headers).unwrap();
    assert_eq!(options.tokens(), &[ConnectionToken::Close]);
}

#[test]
fn persistence_uses_version_defaults() {
    let options = ConnectionOptions::default();
    assert!(options.is_keep_alive(&HttpVersion::Http11));
    assert!(!options.is_keep_alive(&HttpVersion::Http10));
    assert!(!options.is_keep_alive(&HttpVersion::Http20));
}

#[test]
fn http_10_requires_keep_alive_and_close_wins() {
    let mut headers = HeaderMap::new();
    headers.insert("connection".to_string(), HeaderValue::new("keep-alive"));
    let options = ConnectionOptions::from_headers(&headers).unwrap();
    assert!(options.is_keep_alive(&HttpVersion::Http10));

    headers.insert(
        "connection".to_string(),
        HeaderValue::new("keep-alive, close"),
    );
    let options = ConnectionOptions::from_headers(&headers).unwrap();
    assert!(!options.is_keep_alive(&HttpVersion::Http10));
    assert!(!options.is_keep_alive(&HttpVersion::Http11));
}

#[test]
fn parses_and_caches_connection_options() {
    let mut meta = HttpMeta::default();
    meta.header.insert(
        "connection".to_string(),
        HeaderValue::new("keep-alive, upgrade"),
    );

    let options = meta.parse_connection().unwrap();
    assert_eq!(
        options.tokens(),
        &[ConnectionToken::KeepAlive, ConnectionToken::Upgrade]
    );
    assert_eq!(meta.get_connection().unwrap(), options);
}

#[test]
fn delete_removes_cache_and_header() {
    let mut meta = HttpMeta::default();
    meta.set_attribute("connection", "close");
    meta.parse_connection().unwrap();

    meta.delete_connection();

    assert!(meta.get_header("connection").is_none());
    assert!(meta.get_connection().unwrap().is_empty());
}

#[test]
fn connection_errors_integrate_with_meta_error() {
    let error = MetaError::from(ConnectionError::InvalidToken("bad token".to_string()));
    assert_eq!(
        error.to_string(),
        "invalid connection-option token: bad token"
    );
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(StatusCode::from(&error), StatusCode::BAD_REQUEST);
    assert!(!error.can_continue());
}

#[tokio::test]
async fn invalid_connection_option_is_rejected_from_wire() {
    let safety = HttpSafety::default();
    let request = b"GET / HTTP/1.1\r\nConnection: keep alive\r\n\r\n";
    let cursor = Cursor::new(request.to_vec());
    let mut reader = TokioIo::new(BufReader::new(cursor));
    let result = HttpMeta::from_request_stream(&mut reader, &safety, false).await;

    assert!(matches!(
        result,
        Err(Streamed::Err(MetaError::Connection(
            ConnectionError::InvalidToken(ref token)
        ))) if token == "keep alive"
    ));
}

#[test]
fn request_persistence_uses_connection_options() {
    let mut request = HttpRequest::default();
    assert!(request.meta.is_keep_alive());
    assert!(request.is_keep_alive());

    request.meta.set_attribute("connection", "Upgrade, CLOSE");
    assert!(!request.is_keep_alive());

    request
        .meta
        .start_line
        .set_http_version(HttpVersion::Http10);
    request.meta.delete_connection();
    assert!(!request.is_keep_alive());

    request.meta.set_attribute("connection", "Keep-Alive");
    assert!(request.is_keep_alive());

    request
        .meta
        .set_attribute("connection", "keep-alive, close");
    assert!(!request.is_keep_alive());
}

#[test]
fn response_persistence_uses_connection_options() {
    let mut response = HttpResponse::default();
    assert!(response.is_keep_alive());

    response
        .meta
        .start_line
        .set_http_version(HttpVersion::Http10);
    assert!(!response.is_keep_alive());

    response.meta.set_attribute("connection", "keep-alive");
    assert!(response.is_keep_alive());

    response.meta.set_attribute("connection", "upgrade, close");
    assert!(!response.is_keep_alive());
}
