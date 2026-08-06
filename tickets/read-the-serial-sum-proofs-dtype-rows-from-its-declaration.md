---
id: read-the-serial-sum-proofs-dtype-rows-from-its-declaration
title: Read the serial-sum runtime proof's dtype rows from the declaration it already holds
status: in-progress
priority: p2
dependencies: []
related: [declare-host-dtype-dispatchability-at-the-consumer-boundary, validate-bf16-at-the-runtime-routing-boundary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, runtime, routing, authority]
claimed_from: todo
assignee: agent-serial-rows
lease_expires_at: 1786048130
---
## User-visible outcome

`prototypes/serial-sum-run`'s `declared_route_environment` states its dtype-dispatchability rows by reading `BoundMetalCompileDeclaration::dtype_dispatchability_rows` — the declaration it already takes as a parameter — instead of transcribing `tiler-build`'s `FIRST_MACOS_APPLE9` ledger rows into a call-site literal that a retracted or refuted measurement would leave stale.

## Why this is the last transcription of its kind

**Fact.** `declare-host-dtype-dispatchability-at-the-consumer-boundary` added the accessor and converted the two consumer sites in its own scopes: `crates/tiler/src/route.rs` now restates an emitted `RouteFacts` row, and `prototypes/candle-metal-adapter/src/proof.rs` now reads the accessor. This site was left because it is `implementation/runtime`, which that ticket did not hold, and editing it during that branch would have been a scope escape.

**Fact.** The literal at `prototypes/serial-sum-run/src/proof.rs:1097` states `(F32, Dispatchable)` and `(Bf16, Dispatchable)` with a comment recording that they are the ledger's rows transcribed rather than observed. That comment is correct today and is the only thing keeping the two in agreement.

**Inference.** The accessor makes the agreement structural rather than remembered: it answers from the same `TargetProfile` the compile gate consults, so a ledger row that moves moves this environment with it. Nothing else about the site changes — the rows stay producer-declared, and the binary keeps printing the producer-declared-equality label beside every outcome.

## Implementation keys

- Read the rows from the `declaration` parameter this function already takes, mapping `tiler_build::DTypeDispatchability` into `tiler_runtime::load::DTypeDispatch` through an exhaustive match, exactly as `prototypes/candle-metal-adapter` does.
- Keep silence fail-closed: the accessor omits a dtype the profile resolves `Unknown` or `Deferred`, so a dtype nothing measured produces no row and the loader refuses it.
- Update the comment in place to say what the rows now are and what authority gap survives — they remain producer-declared, and this binary asks its device nothing about either dtype.
- Do not widen the proof's claims. Nothing here dispatches a BF16 kernel and nothing here earns a host row; ADR 0086 still refuses the applicability receipt.

## Required evidence

- The run's reported behaviour is unchanged for the rows the ledger currently declares.
- A perturbation of the ledger's `bf16_dispatchability` reaches this environment, so the read is load-bearing rather than decorative.

## Closes when

The literal is gone, the rows come from the declaration, and the surviving authority gap is stated where the rows are read.
