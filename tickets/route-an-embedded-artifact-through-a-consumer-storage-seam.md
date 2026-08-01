---
id: route-an-embedded-artifact-through-a-consumer-storage-seam
title: Dispatch an embedded artifact through a consumer storage seam
status: in-progress
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/frontend, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, runtime, public-boundary]
claimed_from: todo
assignee: worker-route-an-emb
lease_expires_at: 1785568847
---
## Why this exists

`prototype-inline-aot-integration-proof` landed everything an inline region needs up to the routing commit and stopped one step short of a running kernel. This ticket is that step, and what blocks it is a *missing accepted boundary* rather than unwritten code.

**Fact.** `crates/tiler/src/value.rs` publishes no storage access — "Nothing here yields a pointer, a buffer, a byte slice, or a device object" — and states why: "a storage-access surface would be a public boundary with no caller to review it against". A `tiler`-only consumer therefore has nothing to hand a kernel.

**Fact.** `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend` forbids any workspace package from depending on `tiler` or `tiler-macros`, so no in-tree crate can be the consumer that dispatches. The only consumers are the out-of-tree `trybuild` fixtures, which see `tiler`'s dependency list and nothing else.

**Inference.** `tiler_runtime::load::Preflight::commit` is unreachable from any consumer of the facade today, which is why `crates/tiler/src/route.rs` stops at `RouteOutcome::NoDeviceAuthority` and contains no call to `commit` at all.

**Fact.** ADR 0086 refuses on every macOS row, so even with a seam the dispatching path is `prototypes/serial-sum-run`'s producer-declared equality under its labelled diagnostic, never host-earned eligibility.

## User-visible outcome

One inline invocation in an ordinary crate produces a running kernel: the embedded artifact is routed, committed, and dispatched against the consumer's own values, with the fallback still taken before the commit and nowhere after it.

## Closes when

- The storage seam a dispatch needs is designed, put to Tom as a public boundary under ADR 0075, and accepted or refused with a recorded reason. `AdapterCapability::DenseRowMajorStorage` is the reservation it fills.
- `crates/tiler/src/route.rs` gains a committed outcome, and `RouteOutcome::is_fallback` stops reading as a constant.
- The run is recorded with the same labelled producer-declared-equality diagnostic `prototypes/serial-sum-run/src/proof.rs` prints, and says in those words that ADR 0086 refused the host.
- A correctness oracle compares the dispatched result against the semantic fallback's before any performance claim is made.
