use super::context::*;
use crate::message::http_value::StatusCode;
use crate::message::request::HttpRequest;
use crate::message::response::response_templates;
use crate::security::safety::HttpSafety;

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
