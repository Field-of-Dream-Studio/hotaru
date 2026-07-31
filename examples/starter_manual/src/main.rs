//! Minimal Hotaru starter — no macros.
//!
//! Same behaviour as `starter_trans`, built by constructing `Endpoint::<HTTP>`
//! by hand and calling `APP.insert(...)` directly. Shows what the `endpoint!`
//! macros expand to at their core.

use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3013")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

async fn index_body(_ctx: &mut HttpContext) -> HttpResponse {
    akari_render!("home.html", name = "Hotaru", variant = "manual")
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
