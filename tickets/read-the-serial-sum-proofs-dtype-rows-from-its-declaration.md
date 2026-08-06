---
id: read-the-serial-sum-proofs-dtype-rows-from-its-declaration
title: Read the serial-sum runtime proof's dtype rows from the declaration it already holds
status: done
priority: p2
dependencies: []
related: [declare-host-dtype-dispatchability-at-the-consumer-boundary, validate-bf16-at-the-runtime-routing-boundary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, runtime, routing, authority]
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

## Outcome — 2026-08-06

**The literal is gone and the rows are read; the authority gap it stood in front of is unchanged and is now stated where the read happens.** `prototypes/serial-sum-run/src/proof.rs::declared_route_environment` builds `dtype_dispatch` from `declaration.dtype_dispatchability_rows()`, mapping each verdict through a new wildcard-free `host_dtype_dispatch`, exactly as `prototypes/candle-metal-adapter` does. This was the last transcription of its kind: no `ArithmeticType`/`DTypeDispatch` pair is written at a call site in this binary any more, and the `std::collections::BTreeMap` and `ArithmeticType` imports the literal needed are gone with it.

**Fact — what landed.** One file, `prototypes/serial-sum-run/src/proof.rs`, +131/-39.

- `declared_route_environment` reads the rows off the `declaration` parameter it already took. Silence stays fail-closed by construction rather than by rule: the accessor omits a dtype the profile resolves `Unknown` or `Deferred`, so `f16` — which this ledger deliberately does not measure — produces no row, and `ExecutionEnvironment::classify_dtype` resolves the absent key `Unknown`, which refuses.
- `host_dtype_dispatch(DTypeDispatchability) -> DTypeDispatch` is an exhaustive `const fn`. It is duplicated from the Candle prototype rather than shared, because it cannot be shared: `tiler-build` states what a *profile* declares and `tiler-runtime` states what a *host* offers, neither depends on the other, and both prototypes are `[[bin]]`-only packages that are not linkable from each other. `tiler_build` re-exports `DTypeDispatchability` for exactly this, and that is the import this file gained.
- The comment is rewritten in place, not appended to. The function's heading now carries **The dtype rows are read from the declaration, and the gap that leaves**: a moved ledger measurement moves this environment with it, and what that removes is a second copy of the rows and *not* the authority gap — they remain producer-declared, this binary holds a real `MTLDevice` and asks it nothing about either dtype, and [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) keeps the applicability receipt out of reach on every macOS row currently observable, so no observation this binary could take would make the profile this host's to offer.

**Fact — the regression test.** `proof::tests::the_routed_dtype_rows_are_the_declarations_own` derives its expectation from the declaration rather than from today's values, because a literal and a read agree exactly while `FIRST_MACOS_APPLE9` states what it states today — no assertion over today's *values* could tell them apart. It additionally pins both arms of the verdict translation, and pins `classify_dtype(F16) == Unknown` as the fail-closed half.

**Measurement — the perturbation matrix, watched failing.** `FIRST_MACOS_APPLE9.bf16_dispatchability` in `crates/tiler-build/src/metal_declaration.rs` was perturbed two ways, against both spellings of the call site, and the observed `dtype_dispatch` map was printed each time. All five cells ran with `cargo nextest run -p tiler-prototype-run --locked --no-capture -E 'test(the_routed_dtype_rows_are_the_declarations_own)'`; the perturbation was reverted afterwards and the revert verified by `diff` against a pre-perturbation copy.

| Ledger row | Call site | Observed rows | Test |
| --- | --- | --- | --- |
| `Some(Dispatchable)` (as published) | read | `{Bf16: Dispatchable, F32: Dispatchable}` | pass |
| `Some(Unsupported)` — refuted | read | `{Bf16: Unsupported, F32: Dispatchable}` | pass |
| `Some(Unsupported)` — refuted | old literal | `{Bf16: Dispatchable, F32: Dispatchable}` | **FAIL**, `left: {Bf16: Dispatchable, …} right: {Bf16: Unsupported, …}` |
| `None` — retracted | old literal | `{Bf16: Dispatchable, F32: Dispatchable}` | **FAIL**, `left` carries a `Bf16` row, `right: {F32: Dispatchable}` |
| `None` — retracted | read | `{F32: Dispatchable}` | pass |

The first row is also the unchanged-behaviour evidence at the source level: the read produces the identical two-entry map the literal spelled.

**Measurement — the run's reported behaviour is unchanged, byte for byte.** Apple M4 Max, macOS 27.0 build `26A5388g`, Metal compiler `32023.921`, 2026-08-06. `cargo run -q -p tiler-prototype-compile --locked -- --out <scratch>/serial-sum.tiler` published 8 members; `cargo run -q -p tiler-prototype-run --locked -- --artifact <scratch>/serial-sum.tiler` was then run twice against those same bytes — once at base `b3d5a9ed` and once at this change — and the two logs are identical (`diff` exit 0), both exit 0. The run reports: the ADR 0086 refusal before any routing commit, three parallel alternatives agreeing bit for bit and each held to its own declared grouping, 30 cases across 6 reduction members, and 6 contraction cases including the retained `w_decode_kv` digest `79810ce4…8701f`. The offline toolchain on this host is `32023.921` where the ledger's measured row is `32023.883`, so this run is a behaviour comparison between two builds on one host and not a re-measurement of the ledger.

**The surviving authority gap, stated as the ticket requires.** `prototypes/serial-sum-run/src/proof.rs::declared_route_environment` now reads the declaration instead of transcribing it, and **the rows stay producer-declared**. A host-earned row would need a per-dtype observation on the `MTLDevice` this binary already holds; ADR 0086 refuses the applicability receipt on every macOS row currently observable, so no observation this binary could take would make the profile this host's to offer. That is a decision to record, not a task to schedule, and it is recorded in the function's own heading as well as here.

**What this did not do.** It moved no support-matrix row and widened no claim. Nothing here dispatches a BF16 kernel, nothing here earns a host row, and ADR 0086 still refuses the applicability receipt. No public boundary changed: `host_dtype_dispatch` is a private `fn` in a `[[bin]]`-only prototype.

**Commands.** `cargo fmt --all --check`; `git diff --check`; `cargo check --workspace --all-targets --locked`; `cargo clippy --workspace --all-targets --locked --exclude tiler-prototype-run --exclude tiler-prototype-compile --exclude tiler-prototype-candle -- -D warnings` (the Makefile's `lint` target — `prototypes/` is excluded from the style gate by design); `cargo nextest run --workspace --locked` → 2862 passed, 7 skipped; `cargo test --workspace --doc --locked --quiet` → all green; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked -p tiler-prototype-run`; `tkt lint` → ok.
