---
id: correct-the-runtime-execution-environment-fact-census
title: Correct the runtime execution-environment fact census
status: in-progress
priority: p2
dependencies: []
related: [decide-the-adr-0013-plan-determinism-stability-subject]
scopes: [implementation/runtime, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, graph-repair]
claimed_from: todo
assignee: worker-runtime-environment-docs
lease_expires_at: 1787024422
---
## User-visible outcome

The runtime glossary and the loader's own module documentation describe the complete current device-free host declaration: target profile, backend, representation, and dtype-dispatchability evidence. They no longer call this four-field surface a three-fact declaration.

## Exact-base Fact audit — 2026-08-17 at `a01e78b7c99ea8ee00a7e2e58894094587da9def`

- **False — glossary closed census.** `docs/glossary.md`, anchor `Execution environment (host declaration)`, says `ExecutionEnvironment` contains exactly one target profile, backend, and representation “and nothing else.” The complete file was read. Current `crates/tiler-runtime/src/load/host.rs::ExecutionEnvironment` also has public `dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>`.
- **False — runtime module census.** `crates/tiler-runtime/src/load/host.rs`, anchor `the host supplies the three facts a load depends`, omits the same dtype-dispatch declaration. The complete file was read; its later sections already explain `declare_dtype_dispatchability` and use the map in `classify_dtype`.
- **Verified — accepted authority.** `tickets/accept-the-route-facts-dtype-dispatch-field.md` and `tickets/validate-bf16-at-the-runtime-routing-boundary.md`, read in full, record Tom's accepted `ExecutionEnvironment.dtype_dispatch` surface and the fail-closed meaning of an absent row. `tickets/declare-host-dtype-dispatchability-at-the-consumer-boundary.md`, read in full, records the producer-declared route rows and their ownership.
- **Verified — boundary remains narrow.** Dtype dispatch is variant eligibility evidence, not a new live-device attestation or the ADR 0013 runtime-compatibility identity. The current comments already distinguish the host statement from a device, queue, library, pipeline, and command encoder.

These corrections do not change the ticket purpose or any public, behavioral, identity, schema, or maturity boundary.

## Implementation boundary

Edit only the glossary row, the runtime module prose, and this ticket's durable evidence. Preserve the distinction between dtype eligibility and runtime compatibility/attestation. Do not change `ExecutionEnvironment`, routing behavior, accepted ADRs, or the pending ADR 0013 decision.

## Closes when

Both current-truth descriptions enumerate all four declared facts, neither says “three” or “and nothing else” while omitting dtype dispatch, rustdoc remains warning-free, and the exact-base ticket/citation/scope gates pass.

## Outcome — 2026-08-17

Delivered over exact base `a3d09993e37c1f16adb2aec7ec3edb3ce56f0df9`. The glossary row and the runtime module's device-free-loading rationale now enumerate target profile, backend, representation, and per-arithmetic-type dtype-dispatch verdicts. Both retain the existing boundary: these are caller declarations used for load and variant eligibility, not live-device attestation; the dtype map does not become the pending ADR 0013 runtime-compatibility identity.

No Rust item, behavior, identity, schema, accepted ADR, or maturity claim changed. The source check rejected both retired incomplete phrases and found the three replacement anchors:

```text
rg -n 'the host supplies the three facts a load depends|one `RepresentationKey` — and nothing else' crates/tiler-runtime/src/load/host.rs docs/glossary.md
rg -n 'four declarations a load|one map from each declared `ArithmeticType`|dtype map is eligibility evidence' crates/tiler-runtime/src/load/host.rs docs/glossary.md
```

The first command returned no matches. The second returned the runtime module declaration and the corrected glossary row. `cargo fmt --check -- crates/tiler-runtime/src/load/host.rs`, `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-runtime --no-deps`, `tkt lint --format json`, `make citations`, and `git diff --check` passed. The post-commit exact-base `tkt guard` result is recorded with the commit handoff.
