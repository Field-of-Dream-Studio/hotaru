The Hotaru 0.8 era starts from 23/May/2026.

# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.8.6-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

<img width="1634" height="910" alt="7db7cf68-ebaa-476f-a306-4521ff4c1321" src="https://github.com/user-attachments/assets/6a8aae85-f14b-404b-8aa5-986f486eec3c" />

## Overview

<!--The name 'Hotaru' comes from the Japanese Character '蛍（ほたる）' represents the firefly.--> 

> Small, sweet, easy framework with a protocol-neutral, no_std-ready core 

**[Official Website](https://hotaru.rs)** | **[Examples](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples)**

> Repository transfer notice: the Hotaru repository has moved to
> `https://github.com/Field-of-Dream-Studio/hotaru`.

MSRV: 1.88

### Stability in 0.8.x

Hotaru 0.8.x is pre-1.0 and remains experimental. The default Tokio +
HTTP/1.1 path (`trans`, `auto-reg`, `http`, and `tokio`) receives the broadest
test coverage, while alternative runtimes, embedded targets, and non-default
I/O adapters are still under active development.

### Our Repos 

**[Hotaru](https://github.com/Field-of-Dream-Studio/hotaru)**: Hotaru Core and HTTP utils 

**[Hotaru MQTT](https://github.com/fds-pmine/hotaru_mqtt)**: Hotaru MQTT and broker 

**[API Version](https://github.com/Field-of-Dream-Studio/api_version)**: API Version macro 

## Key Features

- **Multi-Protocol Design**: HTTP/1.1 and HTTPS ship in this workspace; the
  `Protocol` trait remains the extension point for separate or custom protocols.
- **Server + Client**: Endpoints and outpoints share the same protocol, routing,
  and middleware model.
- **Runtime- and I/O-Neutral Core**: Tokio is the default, while alternative
  adapters and `no_std`/embedded paths remain experimental.
- **Flexible Routing**: Literal, typed, wildcard, and regex-backed route
  segments share one routing tree.
- **Web Building Blocks**: Akari templates, forms, uploads, cookies, optional
  compression, and `htmstd` middleware.

## Quick Start

The default configuration uses the `trans` endpoint DSL with the `auto-reg`
feature. The generated `index` definition binds to `APP` during startup:

```rust
use hotaru::prelude::*;
use hotaru::http::*;

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
    pub index<HTTP> {
        text_response("Hello, Hotaru!")
    }
}
```

`run_server!(APP)` builds a tokio runtime, blocks the current thread, and shuts down on Ctrl+C. No `async fn main`, no `#[tokio::main]`. See [Core Concepts](#core-concepts) for the sibling macros (`run_server_until!`, `run_server_no_block!`, `run_server_no_block_until!`) when you need a custom stop source or multi-server orchestration.

The same tested path is available as the
[`starter_trans` example](examples/starter_trans).
For a guided first project, see the [Hotaru Quickstart](QUICK_TUTORIAL.md).

## Installation

### Using the CLI Tool (Recommended)

Install the Hotaru CLI tool:

```bash
cargo install hotaru
```

Create a new project:

```bash
hotaru new my_app
cd my_app
cargo run
```

### Manual Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hotaru = "0.8.6"
```

### Optional Features

Default features: `trans`, `auto-reg`, `http`, `tokio`, `full_regex`, `template`, and `cli`. Cargo's additive feature unification means sub-features pull in their prerequisites automatically. You never have to enable a base feature by hand.

**Protocol stack**

- **`http`** *(default)*: HTTP/1.1 server, client, and message types.
- **`tokio`** *(default)*: Tokio runtime and TCP I/O.
- **`https`**: TLS/HTTPS support; enables `http`.
- **`http_compression`**: Optional gzip, deflate, brotli, and zstd body codecs; enables `http`.

**Endpoint macro flavor:** Pick one (see Core Concepts).

- **`trans`** *(default)*: bang macro with hotaru-blocks body
- **`semi-trans`**: stacked attributes above an `fn`
- **`attr`**: single attribute with args

**Registration**

- **`auto-reg`** *(default-on)*: generated endpoint/outpoint constructors bind during startup. Disable it when registration must be explicit; the selected endpoint DSL still generates constructors that can be passed to `App::bind` or `Blueprint::bind`.

**Misc**

- **`debug`**: Development and troubleshooting logs.
- **`external-ctor`**: Use the external [`ctor`](https://crates.io/crates/ctor) implementation; add `ctor` as a direct dependency.

## Binary Commands

Use the CLI to scaffold projects. `hotaru new` writes the manifest, starter,
asset directories, and `build.rs`; the first build then generates
`src/resource.rs` for runtime template/static lookup.

```bash
cargo install hotaru                   # install the CLI (see Installation above)
hotaru new my_app                      # scaffold a new project
hotaru init                            # add starter files to an existing Cargo crate
cd my_app && cargo run                 # serves http://127.0.0.1:3003
```

`hotaru init` does not edit an existing `Cargo.toml`; it prints the package and
dependency entries that you must add manually.

### Project Structure

```
my_app/
├── Cargo.toml              # Dependencies and project metadata
├── build.rs                # Asset copying build script
├── src/
│   ├── main.rs            # Application entry point with LServer! + endpoint!
│   └── resource.rs        # Generated by build.rs on the first build
├── templates/             # Akari HTML templates
└── programfiles/          # Static assets (CSS, JS, images)
```

The build script copies `templates/` and `programfiles/` to the target directory at compile time so they're accessible at runtime.

## Core Concepts

### Endpoints

Three macro flavors are enabled by the `trans` / `semi-trans` / `attr` cargo features. Pick one per project; **`trans` is the default**. All three produce the same route definition; the separate `auto-reg` feature controls whether that definition binds during startup.

**`trans` (default): bang macro with hotaru-blocks body**

```rust
endpoint! {
    APP.url("/users/<int:id>"),
    pub get_user<HTTP> {
        let user_id = req.param("id").unwrap_or_default();
        akari_json!({ id: user_id })
    }
}
```

#### Registration modes

**Macro definition with automatic registration (default):**

```toml
[dependencies]
hotaru = { version = "0.8.6", default-features = false, features = [
    "trans", "auto-reg", "http", "tokio", "full_regex", "template"
] }
```

```rust
fn main() {
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        text_response("Hello, Hotaru!")
    }
}
```

This complete configuration is available in
[`examples/starter_trans`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_trans).

**The same macro definition with explicit registration (`auto-reg` off):**

```rust
fn main() {
    APP.bind(index).expect("bind index");
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        text_response("Hello, Hotaru!")
    }
}
```

See
[`examples/starter_trans_no_auto_reg`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_trans_no_auto_reg)
for its exact feature list.

**Manual endpoint definition without the endpoint DSL:**

```rust
async fn index_body(_ctx: &mut HttpContext) -> HttpResponse {
    text_response("Hello, Hotaru!")
}

fn main() {
    let index = Endpoint::<HTTP>::endpoint(
        "/",
        "index",
        |ctx: &mut HttpContext| Box::pin(index_body(ctx)),
    );
    APP.insert(index).expect("insert index");
    run_server!(APP);
}
```

This path does not enable `trans`, `semi-trans`, `attr`, or `auto-reg`. See
[`examples/starter_manual`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_manual).

**`semi-trans`: stacked attributes above an `fn`**

```rust
#[endpoint]
#[url("/users/<int:id>")]
pub fn get_user<HTTP>() {
    let user_id = req.param("id").unwrap_or_default();
    akari_json!({ id: user_id })
}
```

**`attr`: single attribute with args**

```rust
#[endpoint("/users/<int:id>")]
pub fn get_user<HTTP>() {
    let user_id = req.param("id").unwrap_or_default();
    akari_json!({ id: user_id })
}
```

> `akari_json!` is the JSON-response macro re-exported via `hotaru::prelude`; it already wraps `json_response(...)` so callers don't compose the two. Keys are bare idents (not `"..."`). `req.param(...)` returns `Option<String>`.

### Macro Notes

- With the default `auto-reg` feature, generated endpoint/outpoint definitions bind during startup. Without it, bind their constructors explicitly with `App::bind` or `Blueprint::bind`.
- `trans` form: brace syntax `{}` with doc comments inside the block; angle-bracket body defaults to `req`. Optional fn-style `pub fn name(req: HTTP) { ... }` is also accepted.
- Remaining readme examples use `trans`. To switch, set `default-features = false` on the `hotaru` dependency and turn on the flavor you want, e.g. `hotaru = { version = "0.8.6", default-features = false, features = ["semi-trans", "auto-reg", "http", "tokio"] }`. Cargo feature unification would otherwise keep `trans` on alongside it; remember to re-add `auto-reg`, `http`, and `tokio` when those defaults are wanted.
- See `macro_ra.md` for syntax details. Analyzer support is planned.

### Middleware

Attach a middleware to a protocol via the `ProtocolBuilder`. Add `htmstd = "0.8.6"` to your `Cargo.toml` for the bundled middleware library:

```rust
use htmstd::CookieSession;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<CookieSession>(),
        )
        .build()
);
```

`CookieSession` writes encrypted session cookies. By default, those cookies are
production-safe (`Secure`, `HttpOnly`, `SameSite=Lax`, `Path=/`). If you are
running a plain-HTTP development environment, configure the cookie safety policy
explicitly through the app config:

```rust
use htmstd::{CookieSecurity, CookieSession, CookieSessionSettings};

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .mode(RunMode::Development)
        .set_config(CookieSessionSettings::new().security(CookieSecurity::Auto))
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<CookieSession>(),
        )
        .build()
);
```

`CookieSecurity::Auto` follows `RunMode`: `Production`/`Beta` keep `Secure`
cookies, while `Development`/`Build` allow plain HTTP cookies. For production,
also configure a stable `SessionSecret` so sessions survive process restarts.

Middleware can also be attached per-endpoint via `middleware = [...]` inside the `endpoint!` block. See `example_hotaru` for the pattern.

### Templates

Render HTML with Akari via `akari_render!`. The macro looks up the template file and substitutes the named bindings:

```rust
endpoint! {
    APP.url("/profile"),
    pub profile<HTTP> {
        akari_render!("profile.html", name = "Alice")
    }
}
```

### HTTP Safety Configuration

Configure request validation per endpoint:

```rust
endpoint! {
    APP.url("/upload"),
    config = [HttpSafety::new()
        .with_max_body_size(50 * 1024 * 1024)  // 50MB
        .with_allowed_methods(vec![HttpMethod::POST])
    ],
    pub upload<HTTP> {
        // Handle file upload
    }
}
```

## Examples

Use the examples maintained in this repository:

- [`starter_trans`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_trans): `trans` DSL with default `auto-reg`
- [`starter_trans_no_auto_reg`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_trans_no_auto_reg): the same DSL with explicit `App::bind`
- [`starter_manual`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/starter_manual): manual `Endpoint::<HTTP>::endpoint` plus `App::insert`
- [`tutorial_examples`](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples/tutorial_examples): routing, middleware, multi-protocol, and TCP examples
- [All repository examples](https://github.com/Field-of-Dream-Studio/hotaru/tree/master/examples)

## Crate Ecosystem

Hotaru is built on a modular architecture:

- **[hotaru](https://crates.io/crates/hotaru)** - Main framework with convenient API
- **[hotaru_core](https://crates.io/crates/hotaru_core)** - Core protocol and routing engine
- **[hotaru_trans](https://crates.io/crates/hotaru_trans)** - Procedural macros for endpoint! and middleware!
- **[hotaru_http](https://crates.io/crates/hotaru_http)** - HTTP implementation for Hotaru
- **[hotaru_mqtt](https://crates.io/crates/hotaru_mqtt)** - MQTT implementation for Hotaru and brokers. [Repo](https://github.com/fds-pmine/hotaru_mqtt) 
- **[hotaru_tls](https://crates.io/crates/hotaru_tls)** - TLS/HTTPS implementation for Hotaru
- **[hotaru_rt_tokio](https://crates.io/crates/hotaru_rt_tokio)** - Tokio runtime backend (`TokioRuntime`)
- **[hotaru_io_tokio](https://crates.io/crates/hotaru_io_tokio)** - Tokio TCP/IO backend (`TcpTransport`, `TokioIo`)
- **[hotaru_io_futures](https://crates.io/crates/hotaru_io_futures)** - `futures-io` adapter backend (`FuturesIo`)
- **[hotaru_io_embedded](https://crates.io/crates/hotaru_io_embedded)** - `embedded-io-async` adapter backend (`EmbeddedIo`) 
- **[hotaru_lib](https://crates.io/crates/hotaru_lib)** - Utility functions (compression, encoding, etc.)
- **[htmstd](https://crates.io/crates/htmstd)** - Standard middleware library (CORS, sessions)

## Changelog

### 0.8.6 (Current)

- Rejects ambiguous HTTP/1 request framing, including conflicting `Content-Length` values and `Content-Length` combined with `Transfer-Encoding`.
- Rejects oversized lengths and chunk-size accumulation overflow before they can cross configured `HttpSafety` limits.
- Corrects chunk extension and trailer parsing, rejects unsupported transfer codings, and keeps trailer fields from replacing the request start line.
- Parses `Connection` as a case-insensitive token list across repeated fields and applies the HTTP/1.0 and HTTP/1.1 persistence defaults, with `close` taking precedence.
- Adds focused HTTP regression coverage for these boundary and persistence cases.

### 0.8.0–0.8.5

- Split Tokio runtime and TCP I/O implementations from the protocol-neutral
  core into sibling crates while keeping Tokio as the umbrella crate's default.
- Added synchronous server entry macros, endpoint/outpoint flows, optional
  automatic registration, and experimental explicit/manual registration paths.
- Made HTTP an optional facade feature, moved its public re-exports to
  `hotaru::http`, and made body compression opt-in.
- Added bounded framework I/O, `HttpSafety` limits, and typed HTTP body and
  transfer errors so malformed or incomplete input remains distinguishable.
- Expanded `no_std` and embedded compile coverage with explicit platform,
  task-mobility, and lock-backend feature selections.
- Refined routing, regex selection, middleware inheritance, and the supporting
  language and session middleware crates.
- Updated CLI scaffolds and focused starter projects around the canonical
  `trans` + `auto-reg` + Tokio + HTTP path.

APIs introduced during these releases remain pre-1.0 and experimental.

### 0.7.x

- Expanded routing, middleware inheritance, `HttpSafety`, and security tests.
- Improved worker configuration, CLI scaffolding, and endpoint syntax.
- Added the built-in constructor path and fn-style endpoint/middleware blocks.

### 0.6.x

- Introduced the protocol abstraction and improved request contexts.
- Added the `htmstd` middleware library and cookie-based sessions.

### 0.4.x and earlier

- Established the Tokio HTTP framework and Akari templating.
- Added cookies, file uploads, and form-data processing.

For the complete patch-by-patch history, see
[GitHub Releases](https://github.com/Field-of-Dream-Studio/hotaru/releases) and
[repository tags](https://github.com/Field-of-Dream-Studio/hotaru/tags).

## Learn More

- **Akari Template Engine**: https://crates.io/crates/akari
- **Homepage**: https://hotaru.rs
- **Documentation Home Page**: https://fds.rs
- **GitHub**: https://github.com/Field-of-Dream-Studio/hotaru
- **Documentation**: https://docs.rs/hotaru

| Video Resources | URL |
| --- | --- |
| Quick Tutorial | Youtube: https://www.youtube.com/watch?v=8pV-o04GuKk&t=6s <br> Bilibili: https://www.bilibili.com/video/BV1BamFB7E8n/ |

## AI-assisted development

Definitions and component declarations are maintained in
[POLICY.md](https://github.com/Field-of-Dream-Studio/hotaru/blob/main/POLICY.md#5-ai-declarations).

## 📄 License

MIT License. See [LICENSE.txt](LICENSE.txt).

Copyright (c) 2024-2026 @ [Field of Dreams Studio (FDS)](https://fds.moe) & [Project-StarFall](https://sf.fds.moe) & [PMINE-FDS](https://pmine.rs)
