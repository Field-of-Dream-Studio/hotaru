//! Minimal Hotaru starter — `semi-trans` flavour (stacked attributes on `fn`).
//!
//! Same behaviour as `starter_trans`, different endpoint syntax.

use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3011")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}

#[endpoint]
#[url("/")]
pub fn index<HTTP>() {
    akari_render!("home.html", name = "Hotaru", variant = "semi_trans")
}
