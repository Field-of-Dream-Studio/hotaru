//! Blueprint storage and admission tests (Stage 6).
//!
//! Built-App and builder application remain Stage 7 concerns.

use core::any::TypeId;
use core::convert::Infallible;
use core::future::Future;

use crate::app::{
    Both, InboundOnly, OutboundOnly,
    common::{OperationalConfig, RunMode, RuntimeConfig},
};
use crate::connection::{
    MaybeSend,
    test_support::{TestMeta, TestOutbound, TestTransport, TestWire},
};
use crate::executable::def::{Endpoint, EndpointHandler, Outpoint, OutpointHandler};
use crate::executable::middleware::{
    AsyncFinalHandler, AsyncMiddleware, AsyncMiddlewareChain, BoxFuture, NextFn,
};
use crate::marker::MaybeSendSync;
use crate::prelude::{Arc, Box, Vec, vec};
use crate::protocol::{
    Channel, DefaultProtocolError, Protocol, ProtocolFlow, ProtocolRole, RequestContext,
};
use crate::url::UrlRoot;

use super::{Blueprint, BlueprintError, ConfiguredBlueprint, HomoBlueprint, ProtocolDef};

#[derive(Debug)]
struct TestError;

impl core::fmt::Display for TestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("test error")
    }
}

impl core::error::Error for TestError {}
impl DefaultProtocolError for TestError {}

impl From<Infallible> for TestError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Clone)]
enum NoChannel {}

impl Channel for NoChannel {
    fn is_open(&self) -> bool {
        match *self {}
    }

    fn close(&self) {}
}

#[derive(Default)]
struct TestCtx;

impl RequestContext for TestCtx {
    type Request = ();
    type Response = ();
    type Error = TestError;
    type Channel = NoChannel;

    fn handle_error(&mut self) {}

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Server
    }

    fn inject_request(&mut self, _: ()) {}

    fn into_response(self) {}
}

#[derive(Clone)]
struct TestProto<const N: u8>;

impl<const N: u8> Protocol for TestProto<N> {
    type Wire = TestWire;
    type TS = TestTransport;
    type Channel = NoChannel;
    type Stream = ();
    type Message = ();
    type Context = TestCtx;

    fn name(&self) -> &'static str {
        "blueprint-test"
    }

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Server
    }

    fn lit_parser(input: &str) -> Vec<&str> {
        input.split('/').collect()
    }

    fn detect(_: &[u8]) -> bool {
        true
    }

    fn open_channel(self, reader: TestWire, _writer: TestWire, _meta: TestMeta) -> NoChannel {
        match reader {}
    }

    fn handle(
        _channel: &NoChannel,
        _runtime: Arc<RuntimeConfig>,
        _root: Arc<UrlRoot<TestCtx, TestTransport>>,
    ) -> impl Future<Output = Result<ProtocolFlow, TestError>> + MaybeSend {
        async { Ok(ProtocolFlow::Close) }
    }

    fn acquire_channel(
        &self,
        _runtime: &Arc<RuntimeConfig>,
        _outbound: Arc<TestOutbound>,
    ) -> impl Future<Output = Result<NoChannel, TestError>> + MaybeSend {
        async { Err(TestError) }
    }

    fn send(ctx: TestCtx) -> impl Future<Output = Result<TestCtx, TestError>> + MaybeSend {
        async move { Ok(ctx) }
    }

    fn install_channel(_ctx: &mut TestCtx, channel: NoChannel) {
        match channel {}
    }
}

fn final_handler() -> Arc<dyn AsyncFinalHandler<TestCtx>> {
    Arc::new(|ctx: TestCtx| async move { Ok(ctx) })
}

fn endpoint(url: &str, name: &str) -> Endpoint<TestProto<0>> {
    Endpoint::from_final_handler(url, name, final_handler())
}

fn body_middleware() -> Arc<dyn AsyncMiddleware<TestCtx>> {
    struct BodyMiddleware;

    impl AsyncMiddleware<TestCtx> for BodyMiddleware {
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }

        fn return_self() -> Self {
            Self
        }

        fn handle<'a>(&self, context: TestCtx, next: Box<NextFn<TestCtx>>) -> BoxFuture<TestCtx> {
            next(context)
        }
    }

    Arc::new(BodyMiddleware)
}

