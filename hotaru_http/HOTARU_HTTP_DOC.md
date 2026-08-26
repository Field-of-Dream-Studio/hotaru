# Hotaru HTTP/1.1 Guide

This guide documents the current HTTP API in the Hotaru 0.8.6 candidate. For
project creation and the canonical server shape, start with the
[Hotaru Quickstart](../QUICK_TUTORIAL.md).

Hotaru 0.8.x is pre-1.0 and experimental. HTTP/1.1 over Tokio is the most
heavily tested path. HTTP/2 and WebSocket do not ship in this workspace.

## 1. Imports and server setup

Most applications should depend on the umbrella crate and use its facade:

```rust
use hotaru::http::*;
use hotaru::prelude::*;
```

Install HTTP/1.1 with a shared safety baseline:

```rust
LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}
```

The default `hotaru` features include `http`, `tokio`, `trans`, and
`auto-reg`. Direct users of `hotaru_http` are working at the protocol seam and
must select runtime, transport, and registration pieces themselves.

## 2. Request routing and metadata

The generated endpoint context is named `req` unless the fn-style syntax gives
it another name.

```rust
endpoint! {
    APP.url("/users/<int:id>"),
    pub user<HTTP> {
        let method = req.method();
        let path = req.path();
        let id = req.param("id").unwrap_or_else(|| "unknown".to_string());
        let verbose = req.query("verbose").unwrap_or_else(|| "false".to_string());

        text_response(format!(
            "method={method:?} path={path} id={id} verbose={verbose}"
        ))
    }
}
```

Hotaru route patterns use angle brackets, for example `<int:id>`,
`<uuid:token>`, and `<**path>`.

### Headers

Use the context helpers for ordinary reads:

```rust
let user_agent = req.header_str("user-agent").unwrap_or("unknown");
let has_accept = req.has_header("accept");
```

`HeaderMap` treats names case-insensitively and preserves repeated field
values. When changing metadata, prefer typed setters or
`HttpMeta::set_attribute`; direct mutation of the public `header` field can
bypass cached typed metadata.

### Connection addresses

The context exposes the peer and local socket addresses when the transport
provides them:

```rust
let peer = req.remote_addr();
let local = req.local_addr();
```

Both return `Option<SocketAddr>`.

## 3. Responses

The facade re-exports common response helpers:

```rust
endpoint! {
    APP.url("/plain"),
    pub plain<HTTP> {
        text_response("Hello from Hotaru")
    }
}

endpoint! {
    APP.url("/status"),
    pub status<HTTP> {
        akari_json!({
            status: "ok",
            framework: "Hotaru"
        })
    }
}
```

Responses can be customized with builder methods:

```rust
endpoint! {
    APP.url("/created"),
    pub created<HTTP> {
        text_response("created")
            .status(StatusCode::CREATED)
            .add_header("x-hotaru", "0.8")
    }
}
```

Useful helpers include `text_response`, `html_response`, `json_response`,
`redirect_response`, `normal_response`, and `return_status`.

## 4. Request bodies

Bodies are buffered under `HttpSafety` limits and parsed only when requested by
the handler.

### JSON

For `Content-Type: application/json`:

```rust
endpoint! {
    APP.url("/json"),
    config = [HttpSafety::new().with_allowed_method(POST)],
    pub json<HTTP> {
        match req.json().await {
            Ok(value) => akari_json!(value.clone()),
            Err(_) => return_status(StatusCode::BAD_REQUEST),
        }
    }
}
```

### URL-encoded forms

For `Content-Type: application/x-www-form-urlencoded`:

```rust
endpoint! {
    APP.url("/form"),
    config = [HttpSafety::new().with_allowed_method(POST)],
    pub form<HTTP> {
        let username = req
            .form()
            .await
            .ok()
            .and_then(|form| form.get("username"))
            .cloned()
            .unwrap_or_default();

        text_response(format!("username={username}"))
    }
}
```

### Multipart forms

