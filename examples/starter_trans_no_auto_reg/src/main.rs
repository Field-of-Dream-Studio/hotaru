//! Minimal Hotaru starter — `trans` macro but with `auto-reg` disabled.
//!
//! Same syntax as `starter_trans`, but the `endpoint!` macro no longer emits a
//! `#[ctor]` hook, so `main()` binds the constructor by hand before
//! `run_server!`. Useful on bare-metal targets (`target_os = "none"`) where the
//! built-in ctor's linker sections do not fire.

use hotaru::http::*;
use hotaru::prelude::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3014")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    APP.bind(index).expect("bind index");
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        akari_render!("home.html", name = "Hotaru", variant = "trans_no_auto_reg")
    }
}
