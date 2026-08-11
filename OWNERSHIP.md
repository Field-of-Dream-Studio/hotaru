# Hotaru Component Ownership and AI Tier Assignments

This file is the canonical registry of who owns which components in Hotaru and
which AI-collaboration tier applies to each component. The rules that govern
these roles, the tier definitions themselves, and the appointment/succession
process live in [GOVERNANCE.md](./GOVERNANCE.md).

**Short names** in the tables below are the values to write in the `Family:`
and `Component:` fields of the Update Report and QA record forms.

## Family overview

| Short | Full family name | Family Maintainer |
| --- | --- | --- |
| Trans | Core framework | [@Redstone-D](https://github.com/Redstone-D) |
| Facade | Facade and tooling | [@JerrySu5379](https://github.com/JerrySu5379) |
| Protocol | Protocol implementations | [@Redstone-D](https://github.com/Redstone-D) |
| Runtime | Runtime implementations | [@JerrySu5379](https://github.com/JerrySu5379) |
| IO | I/O implementations | [@JerrySu5379](https://github.com/JerrySu5379) |

## Component ownership

### Trans — Core framework

Core contracts and the procedural-macro DSL.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Short | Component | Files and directories | Component Maintainer |
| --- | --- | --- | --- |
| Defs | Core contracts and semantics | `hotaru_core/**` except the URL paths below | [@Redstone-D](https://github.com/Redstone-D) |
| DSL | DSL and procedural macros | `hotaru_trans/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Facade — Facade and tooling

Routing, the public facade and feature surface, CLI tooling, templates, and
shared user-facing utilities.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Short | Component | Files and directories | Component Maintainer |
| --- | --- | --- | --- |
| URL | Routing and URL semantics | `hotaru_core/src/url.rs`, `hotaru_core/src/url/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Reexport | Facade and public feature surface | `hotaru/src/lib.rs`, `hotaru/src/prelude.rs`, `hotaru/src/http.rs`, `hotaru/src/test.rs`, `hotaru/Cargo.toml`, `hotaru/readme.md` | [@Redstone-D](https://github.com/Redstone-D) |
| CLI | CLI and project templates | `hotaru/src/main.rs`, `templates/**`, `programfiles/**`, `hotaru_style_guide/**` | [@Redstone-D](https://github.com/Redstone-D) |
| Utils | Shared utilities | `hotaru_lib/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Protocol — Protocol implementations

Wire protocols, protocol-specific security, and standard middleware.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Short | Component | Files and directories | Component Maintainer |
| --- | --- | --- | --- |
| Web | HTTP, TLS, and web middleware | `hotaru_http/**`, `hotaru_tls/**`, `htmstd/**`, `ahttpm/**` | [@Redstone-D](https://github.com/Redstone-D) |
| MQTT | MQTT client and broker | [`Field-of-Dream-Studio/hotaru_mqtt`](https://github.com/fds-pmine/hotaru_mqtt) | [@JerrySu5379](https://github.com/JerrySu5379) |
| Experimental | Experimental protocol integrations | `h2per/**`, `hotaru_grpc/**` | [@Redstone-D](https://github.com/Redstone-D), [@JerrySu5379](https://github.com/JerrySu5379) |

The MQTT repository should maintain its own matching ownership rules.

### Runtime — Runtime implementations

Runtime scheduling, spawning, and runtime-specific integration.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Short | Component | Files and directories | Component Maintainer |
| --- | --- | --- | --- |
| RT-Tokio | Tokio runtime | `hotaru_rt_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| RT-Embassy | Embassy runtime | `hotaru_rt_embassy/**` | [@zkmaojack](https://github.com/zkmaojack) |

### IO — I/O implementations

Adapters between Hotaru's transport contracts and concrete I/O ecosystems.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Short | Component | Files and directories | Component Maintainer |
| --- | --- | --- | --- |
| IO-Tokio | Tokio I/O | `hotaru_io_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| IO-Futures | Futures I/O | `hotaru_io_futures/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| IO-Embedded | Embedded I/O | `hotaru_io_embedded/**` | [@zkmaojack](https://github.com/zkmaojack) |

## AI tier assignments

Each Family Maintainer chooses and updates the declarations for components in
their family. When scopes inside one component use different tiers, the more
specific declaration applies. Tier definitions are in
[GOVERNANCE.md § 3](./GOVERNANCE.md#3-ai-declarations).

| Family | Component (scope) | Tier |
| --- | --- | --- |
| Trans | Defs (`app`, `connection`, `executable`, `protocol`) | **Author-Owned** |
| Trans | Defs (remaining) | **Human-Led** |
| Trans | DSL (`endpoint`, `outpoint`, `middleware`) | **Author-Owned** |
| Trans | DSL (remaining) | **Human-Led** |
| Facade | URL | **Author-Owned** |
| Facade | Reexport | **Co-Authored** |
| Facade | CLI | **Co-Authored** |
| Facade | Utils | **Human-Led** |
| Protocol | Web (HTTP, CORS, session middleware) | **Human-Led** |
| Protocol | Web (TLS, `ahttpm`, remaining middleware) | **Co-Authored** |
| Protocol | MQTT (client, general implementation) | **Human-Led** |
| Protocol | MQTT (broker, traits) | **Co-Authored** |
| Protocol | Experimental | **Co-Authored** |
| Runtime | RT-Tokio, RT-Embassy | **Co-Authored** |
| IO | IO-Tokio, IO-Futures, IO-Embedded | **Co-Authored** |