fn outpoint(url: &str, name: &str) -> Outpoint<TestProto<0>> {
    Outpoint::outpoint(url, name, body_middleware())
}

fn inbound() -> Blueprint<TestTransport, InboundOnly> {
    Blueprint::new().with_protocol(TestProto::<0>).unwrap()
}

fn outbound() -> Blueprint<TestTransport, OutboundOnly> {
    Blueprint::new().with_protocol(TestProto::<0>).unwrap()
}

fn both() -> Blueprint<TestTransport, Both> {
    Blueprint::new().with_protocol(TestProto::<0>).unwrap()
}

fn endpoint_group(
    blueprint: &Blueprint<TestTransport, InboundOnly>,
) -> &HomoBlueprint<TestProto<0>, EndpointHandler<TestProto<0>>> {
    blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, EndpointHandler<TestProto<0>>>>()
        })
        .expect("endpoint group exists")
}

#[test]
fn erased_group_downcasts_to_concrete_group() {
    assert!(
        inbound().groups()[0]
            .as_any()
            .downcast_ref::<HomoBlueprint<TestProto<0>, EndpointHandler<TestProto<0>>>>()
            .is_some()
    );
}

#[test]
fn inbound_admits_endpoint() {
    let blueprint = inbound();
    blueprint.insert(endpoint("/a", "a")).unwrap();
    assert_eq!(
        endpoint_group(&blueprint).access_points.read().defs.len(),
        1
    );
}

#[test]
fn outbound_admits_outpoint() {
    let blueprint = outbound();
    blueprint.insert(outpoint("/send", "send")).unwrap();
    let group = blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, OutpointHandler<TestProto<0>>>>()
        })
        .expect("outpoint group exists");
    assert_eq!(group.access_points.read().defs.len(), 1);
}

#[test]
fn both_admits_either_flavour() {
    let blueprint = both();
    blueprint.insert(endpoint("/a", "a")).unwrap();
    blueprint.insert(outpoint("/send", "send")).unwrap();
    assert_eq!(blueprint.groups().len(), 2);
}

#[test]
fn both_groups_share_protocol_arc() {
    let blueprint = both();
    let endpoint = blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, EndpointHandler<TestProto<0>>>>()
        })
        .unwrap();
    let outpoint = blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, OutpointHandler<TestProto<0>>>>()
        })
        .unwrap();

    assert!(Arc::ptr_eq(
        endpoint.protocol_def(),
        outpoint.protocol_def()
    ));
}

#[test]
fn flavour_groups_are_distinct() {
    let blueprint = both();
    let endpoint = blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, EndpointHandler<TestProto<0>>>>()
        })
        .unwrap() as *const _;
    let outpoint = blueprint
        .groups()
        .iter()
        .find_map(|group| {
            group
                .as_any()
                .downcast_ref::<HomoBlueprint<TestProto<0>, OutpointHandler<TestProto<0>>>>()
        })
        .unwrap() as *const _;

    assert_ne!(endpoint as usize, outpoint as usize);
}

#[test]
fn clone_shares_ap_storage() {
    let blueprint = inbound();
    let clone = blueprint.clone();
    blueprint.insert(endpoint("/a", "a")).unwrap();
    assert_eq!(endpoint_group(&clone).access_points.read().defs.len(), 1);
}

#[test]
fn clone_freezes_protocol_set() {
    let blueprint = inbound();
    let _clone = blueprint.clone();
    assert_eq!(
        blueprint.with_protocol(TestProto::<1>).unwrap_err(),
        BlueprintError::SharedBlueprint
    );
}

#[test]
fn duplicate_protocol_rejected() {
    let error = Blueprint::<TestTransport, InboundOnly>::new()
        .with_protocol(TestProto::<0>)
        .unwrap()
        .with_protocol(TestProto::<0>)
        .unwrap_err();

    assert!(matches!(error, BlueprintError::DuplicateProtocol(_)));
}

