//! App-level `combine()`: two independently assembled server blueprints are
//! merged into one app before serving. Left side wins every conflict.
//!
//! Blueprint statics are the auto-reg target instead of a shared `APP`
//! static: the `endpoint!` hook binds each constructor into its Blueprint at
//! load time, then `main()` builds two independent `Arc<Server>` values from
//! them and combines them dynamically. `try_combine` still needs unique
//! ownership of each built `Arc<Server>`, which the builder provides fresh.
//!
//! Run `cargo run -p combine_example`, then:
//!   curl http://127.0.0.1:3005/hello   -> served by blueprint A (collision: A wins)
//!   curl http://127.0.0.1:3005/world   -> served by blueprint B (adopted subtree)

use std::sync::LazyLock;

use hotaru::http::*;
use hotaru::prelude::*;

static BLUEPRINT_A: LazyLock<Blueprint<TcpTransport, InboundOnly>> = LazyLock::new(|| {
    Blueprint::new()
        .with_protocol(HTTP::server(HttpSafety::default()))
        .expect("blueprint A protocol")
});

static BLUEPRINT_B: LazyLock<Blueprint<TcpTransport, InboundOnly>> = LazyLock::new(|| {
    Blueprint::new()
        .with_protocol(HTTP::server(HttpSafety::default()))
        .expect("blueprint B protocol")
});

endpoint! {
    BLUEPRINT_A: "/hello", pub hello_a<HTTP> {
        response_templates::normal_response(200u16, "hello from blueprint A (left wins)\n")
    }
}

endpoint! {
    BLUEPRINT_B: "/hello", pub hello_b<HTTP> {
        response_templates::normal_response(200u16, "hello from blueprint B (should never be served)\n")
    }
}

endpoint! {
    BLUEPRINT_B: "/world", pub world<HTTP> {
        response_templates::normal_response(200u16, "world from blueprint B (adopted subtree)\n")
    }
}

fn main() {
    let a = build_server(&BLUEPRINT_A, "127.0.0.1:3005");
    let b = build_server(&BLUEPRINT_B, "127.0.0.1:9999");

    // While another handle to A exists, the merge is refused and both apps
    // come back untouched inside the error.
    let held = a.clone();
    let (a, b) = match a.try_combine(b) {
        Ok(_) => unreachable!("A is still shared"),
        Err(AppInUse { app, other }) => {
            println!("try_combine refused while a handle to A is held");
            (app, other)
        }
    };
    drop(held);

    // Both blueprints register `/hello`; A's handler and binding survive.
    let app = a.try_combine(b).expect("no extra handles held now");
    println!("blueprints combined; serving on 127.0.0.1:3005");
    run_server!(app);
}

fn build_server(blueprint: &Blueprint<TcpTransport, InboundOnly>, binding: &str) -> Arc<Server> {
    Server::new()
        .binding(binding)
        .apply(blueprint)
        .expect("apply blueprint")
        .build()
}
