use hotaru::hotaru_core::prelude::Arc;
use hotaru::hotaru_http::HTTP;
use hotaru::{
    Blueprint, ConfiguredBlueprint, Endpoint, InboundOnly, OutboundOnly, Outpoint, Protocol,
    Server, TokioRuntime,
};

type TS = <HTTP as Protocol>::TS;

#[allow(dead_code)]
fn public_root_and_prelude_exports(
    _root_blueprint: Blueprint<TS, InboundOnly>,
    _root_configured: ConfiguredBlueprint<TS, OutboundOnly>,
    _root_error: hotaru::BlueprintError,
    _root_both: hotaru::Both,
    _prelude_blueprint: hotaru::prelude::Blueprint<TS, hotaru::prelude::InboundOnly>,
    _prelude_configured: hotaru::prelude::ConfiguredBlueprint<TS, hotaru::prelude::OutboundOnly>,
    _prelude_error: hotaru::prelude::BlueprintError,
    _prelude_both: hotaru::prelude::Both,
) {
}

#[cfg(feature = "inbound_rejects_outpoint")]
fn gate(blueprint: &Blueprint<TS, InboundOnly>, definition: Outpoint<HTTP>) {
    let _ = blueprint.insert(definition);
}

#[cfg(feature = "outbound_rejects_endpoint")]
fn gate(blueprint: &Blueprint<TS, OutboundOnly>, definition: Endpoint<HTTP>) {
    let _ = blueprint.insert(definition);
}

#[cfg(feature = "server_builder_rejects_outbound")]
fn gate(blueprint: &Blueprint<TS, OutboundOnly>) {
    let _ = Server::<TS, TokioRuntime>::new().apply(blueprint);
}

#[cfg(feature = "server_app_rejects_outbound")]
fn gate(app: Arc<Server<TS, TokioRuntime>>, blueprint: &Blueprint<TS, OutboundOnly>) {
    let _ = app.apply(blueprint);
}

#[cfg(feature = "server_bind_rejects_outpoint")]
fn rejected_constructor() -> Outpoint<HTTP> {
    panic!("type-check only")
}

#[cfg(feature = "server_bind_rejects_outpoint")]
fn gate(app: Arc<Server<TS, TokioRuntime>>) {
    let _ = app.bind(rejected_constructor);
}

#[cfg(feature = "erased_trait_is_private")]
use hotaru::hotaru_core::app::blueprint::HomoBluePrintTrait;

#[cfg(feature = "blueprint_has_no_build")]
fn gate(blueprint: Blueprint<TS, InboundOnly>) {
    let _ = blueprint.build();
}

#[cfg(feature = "configured_has_no_build")]
fn gate(blueprint: ConfiguredBlueprint<TS, InboundOnly>) {
    let _ = blueprint.build();
}

#[cfg(feature = "built_app_rejects_configured")]
fn gate(
    app: Arc<Server<TS, TokioRuntime>>,
    configured: &ConfiguredBlueprint<TS, InboundOnly>,
) {
    let _ = app.apply(configured);
}

fn main() {}