#[test]
fn distinct_protocols_admitted() {
    let blueprint = Blueprint::<TestTransport, InboundOnly>::new()
        .with_protocol(TestProto::<0>)
        .unwrap()
        .with_protocol(TestProto::<1>)
        .unwrap();

    assert_eq!(blueprint.groups().len(), 2);
    let ids: Vec<TypeId> = blueprint
        .groups()
        .iter()
        .map(|group| group.protocol_type_id())
        .collect();
    assert!(ids.contains(&TypeId::of::<TestProto<0>>()));
    assert!(ids.contains(&TypeId::of::<TestProto<1>>()));
}

#[test]
fn admission_without_protocol_group_errors() {
    let blueprint: Blueprint<TestTransport, InboundOnly> = Blueprint::new();
    assert!(matches!(
        blueprint.insert(endpoint("/a", "a")).unwrap_err(),
        BlueprintError::ProtocolNotFound(_)
    ));
}

#[test]
fn bind_calls_constructor_once() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    fn constructor() -> Endpoint<TestProto<0>> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        endpoint("/a", "a")
    }

    CALLS.store(0, Ordering::SeqCst);
    inbound().bind(constructor).unwrap();
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn extend_retains_batch() {
    let blueprint = inbound();
    blueprint
        .extend(vec![endpoint("/a", "a"), endpoint("/b", "b")])
        .unwrap();
    assert_eq!(
        endpoint_group(&blueprint).access_points.read().defs.len(),
        2
    );
}

#[test]
fn materialize_is_repeatable() {
    let blueprint = inbound();
    blueprint.insert(endpoint("/a", "a")).unwrap();

    let first = blueprint.materialize_registry().unwrap();
    let second = blueprint.materialize_registry().unwrap();

    assert!(first.entry::<TestProto<0>>().is_some());
    assert!(second.entry::<TestProto<0>>().is_some());
    assert_eq!(
        endpoint_group(&blueprint).access_points.read().defs.len(),
        1
    );

    let first_root = first.url::<TestProto<0>>().unwrap();
    let second_root = second.url::<TestProto<0>>().unwrap();
    assert!(!Arc::ptr_eq(&first_root, &second_root));
}

#[test]
fn configured_delegates_admission_and_stores_defaults() {
    fn constructor() -> Endpoint<TestProto<0>> {
        endpoint("/bound", "bound")
    }

    let configured = ConfiguredBlueprint::new(inbound())
        .with_mode(RunMode::Production)
        .with_operational(OperationalConfig::from_server_parts(
            4,
            crate::app::common::TimeoutSetting::Seconds(9),
            crate::app::common::TimeoutSetting::Seconds(3),
        ));

    configured.bind(constructor).unwrap();
    configured
        .insert(endpoint("/inserted", "inserted"))
        .unwrap();
    configured
        .extend(vec![
            endpoint("/extended-a", "extended-a"),
            endpoint("/extended-b", "extended-b"),
        ])
        .unwrap();

    assert_eq!(configured.mode(), Some(&RunMode::Production));
    assert_eq!(configured.operational().unwrap().worker(), 4);
    assert_eq!(
        endpoint_group(configured.blueprint())
            .access_points
            .read()
            .defs
            .len(),
        4
    );
}

#[test]
fn blueprint_storage_satisfies_active_mobility_marker() {
    fn assert_maybe_send_sync<T: MaybeSendSync>() {}

    assert_maybe_send_sync::<Blueprint<TestTransport, InboundOnly>>();
    assert_maybe_send_sync::<ConfiguredBlueprint<TestTransport, OutboundOnly>>();
}

#[allow(dead_code)]
fn protocol_def_constructor_is_public() {
    let _ = ProtocolDef::new(TestProto::<0>, AsyncMiddlewareChain::<TestCtx>::new());
}

// ---------------------------------------------------------------------------
// Stage-7.2 timeout preservation tests.
//
// Design note: 0.8 keeps two role-specific builder fields —
// `AppBuilder.max_frame_process_timeout` (server) and `.request_timeout`
// (client) — mirroring `OperationalConfig` 1:1. Both defaults resolve to 5s.
// All timeout APIs take `TimeoutSetting`; no `usize` compat setter exists.
// ---------------------------------------------------------------------------

