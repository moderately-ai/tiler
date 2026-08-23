---
id: give-the-private-conformance-gate-a-typed-host-unavailability-outcome
title: Give the private conformance gate a typed host-unavailability outcome
status: in-progress
priority: p2
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance]
claimed_from: todo
assignee: worker-unavail
lease_expires_at: 1787457351
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

## Coordinator re-audit at `d3170995`, 2026-08-22 — both Facts verified, plus one thing that looks like a defect and is not

**Fact 1 — verified.** `crates/tiler-conformance/src/measurement.rs` carries `Unavailable(String)` and `pub(crate) const REQUIRE_MEASUREMENT: &str = "TILER_REQUIRE_METAL_CONFORMANCE";`. The single read is `std::env::var_os(REQUIRE_MEASUREMENT).is_none()` inside an `assert!` in `require_or_report`.

**Fact 2 — verified.** The fixture's `ExecutionOutcome` in `crates/tiler-conformance/tests/independent_backend/nodefold_adapter.rs` derives only `Clone, Debug` — no `PartialEq`, no `Default` — so an unavailable outcome cannot compare equal to a completed one.

**Also verified: the crate still exports nothing.** `grep -c "^pub " crates/tiler-conformance/src/lib.rs` returns **0**. That is the boundary you must not move.

**The thing that looks like a defect and is not — read this before deciding to retire the switch.** Nothing in `crates/` or the `Makefile` ever *sets* `TILER_REQUIRE_METAL_CONFORMANCE`, so under `make full` the `assert!` always holds and the unavailable path prints its notice and passes. That reads exactly like the unfireable check AGENTS.md warns about, and I nearly briefed it as one. **It is deliberate.** The switch is opt-in hardening a human applies by hand, and it has been watched firing in both directions with quoted output — see [`produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate`](produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate.md), which records `env -i PATH=/var/empty … TILER_REQUIRE_METAL_CONFORMANCE=1` producing `TILER_REQUIRE_METAL_CONFORMANCE is set and the measured half is unavailable: no qualified Apple Metal…`, and passing with `PATH` emptied when the variable is unset. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) records the same pair independently.

So the Required work's "retire or retain" question is genuinely open, but **retiring it must preserve that hardening capability or argue explicitly why it is no longer needed** — the ability to make an unavailable host a hard failure is a property two landed tickets paid for and watched. A call-site policy can supply it; an omission cannot. Do not treat the absence of a setter in `make full` as evidence the switch is dead.

**On the perturbations.** The two properties really are independent, as the ticket says: no-equality-with-pass is a type property of the outcome, and no-ambient-read is a property of where policy is decided. Perturb each separately and quote both failures; a perturbation that reddens both cannot show which is load-bearing.
