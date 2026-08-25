# hotaru_http

HTTP/1.1 implementation for the [Hotaru](../hotaru) web framework — context type, channel, request/response model, error mapping, and the `Http1Protocol` impl that bridges Hotaru's `Protocol` trait to HTTP/1.1 over any `ConnStream` transport.

Most users should depend on the umbrella `hotaru` crate; this crate is the seam where HTTP-specific code lives so future protocols (HTTP/2, etc.) can sit beside it.

## Features

- `tls` — pulls in [`hotaru_tls`](../hotaru_tls) and exposes `HTTPS = Http1Protocol<TlsStream, TlsTransport>` plus the TLS transport/config re-exports.

## Layout

- `protocol/` — `Http1Protocol`, `HttpError`, and protocol error responses.
- `channel/` — `HttpChannel` trait + `Http1Channel<W>` (per-exchange wire wrapper).
- `context/` — `HttpContext<TS>` (the `RequestContext` impl).
- `message/` — `HttpRequest`, `HttpResponse`, `HttpBody`, `HttpMeta`, `HttpStartLine`, types.
- `security/` — `HttpSafety` (size/limit knobs).
- `util/` — typed `Connection` options, cookies, encoding, and form parsing.

## Versioning

Hotaru workspace crates are released in lockstep. The package version and
exact sibling-crate dependency pins are declared in [`Cargo.toml`](Cargo.toml).