mod timeouts {
    use crate::app::common::{
        AppBuilder, OperationalConfig, TimeoutSetting,
        builder::{ClientRole, ServerRole},
    };
    use crate::app::runtime::test_support::PhantomRt;
    use crate::app::{Blueprint, InboundOnly, OutboundOnly};
    use crate::connection::test_support::TestTransport;

    use super::ConfiguredBlueprint;

    fn variants() -> [TimeoutSetting; 4] {
        [
            TimeoutSetting::Inherit,
            TimeoutSetting::Disabled,
            TimeoutSetting::Milliseconds(500),
            TimeoutSetting::Seconds(7),
        ]
    }

    /// T17+T18 — `apply_configured` copies the configured timeout into the
    /// matching builder field losslessly for every `TimeoutSetting` variant,
    /// on both roles.
    #[test]
    fn apply_configured_preserves_role_timeouts_losslessly() {
        for input in variants() {
            let configured_server =
                ConfiguredBlueprint::new(Blueprint::<TestTransport, InboundOnly>::new())
                    .with_operational(OperationalConfig::from_server_parts(
                        1,
                        TimeoutSetting::Inherit,
                        input,
                    ));
            let server = AppBuilder::<ServerRole, TestTransport, PhantomRt>::new()
                .apply_configured(&configured_server)
                .expect("server apply_configured");
            assert!(
                matches_timeout(
                    server.get_max_frame_process_timeout().unwrap(),
                    input,
                ),
                "server frame timeout: expected {input:?}",
            );

            let mut operational = OperationalConfig::default();
            operational.set_request_timeout(input);
            let configured_client =
                ConfiguredBlueprint::new(Blueprint::<TestTransport, OutboundOnly>::new())
                    .with_operational(operational);
            let client = AppBuilder::<ClientRole, TestTransport, PhantomRt>::new()
                .apply_configured(&configured_client)
                .expect("client apply_configured");
            assert!(
                matches_timeout(client.get_request_timeout().unwrap(), input),
                "client request timeout: expected {input:?}",
            );
        }
    }

    /// T18b — An explicit setter beats a configured default. Setter-before and
    /// setter-after go through the same code path (`apply_configured` only
    /// fills `None` fields), so one order suffices per role.
    #[test]
    fn explicit_setter_beats_configured_default() {
        let configured_server =
            ConfiguredBlueprint::new(Blueprint::<TestTransport, InboundOnly>::new())
                .with_operational(OperationalConfig::from_server_parts(
                    1,
                    TimeoutSetting::Inherit,
                    TimeoutSetting::Seconds(11),
                ));
        let server = AppBuilder::<ServerRole, TestTransport, PhantomRt>::new()
            .apply_configured(&configured_server)
            .expect("server apply_configured")
            .max_frame_process_timeout(TimeoutSetting::Seconds(3));
        assert!(matches_timeout(
            server.get_max_frame_process_timeout().unwrap(),
            TimeoutSetting::Seconds(3),
        ));

        let mut operational = OperationalConfig::default();
        operational.set_request_timeout(TimeoutSetting::Seconds(11));
        let configured_client =
            ConfiguredBlueprint::new(Blueprint::<TestTransport, OutboundOnly>::new())
                .with_operational(operational);
        let client = AppBuilder::<ClientRole, TestTransport, PhantomRt>::new()
            .apply_configured(&configured_client)
            .expect("client apply_configured")
            .request_timeout(TimeoutSetting::Seconds(3));
        assert!(matches_timeout(
            client.get_request_timeout().unwrap(),
            TimeoutSetting::Seconds(3),
        ));
    }

