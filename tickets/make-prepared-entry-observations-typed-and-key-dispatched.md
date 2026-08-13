---
id: make-prepared-entry-observations-typed-and-key-dispatched
title: Make prepared-entry observations typed and key-dispatched
status: done
priority: p1
dependencies: []
related: [decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality, accept-the-prepared-entry-observation-surface]
scopes: [implementation/runtime, implementation/candle, implementation/conformance, contracts/artifacts, contracts/decisions, implementation/frontend, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, preflight, public-boundary, correctness, fail-closed]
---
## User-visible outcome

A runtime adapter can report that it does not recognize a prepared-entry property, and cannot accidentally answer one property with an unrelated pipeline quantity.

## Fact — 2026-08-11

`RuntimeAdapter::observe_prepared_entry` returns a bare `u64`. The Candle Metal implementation reads neither the request's property key nor its provider identity and returns `maxTotalThreadsPerThreadgroup` for every request. This is harmless only while exactly one query exists; a second legal key creates a false-admission path when the unrelated quantity equals the required value.

## Fact audit — 2026-08-13 at `b0aa7d6e`

- **Verified.** `RuntimeAdapter::observe_prepared_entry` returned `-> u64`. Anchor: `fn observe_prepared_entry` in `crates/tiler-runtime/src/adapter.rs`.
- **Verified.** Candle Metal ignored the request key/provider and returned `max_total_threads_per_threadgroup` for every request. Anchor: `fn observe_prepared_entry` in `prototypes/candle-metal-adapter/src/adapter.rs`.
- **Census at this base.** Eleven `RuntimeAdapter` implementors: the trait, Candle Metal, the scalar-host adapter-route fixture, identity-join, three facade tests, `crates/tiler/src/route/tests.rs`, inline-dispatch, and the backend-provider-portfolio Metal and CPU adapters. Eight `resolve_target_properties` closures answered a bare `u64` (serial-sum-run, conformance envelope/apple). Every one now returns `PreparedEntryObservation`.

The 2026-08-11 Fact described the defect at that base. This branch replaces the bare number with `PreparedEntryObservation::{Quantity(u64), Unrecognized}` and exact-matches provider namespace/name/revision and property key before reading a quantity.

## Required delivery

- Replace the bare number with an exhaustive typed observation carrying `Quantity(u64)` or `Unrecognized`; no numeric sentinel, `Option` default, or catch-all success.
- Make every adapter exact-match provider namespace/name/revision and property key before reading a quantity. Unknown ownership returns `Unrecognized` and the loader classifies it separately from an observed mismatch.
- Preserve loader-owned comparison. An adapter reports an observation and never returns its own satisfaction verdict.
- Include property key, provider, entry, required/observed value, and relation in the typed diagnostic where the repository's error-bound permits them.
- Perturb provider, key, result variant, value, and entry independently. Show that the old wrong-property equality coincidence now refuses.
- Rebaseline the public adapter surface deliberately; do not retain a compatibility method that maps unknown properties to a number.

## Closes when

Every prepared-property consumer is exhaustive over the typed observation, an unknown property cannot be confused with a quantity, and the old wrong-property false admission has a failing-then-green regression.

## Outcome

Landed at `b19bc60dd7a6f6bd35d880472885b62b2bf374d1`, merged to `main` at `0742e22dc5e3cc1e24b017f4bc2d4b0f0fde9c03`. `make full` on that hash: 3456 passed. The public observation surface is a labelled draft; Tom's packet is [`accept-the-prepared-entry-observation-surface`](accept-the-prepared-entry-observation-surface.md).
