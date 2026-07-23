---
schema: "tiler-doc/v1"
id: "ADR-0067"
kind: "decision"
title: "Use pinned-nightly dependent arrays for exact shape evidence"
topics: ["rust", "toolchains", "shapes", "const-generics", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.frontend-integration"]
evidence: ["tiler.research.shapes.nightly-const-shape-parameters", "tiler.research.shapes.public-static-shape-spelling", "tiler.research.shapes.stable-rust-shape-evidence"]
refines: ["ADR-0061"]
supersedes: ["ADR-0057"]
ticket: "adopt-nightly-dependent-static-shapes"
---

# 0067: Use pinned-nightly dependent arrays for exact shape evidence

**Status:** accepted; conformance harness, workspace pin, and bounded shaped-
value API implemented; public-interface review pending

## Context

ADR 0061 accepts optional graph-checked Rust shape evidence while leaving its
exact public spelling unresolved. The stable-Rust comparison found sealed
`StaticShapeN` arity families to be the best stable option, but every published
family covers only one exact-evidence rank.

Rust's intended const-generics model is a closer architectural match: constant
values participate structurally in type identity. The July 2026 nightly accepts
an array const parameter whose length depends on an earlier rank parameter. Tom
accepts the cost of an exact nightly pin and source evolution while these
features mature in exchange for one canonical arbitrary-rank family.

## Decision

The initial exact static-shape evidence has the conceptual public form:

```rust,ignore
pub struct StaticShape<
    const RANK: usize,
    const EXTENTS: [u64; RANK],
>;

type Matrix = ShapedValue<F32, StaticShape<2, { [2, 3] }>>;
```

`RANK` uses `usize` because Rust array lengths do. Extents use `u64` so their
evidence spelling is independent of host pointer width and agrees with Tiler's
canonical extent domain. Admission converts both into the corresponding
semantic newtypes and checks them against the value's authoritative graph
shape. A mismatched rank and array length is not representable as a Rust type.

The implementation enables only:

```rust,ignore
#![feature(min_adt_const_params)]
#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]
```

It does not enable `unsized_const_params`, use `&'static [u64]` const
parameters, expose downstream static-shape descriptors, or require the broader
`adt_const_params`, `generic_const_args`, or `generic_const_exprs` features.
Those capabilities require separate evidence and decisions.

The initial governed compiler pin is `nightly-2026-07-19`, observed as
`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`. `rust-toolchain.toml`, not
Cargo's stable-version `rust-version` field, is the authoritative channel and
revision contract. Workspace implementation removes the claim that these
nightly-only crates support a stable MSRV. Rust 2024 remains the edition.

The durable API invariant is “one structural exact-extent-vector evidence type
for every representable rank,” not the current feature names, punctuation, or
parameter ordering. Nightly-driven syntax changes may revise the Rust spelling
without changing canonical graph shape, refinement authority, or artifact
identity. A type-position `static_shape![2, 3]` convenience may be added after
call-site evidence, but its expansion is the same nominal `StaticShape` type and
the explicit form remains documented.

Every compiler-pin change is an explicit migration. The candidate compiler is
tested alongside the governed pin using the retained conformance harness;
feature changes, diagnostics, type identity, symbols, rustdoc, proc-macro call
sites, and compile cost are reviewed before `rust-toolchain.toml` changes. A
rolling `nightly` channel is forbidden.

The pinned nightly requirement applies to Rust consumers compiling Tiler or an
inline Tiler proc macro. It does not authorize unstable procedural-macro APIs:
the accepted inline expansion, external-input tracking, and consumer DX
contracts continue to rely only on stable proc-macro capabilities unless a
separate ADR changes them.

The conformance spike is an implementation and upgrade gate, not a reopened
product choice. A failed compiler conformance blocks the shaped-value
implementation or pin migration and requires an explicit superseding decision;
it does not silently select the stable arity-family fallback.

The retained harness passed on the governed compiler and the immediately
adjacent `nightly-2026-07-20` compiler. It established cross-crate structural
identity, exact feature requirements, stable-proc-macro token generation,
rank-zero through rank-64 coverage, compile-fail diagnostics, authority
isolation, and bounded 1,000-shape compile cost. The repository now uses the
governed pin and declares no stable `rust-version`. Implementing the production
`tiler-ir` now implements the selected family as sealed evidence checked by
builder- and program-owned refinement. The implementation remains an
experimental public-interface review draft rather than a stabilized API.

## Consequences

- Exact static evidence supports arbitrary rank through one canonical family.
- Equivalent extent arrays no longer depend on which crate named a descriptor.
- Tiler's prototype and any Rust consumer require the exact governed nightly.
- An unavailable or incompatible nightly fails at build time; Tiler does not
  weaken evidence or fall back to another public type spelling automatically.
- The feature-dependent module and conformance harness localize syntax churn,
  but a future compiler migration may still require a source-breaking release.
- Rank-only `ShapedValue<T, Rank<R>>` and unrefined `Value<T>` remain available
  for symbolic, runtime-dependent, or intentionally weaker Rust evidence.
- Rust evidence remains non-authoritative, explicitly weakenable, checked on
  refinement, and excluded from semantic, compilation, artifact, and cache
  identity.

## Alternatives considered

Stable `StaticShapeN` families avoid nightly but publish a finite exact-evidence
rank vocabulary. A borrowed-slice const parameter is shorter but relies on
reference identity behavior Rust may remove. A padded structural value replaces
per-rank families with a maximum rank and admits noncanonical encodings. A
recursive type list creates a second type-level shape algebra. Downstream
descriptors are sound after checking but give identical extents different Rust
types across crates.
