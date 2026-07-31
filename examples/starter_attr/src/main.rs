//! Minimal Hotaru starter — `attr` flavour (single attribute with args).
//!
//! Same behaviour as `starter_trans`, different endpoint syntax.

use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3012")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}

#[endpoint("/")]
pub fn index<HTTP>() {
    akari_render!("home.html", name = "Hotaru", variant = "attr")
}
