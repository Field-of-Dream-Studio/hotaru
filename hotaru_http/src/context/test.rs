use super::context::*;
use crate::message::body::{BodyError, HttpBody};
use crate::message::http_value::StatusCode;
use crate::util::content_type::HttpContentType;
use crate::message::request::HttpRequest;
use crate::message::response::{HttpResponse, response_templates};
use crate::protocol::safety::HttpSafety;
use crate::protocol::{HttpError, HttpSyncSettings};
use hotaru_io_tokio::TokioIo;
use std::io::Cursor;
use tokio::io::BufReader;

type TestHttpContext = HttpContext<hotaru_io_tokio::TcpTransport>;

fn client_context(host: &str) -> TestHttpContext {
    TestHttpContext::new_client(host.to_string(), HttpSafety::default())
}

#[test]
fn take_request_sets_missing_host_from_context() {
    let mut ctx = client_context("example.com");

    let mut request = ctx.take_request();

    assert_eq!(request.meta.get_host(), Some("example.com".to_string()));
}

#[test]
fn take_request_preserves_existing_request_host() {
    let mut ctx = client_context("context.example");
    let mut request = HttpRequest::default();
    request.meta.set_host(Some("request.example".to_string()));
    ctx.request = request;

    let mut request = ctx.take_request();

    assert_eq!(request.meta.get_host(), Some("request.example".to_string()));
}

#[test]
fn take_request_ignores_empty_context_host() {
    let mut ctx = client_context("");

    let mut request = ctx.take_request();

    assert_eq!(request.meta.get_host(), None);
}

