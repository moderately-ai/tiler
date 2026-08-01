---
id: register-opaque-calls-on-the-compile-path
title: Register opaque calls on the compile path
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [physical-planning, extensions, implementation]
---
## User-visible outcome

A declared and registered opaque physical call can reach `enumerate_frontier` through `session::compile`, so the opaque-call admission path that already exists is reachable by some caller rather than by none.

## Why this slice exists

**Fact, at `e6a47d9`.** `implement-opaque-physical-call-providers` built the declaration, ABI, effect, aliasing, placement, failure-stage, and coherence contracts, and `integrate-opaque-calls-into-the-physical-frontier` made `enumerate_frontier` admit a registered call. `enumerate_frontier` takes an `&OpaqueCallRegistry`, and the sole production call site constructs an empty one inline: `crates/tiler-compiler/src/pipeline/planning.rs:228` passes `&crate::call_registry::OpaqueCallRegistry::new()`.

Reproduce with `grep -rn "OpaqueCallRegistry" crates/`. Every hit other than the definition module and the `use`/parameter in `frontier.rs` is inside a `#[cfg(test)]` module: `selection.rs` after its `#[cfg(test)]` at line 1689, `frontier.rs`'s own test module, and `pipeline/tests.rs`. The positive control for that reading is `GovernedPhysicalProvider`, which the same style of grep finds at a production site, `planning.rs:171`.

**Inference.** No opaque call can reach a compilation, so the admission path is implemented support behind an authority nothing populates, and a test asserting the frontier admits a registered call proves nothing about `compile()`.

This is a narrower statement than ADR 0078's 2026-07-31 correction, which records that opaque declaration and registration are compiler-owned and crate-private so that no *out-of-crate* provider registers a call. That classification is unaffected and is not reopened here: the gap is that no caller of any kind registers one.

## Implementation keys

- Decide where a compile-path registry is composed and by which authority; the call registry stays crate-private, so this is an internal wiring question and not a public seam.
- Keep an empty registry a legitimate state. A compilation with no declared call must behave exactly as it does now.
- Add one compile-path test that a registered call reaches an admitted alternative through `session::compile`, and one that an unregistered call named by a proposal is `FrontierRejection::OpaqueCall` with cause `Unregistered`.
- Run each new check against a case that must fail before relying on it.

## Closes when

A registered opaque call is admitted through `session::compile` by a test that fails when the registration is removed, the empty-registry path is unchanged, and targeted package checks pass.

## Graph maintenance

- Record the outcome against ADR 0078's opaque-call correction if the classification is affected; it should not be.
