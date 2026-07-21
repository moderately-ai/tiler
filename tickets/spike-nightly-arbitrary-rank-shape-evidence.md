---
id: spike-nightly-arbitrary-rank-shape-evidence
title: Spike nightly arbitrary-rank static shape evidence
status: todo
priority: p0
dependencies: [research-nightly-const-shape-parameters]
related: [prototype-shaped-value-api]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [spike, rust-api, const-generics, shapes]
---
# Spike nightly arbitrary-rank static shape evidence

## Goal

Test whether one pinned nightly Rust type can carry canonical exact extents of
arbitrary rank as optional, graph-checked evidence without public per-rank
families or downstream descriptor types.

## Candidate surface

Prioritize the dependent-array form identified by the research:

```rust,ignore
pub struct StaticShape<const RANK: usize, const DIMS: [u64; RANK]>;

type Matrix = ShapedValue<F32, StaticShape<2, { [2, 3] }>>;
```

Retain `StaticShape<const DIMS: &'static [u64]>` as a comparison case because
Rust's own implementation notes allow references in const generics to be
forbidden later. The spike may adjust surface syntax as the feature evolves,
but it must preserve one value-based canonical type identity across ranks.

## Required experiments

- pin an exact dated nightly in the isolated spike and record every feature
  gate and compiler commit;
- distinguish behavior that needs `min_adt_const_params`,
  `generic_const_parameter_types`, and `unsized_const_params`, and prove the
  selected form does not enable an unnecessary feature;
- prove ranks 0, 1, representative tensor ranks, and a high rank compile through
  the same public type family;
- prove equal literal shapes written in separate modules and separate crates
  unify as the same Rust type, while unequal shapes fail at compilation;
- test private/public constants, reexports, aliases, and macro-produced const
  arguments for identity equivalence;
- retain positive and negative diagnostics for refinement, operation
  composition, and type mismatch;
- confirm only the graph constructs refined handles and canonical graph facts,
  rather than Rust const identity, determine semantic and artifact identity;
- exercise rustdoc, clippy, incremental and clean builds, proc-macro-compatible
  generated call sites, symbol generation, and at least 1,000 distinct shapes;
- probe the next available nightly separately to expose compiler-pin sensitivity
  without silently updating the governed pin; and
- catalogue compiler errors, ICEs, feature interactions, and syntax changes as
  evidence rather than working around them invisibly.

## Acceptance gates

Adopt the nightly form only if it demonstrates canonical cross-crate identity,
arbitrary rank through one family, no unsafe authority leak, acceptable
diagnostics and bounded compile cost, and a feature premise aligned with Rust's
documented direction. A dated nightly pin and incomplete-feature allowance must
be explicit repository policy, not an incidental developer setup.

Reject or defer it if identity depends on reference allocation, compiler
behavior contradicts the intended structural-value model, routine use triggers
an ICE, the required feature is explicitly headed toward removal, or a compiler
upgrade would alter Tiler semantic or artifact identity. Failure falls back to
the already measured stable-Rust `StaticShapeN` families; it does not block
canonical graph shapes or rank-only evidence.

## Deliverables

- retained source, cross-crate fixtures, compile-pass/fail cases, measurement
  harness, and compact results under `spikes/shapes/`;
- a research update that distinguishes premise, implementation, and measured
  compiler revision;
- an ADR update accepting either the pinned-nightly form or the stable fallback;
  and
- corresponding MSRV/toolchain-policy and implementation-ticket updates.