#[test]
fn set_response_stores_response() {
    let mut ctx = client_context("");
    let response = response_templates::normal_response(StatusCode::CREATED, "created");

    ctx.set_response(response);

    assert_eq!(
        ctx.response.meta.start_line.status_code(),
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn send_and_receive_preserve_request_and_response_payloads() {
    let settings = HttpSyncSettings::default();
    for chunked in [false, true] {
        for (body, expected) in [
            (HttpBody::Empty, Vec::new()),
            (HttpBody::Binary(vec![0, 0xff]), vec![0, 0xff]),
        ] {
            let mut request_meta = HttpRequest::default().meta;
            let mut response_meta = HttpResponse::default().meta;
            request_meta.set_host(Some("example.test".to_string()));
            if chunked {
                request_meta.set_attribute("transfer-encoding", "chunked");
                response_meta.set_attribute("transfer-encoding", "chunked");
                for meta in [&mut request_meta, &mut response_meta] {
                    meta.set_attribute("content-length", "999");
                    meta.set_content_length(999);
                }
            }
            let request = HttpRequest::new(request_meta, body.clone());
            let response = HttpResponse::new(response_meta, body);
            let mut request_writer = TokioIo::new(Vec::new());
            let mut response_writer = TokioIo::new(Vec::new());
            request.send(&mut request_writer, &settings).await.unwrap();
            response
                .send(&mut response_writer, &settings)
                .await
                .unwrap();

            for (wire, is_request) in [
                (request_writer.into_inner(), true),
                (response_writer.into_inner(), false),
            ] {
                let split = wire
                    .windows(4)
                    .position(|part| part == b"\r\n\r\n")
                    .unwrap();
                let headers = std::str::from_utf8(&wire[..split]).unwrap();
                if chunked {
                    assert!(
                        headers
                            .lines()
                            .any(|line| line == "transfer-encoding: chunked")
                    );
                    assert!(!headers.contains("content-length:"));
                } else {
                    assert!(headers.contains(&format!("content-length: {}", expected.len())));
                }

                let mut reader = TokioIo::new(BufReader::new(Cursor::new(wire)));
                let body = if is_request {
                    HttpRequest::receive(&mut reader, &HttpSafety::default(), false)
                        .await
                        .unwrap()
                        .body
                } else {
                    HttpResponse::receive(&mut reader, &HttpSafety::default(), false)
                        .await
                        .unwrap()
                        .body
                };
                let HttpBody::Buffer { data, .. } = body else {
                    panic!("expected a received body buffer");
                };
                assert_eq!(data, expected);
            }
        }
    }
}

#[tokio::test]
async fn send_respects_all_sync_flags() {
    let prepared = HttpBody::Text("hello".to_string())
        .encode(Default::default())
        .unwrap();
    for (sync_encoding, sync_length, sync_content_type, sync_date) in [
        (false, false, false, false),
        (false, false, false, true),
        (false, false, true, false),
        (false, false, true, true),
        (false, true, false, false),
        (false, true, false, true),
        (false, true, true, false),
        (false, true, true, true),
        (true, false, false, false),
        (true, false, false, true),
        (true, false, true, false),
        (true, false, true, true),
        (true, true, false, false),
        (true, true, false, true),
        (true, true, true, false),
        (true, true, true, true),
    ] {
        let settings = HttpSyncSettings {
            sync_encoding,
            sync_length,
            sync_content_type,
            sync_date,
        };
        let mut request_meta = HttpRequest::default().meta;
        let mut response_meta = HttpResponse::default().meta;
        for meta in [&mut request_meta, &mut response_meta] {
            meta.set_attribute("content-encoding", "gzip");
            meta.set_attribute("content-length", "999");
            meta.set_attribute("content-type", "text/plain");
        }
        let request = HttpRequest::new(request_meta, prepared.clone());
        let response = HttpResponse::new(response_meta, prepared.clone());
        let mut request_writer = TokioIo::new(Vec::new());
        let mut response_writer = TokioIo::new(Vec::new());
        request.send(&mut request_writer, &settings).await.unwrap();
        response
            .send(&mut response_writer, &settings)
            .await
            .unwrap();

        for wire in [request_writer.into_inner(), response_writer.into_inner()] {
            let split = wire
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            let headers = std::str::from_utf8(&wire[..split]).unwrap();
            assert_eq!(
                headers
                    .lines()
                    .filter(|line| line.starts_with("date: "))
                    .count(),
                usize::from(settings.sync_date)
            );
            let length = if settings.sync_length { 5 } else { 999 };
            assert!(headers.contains(&format!("content-length: {length}")));
            assert_eq!(
                headers.contains("content-encoding: gzip"),
                !settings.sync_encoding
            );
            let content_type = if settings.sync_content_type {
                HttpContentType::TextPlain().to_string()
            } else {
                "text/plain".to_string()
            };
            assert!(
                headers
                    .lines()
                    .any(|line| line == format!("content-type: {content_type}"))
            );
            assert_eq!(&wire[split + 4..], b"hello");
        }
    }
}

#[tokio::test]
async fn send_preparation_failure_writes_no_headers() {
    let request = HttpRequest::new(HttpRequest::default().meta, HttpBody::Unparsed);
    let mut writer = TokioIo::new(Vec::new());
    let result = request
        .send(&mut writer, &HttpSyncSettings::default())
        .await;

    assert!(matches!(
        result,
        Err(HttpError::Body(BodyError::IncompleteBody))
    ));
    assert!(writer.into_inner().is_empty());
}

#[tokio::test]
async fn send_preserves_specific_binary_content_types_by_default() {
    let mut image = HttpResponse::new(
        HttpResponse::default().meta,
        HttpBody::Binary(vec![0, 0xff]),
    );
    image.meta.set_attribute("content-type", "image/png");
    for (response, expected_type, expected_payload) in [
        (
            response_templates::text_response("hello"),
            HttpContentType::TextPlain(),
            b"hello".to_vec(),
        ),
        (
            response_templates::html_response(b"<p>hello</p>".to_vec()),
            HttpContentType::TextHtml(),
            b"<p>hello</p>".to_vec(),
        ),
        (image, HttpContentType::from_str("image/png"), vec![0, 0xff]),
    ] {
        let mut writer = TokioIo::new(Vec::new());
        response
            .send(&mut writer, &HttpSyncSettings::default())
            .await
            .unwrap();
        let wire = writer.into_inner();
        let split = wire
            .windows(4)
            .position(|part| part == b"\r\n\r\n")
            .unwrap();
        let headers = std::str::from_utf8(&wire[..split]).unwrap();
        assert!(
            headers
                .lines()
                .any(|line| line == format!("content-type: {expected_type}"))
        );
        assert_eq!(&wire[split + 4..], expected_payload);
    }
}
