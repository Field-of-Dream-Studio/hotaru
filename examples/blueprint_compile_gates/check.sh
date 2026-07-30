#!/bin/sh
set -eu

cd "$(dirname "$0")"

# Positive control: public Blueprint types resolve through the umbrella.
cargo check --quiet

check_gate() {
    feature="$1"
    code="$2"
    output="target/${feature}.stderr"

    if cargo check --quiet --features "$feature" >"$output" 2>&1; then
        echo "gate unexpectedly compiled: $feature" >&2
        exit 1
    fi

    if ! grep -q "error\\[$code\\]" "$output"; then
        echo "gate failed for the wrong reason: $feature (wanted $code)" >&2
        sed -n '1,160p' "$output" >&2
        exit 1
    fi
}

check_gate inbound_rejects_outpoint E0308
check_gate outbound_rejects_endpoint E0308
check_gate erased_trait_is_private E0603
check_gate blueprint_has_no_build E0599
check_gate configured_has_no_build E0599

# Stage 7 gates: App/builder application refuses wrong-target and configured.
check_gate server_builder_rejects_outbound E0308
check_gate server_app_rejects_outbound E0308
check_gate server_bind_rejects_outpoint E0308
check_gate built_app_rejects_configured E0308
