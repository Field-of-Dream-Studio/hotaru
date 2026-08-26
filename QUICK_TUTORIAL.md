# Hotaru Quickstart

This guide follows the default Hotaru path: the `trans` endpoint DSL,
automatic registration, Tokio, and HTTP/1.1. The generated project is the
source of truth for this document.

Hotaru 0.8.x is pre-1.0 and remains experimental. Use Rust 1.88 or newer.

## 1. Create and run a project

Install the CLI and generate a project:

```bash
cargo install hotaru
hotaru new hello_hotaru
cd hello_hotaru
cargo run
```

The server listens on `127.0.0.1:3003`. From another terminal:

```bash
curl -i http://127.0.0.1:3003/
```

The response body is:

```text
Hello, world!
```

`hotaru new` creates the manifest, `src/main.rs`, `build.rs`, `templates/`,
and `programfiles/`. The first build generates `src/resource.rs`.

## 2. Understand the generated server

The generated `src/main.rs` uses the same canonical shape as the repository
README and website:

```rust
use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub hello_world<HTTP> {
        text_response("Hello, world!")
    }
}
```

The important pieces are:

- `LServer!` declares the application server.
- `HTTP::server(...)` installs HTTP/1.1 with a shared `HttpSafety` baseline.
- `run_server!(APP)` creates the Tokio runtime and blocks until shutdown.
- `endpoint!` defines a route; the default `auto-reg` feature registers it.

No `#[tokio::main]` or `async fn main` is required on the default path.

## 3. Add a path parameter

Add this endpoint after the generated endpoint:

```rust
endpoint! {
    APP.url("/users/<int:id>"),
    pub user<HTTP> {
        let id = req.param("id").unwrap_or_else(|| "unknown".to_string());
        text_response(format!("user {id}"))
    }
}
```

Restart the server and request it:

```bash
curl http://127.0.0.1:3003/users/42
```

Expected body:

```text
user 42
```

Hotaru patterns use angle brackets. Common forms include `<int:id>`,
`<uuid:token>`, and `<**path>`.

## 4. Return JSON

`akari_json!` creates a JSON response on the default facade:

```rust
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

Try it with:

```bash
curl -i http://127.0.0.1:3003/status
```

## 5. Read request information

The endpoint context is available as `req`. Its convenience methods cover
common routing information, while `req.request` exposes the HTTP message:

```rust
endpoint! {
    APP.url("/request-info"),
    pub request_info<HTTP> {
        let method = req.method();
        let path = req.path();
        let user_agent = req.header_str("user-agent").unwrap_or("unknown");

        text_response(format!("{method:?} {path} user-agent={user_agent}"))
    }
}
```

Header names are case-insensitive. Prefer the typed or convenience APIs when
one exists; direct mutation of the public header map can bypass cached typed
metadata.

## 6. Configure HTTP safety limits

Pass a configured baseline to `HTTP::server`:

```rust
LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(
            HttpSafety::default().with_max_body_size(1024 * 1024)
        )))
        .build()
);
```

Route-specific method restrictions can be attached to an endpoint:

```rust
endpoint! {
    APP.url("/submit"),
    config = [HttpSafety::new().with_allowed_method(POST)],
    pub submit<HTTP> {
        text_response("accepted")
    }
}
```

The protocol baseline applies to every request; endpoint configuration overlays
route-specific restrictions.

## 7. Initialize an existing Cargo project

Inside an existing Cargo package:

```bash
hotaru init
```

This creates or updates the starter files without overwriting unrelated source.
It does **not** edit an existing `Cargo.toml`; follow the dependency and
`build = "build.rs"` instructions printed by the command.

## 8. Troubleshooting

- **Address already in use:** change `.binding("127.0.0.1:3003")` or stop the
  process currently using the port.
- **The `hotaru` command is missing:** ensure Cargo's binary directory is on
  `PATH`, then rerun `cargo install hotaru`.
- **A route does not match:** use Hotaru pattern syntax such as `<int:id>`, not
  `{id}`.
- **Templates or static files are missing:** keep them in `templates/` and
  `programfiles/`, then rebuild so `build.rs` can copy them.

## Next steps

- [Main README](readme.md) — features, registration modes, and release notes.
- [HTTP guide](hotaru_http/HOTARU_HTTP_DOC.md) — current request, response,
  body, safety, and HTTP/1.1 behavior.
- [`examples/starter_trans`](examples/starter_trans) — the minimal default
  registration example.
- [`examples/starter_trans_no_auto_reg`](examples/starter_trans_no_auto_reg) —
  explicit registration without `auto-reg`.
- [`examples/starter_manual`](examples/starter_manual) — the experimental
  manual endpoint API.