    /// TimeoutSetting doesn't implement PartialEq, so pattern-match structurally.
    fn matches_timeout(a: TimeoutSetting, b: TimeoutSetting) -> bool {
        match (a, b) {
            (TimeoutSetting::Inherit, TimeoutSetting::Inherit) => true,
            (TimeoutSetting::Disabled, TimeoutSetting::Disabled) => true,
            (TimeoutSetting::Fixed(x), TimeoutSetting::Fixed(y)) => x == y,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Stage-7.1 App::apply tests (T15, T16).
// ---------------------------------------------------------------------------

mod apply {
    use core::marker::PhantomData;

    use crate::app::common::{OperationalConfig, RuntimeConfig};
    use crate::app::instance::{App, InboundOnly, InboundState};
    use crate::app::registry::ProtocolRegistryKind;
    use crate::app::runtime::test_support::PhantomRt;
    use crate::connection::test_support::TestTransport;
    use crate::executable::middleware::AsyncMiddlewareChain;
    use crate::prelude::Arc;
    use crate::url::UrlRoot;

    use super::{
        Blueprint, BlueprintError, Endpoint, TestCtx, TestProto, endpoint,
    };

    fn built_server_with_proto0()
    -> Arc<App<TestTransport, PhantomRt, InboundOnly>> {
        Arc::new(App::<TestTransport, PhantomRt, InboundOnly> {
            registry: ProtocolRegistryKind::single(
                TestProto::<0>,
                Arc::new(UrlRoot::new()),
                AsyncMiddlewareChain::<TestCtx>::new(),
            ),
            inbound_state: InboundState {
                binding: (),
                inbound: Default::default(),
            },
            outbound_state: (),
            runtime: Arc::new(RuntimeConfig::new()),
            config: OperationalConfig::default(),
            _rt: PhantomData,
            _target: PhantomData,
        })
    }

    fn blueprint_with(defs: Vec<Endpoint<TestProto<0>>>) -> Blueprint<TestTransport, InboundOnly> {
        let bp = Blueprint::<TestTransport, InboundOnly>::new()
            .with_protocol(TestProto::<0>)
            .unwrap();
        bp.extend(defs).unwrap();
        bp
    }

    /// T15 — Applying to a built App registers each retained route into the
    /// existing protocol entry, and a second apply rebinds the same names in
    /// place rather than creating a fresh protocol instance.
    #[test]
    fn app_apply_registers_and_reapplies_in_place() {
        let app = built_server_with_proto0();
        let entry_ptr =
            app.registry.entry::<TestProto<0>>().unwrap() as *const _;

        let bp = blueprint_with(vec![endpoint("/a", "a"), endpoint("/b", "b")]);
        app.apply(&bp).expect("first apply");

        let after_first = app.registry.entry::<TestProto<0>>().unwrap();
        assert!(after_first.access_points.contains("a"));
        assert!(after_first.access_points.contains("b"));
        assert_eq!(after_first.access_points.len(), 2);
        assert_eq!(after_first as *const _, entry_ptr, "entry must not be swapped");

        // Second apply — rebind semantics: same names, no duplicates.
        app.apply(&bp).expect("second apply");
        let after_second = app.registry.entry::<TestProto<0>>().unwrap();
        assert_eq!(after_second.access_points.len(), 2);
        assert_eq!(
            after_second as *const _, entry_ptr,
            "entry pointer must remain stable across reapply"
        );
    }

    /// T16 — Full preflight: a blueprint carrying a protocol the App does
    /// not own is rejected before any group mutates the App.
    #[test]
    fn app_apply_rejects_missing_protocol_before_mutating() {
        let app = built_server_with_proto0();
        let entry = app.registry.entry::<TestProto<0>>().unwrap();
        assert_eq!(entry.access_points.len(), 0);

        // Blueprint has proto 0 (matches app) AND proto 1 (missing).
        let bp = Blueprint::<TestTransport, InboundOnly>::new()
            .with_protocol(TestProto::<0>)
            .unwrap()
            .with_protocol(TestProto::<1>)
            .unwrap();
        bp.extend(vec![endpoint("/a", "a")]).unwrap();

        let error = app.apply(&bp).expect_err("missing protocol must be rejected");
        assert!(matches!(error, BlueprintError::ProtocolNotFound(_)));

        // Preflight ran before any mutation — the matching-proto group must
        // not have been applied.
        let entry = app.registry.entry::<TestProto<0>>().unwrap();
        assert_eq!(
            entry.access_points.len(),
            0,
            "preflight failure must leave the App untouched"
        );
    }
}