`req.files().await` parses `multipart/form-data` into `MultiForm`. Use
`get_text`, `get_files`, or `get_first_file` on the returned form. Parsing can
fail for a missing boundary, malformed headers, invalid UTF-8 text fields, or
an incomplete body; handle the returned `BodyError` rather than assuming the
payload is valid.

## 5. Cookies

Cookie values and attributes are represented by `Cookie`. The cookie name is
passed separately when adding it to a response:

```rust
endpoint! {
    APP.url("/cookie"),
    pub cookie<HTTP> {
        let cookie = Cookie::new("dark")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax);

        text_response("cookie set").add_cookie("theme", cookie)
    }
}
```

Read request cookies through `req.get_cookie("theme")` or inspect the full map
with `req.get_cookies()`.

Session management is not part of `hotaru_http`; the sibling `htmstd` crate
contains session middleware.

## 6. Safety configuration

`HttpSafety` controls parsing and route policy. A protocol-level value is the
baseline for every request:

```rust
let safety = HttpSafety::default()
    .with_max_body_size(1024 * 1024)
    .with_max_header_size(32 * 1024)
    .with_max_line_length(8 * 1024);
```

Pass it to `HTTP::server(safety)`. A route can overlay method and content-type
rules:

```rust
endpoint! {
    APP.url("/submit"),
    config = [HttpSafety::new()
        .with_allowed_method(POST)
        .with_allowed_content_type(HttpContentType::ApplicationJson())],
    pub submit<HTTP> {
        text_response("accepted")
    }
}
```

The parser enforces configured body, header, line, and header-count limits.

## 7. TLS and compression

HTTPS is opt-in:

```toml
[dependencies]
hotaru = { version = "0.8.5", features = ["https"] }
```

HTTP body compression is also opt-in and is not part of the default build:

```toml
[dependencies]
hotaru = { version = "0.8.5", features = ["http_compression"] }
```

The version numbers above track the latest published release. They are changed
to `0.8.6` only when the release candidate is frozen for publication.

## 8. HTTP/1.1 framing and persistence

The 0.8.6 candidate adds regression coverage for these observable behaviors:

- conflicting repeated `Content-Length` values are rejected;
- `Content-Length` combined with `Transfer-Encoding` is rejected;
- oversized lengths and cumulative chunk-size overflow are rejected;
- standard chunk extensions are accepted;
- unsupported non-chunked transfer codings are rejected;
- trailer fields do not replace the request start line;
- repeated and comma-separated `Connection` values are tokenized;
- `close` wins over `keep-alive`;
- HTTP/1.1 is persistent by default, while HTTP/1.0 requires explicit
  `keep-alive`.

These corrections do not imply that every HTTP/1.1 edge case is implemented.
In particular, do not manually request outgoing chunked serialization, and do
not rely on the built-in client for close-delimited responses or special
HEAD/1xx/204/304 body semantics until those paths have dedicated coverage.

## 9. Errors

HTTP operations return typed `HttpError` variants covering metadata, body,
framing, method, protocol, and I/O failures. Treat malformed framing as a
connection-level failure: do not continue parsing another message from a
stream whose boundary is no longer trustworthy.

Application handlers should convert expected input errors into an appropriate
response, for example `BAD_REQUEST`, `UNSUPPORTED_MEDIA_TYPE`, or
`METHOD_NOT_ALLOWED`, while allowing unexpected transport failures to reach
the framework error path.

## 10. Further examples

- [`examples/starter_trans`](../examples/starter_trans) — canonical default
  registration.
- [`examples/starter_trans_no_auto_reg`](../examples/starter_trans_no_auto_reg)
  — explicit registration.
- [`examples/example_hotaru_2`](../examples/example_hotaru_2) — larger HTTP,
  form, cookie, and middleware examples. Treat commented legacy client samples
  as non-authoritative.
- [Hotaru Quickstart](../QUICK_TUTORIAL.md) — start from an empty directory.
