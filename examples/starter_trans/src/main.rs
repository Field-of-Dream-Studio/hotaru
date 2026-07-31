//! Minimal Hotaru starter — `trans` flavour (bang macro, hotaru-block body).
//!
//! One server on 127.0.0.1:3010 with a single `/` endpoint that renders one
//! HTML response with one substituted variable.

use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3010")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        akari_render!("home.html", name = "Hotaru", variant = "trans")
    }
}
