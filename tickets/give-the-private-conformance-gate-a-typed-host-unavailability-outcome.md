---
id: give-the-private-conformance-gate-a-typed-host-unavailability-outcome
title: Give the private conformance gate a typed host-unavailability outcome
status: todo
priority: p2
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance]
---
## User-visible outcome

The private `tiler-conformance` gate reports an unmeasurable host through a typed outcome that cannot be compared equal to a pass and does not read an ambient environment variable, matching the shape the independent backend fixture demonstrated.

## Why this exists

Filed 2026-08-22 by `worker-packet` out of the second re-derivation on `decide-the-backend-provider-conformance-harness-public-surface`. Itemizing the fixture against the accepted seams found exactly one extraction with real substance, and it is **internal** rather than a public facade — which is why it is a separate, non-Tom ticket rather than part of that public-boundary answer.

**Fact — the gate's current shape.** `crates/tiler-conformance/src/measurement.rs` carries `Measured<T>` with an `Unavailable(String)` arm and a `REQUIRE_MEASUREMENT` constant naming `TILER_REQUIRE_METAL_CONFORMANCE`. The crate header already states the obligation this serves: a host that cannot measure `reports the measured half as unavailable, naming what was missing`, and `never skips silently`.

**Fact — the fixture demonstrated a strictly better private design, and each property is checked.** In `crates/tiler-conformance/tests/independent_backend/nodefold_adapter.rs`: `ExecutionOutcome` derives neither `PartialEq` nor `Default`, so an unavailable outcome cannot compare equal to a completed one; `completed()` answers `None` for an unavailable host; `agrees_with_reference` takes bits rather than an outcome, so reaching a comparison requires destructuring a completion; and `HostPolicy` is applied by `apply_policy` **at the call site**, with no environment read anywhere in the file. `an_unavailable_execution_host_is_typed_and_cannot_pass` watches both policies with the adapter's report held identical.

## Required work

- Re-audit both Facts at your base before editing.
- This is an internal change to a crate with no public surface. It must add no `pub` item; `crates/tiler-conformance/src/lib.rs` says under `# Public surface` that `There is none`, and that is an accepted boundary under ADR 0075.
- Decide deliberately whether the ambient `TILER_REQUIRE_METAL_CONFORMANCE` switch is retired or retained, and say which. If it is retained, say what reads it and why a call-site policy could not replace it; the gate is invoked from `make full`, so a caller-owned policy needs a caller.
- Perturb the subject, not the assertion: make an unavailable outcome reachable where a pass is expected and quote the failure text. Perturb the no-equality property separately from the no-ambient-read property, since they are independent.

## Non-goals

Any public export. This is explicitly not the conformance facade — that question is `decide-the-backend-provider-conformance-harness-public-surface`, and its second re-derivation recommends publishing none.

## Closes when

The private gate's unavailable outcome is typed, has no path to a pass, and its policy is the caller's; the crate still exports nothing; and each property has been watched failing separately.
