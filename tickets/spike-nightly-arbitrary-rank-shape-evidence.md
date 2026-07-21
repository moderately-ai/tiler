---
id: spike-nightly-arbitrary-rank-shape-evidence
title: Spike nightly arbitrary-rank static shape evidence
status: done
priority: p0
dependencies: [research-nightly-const-shape-parameters, adopt-nightly-dependent-static-shapes]
related: [prototype-shaped-value-api]
scopes: [research/shapes, implementation/workspace, implementation/ir, implementation/reference, implementation/artifact, implementation/compiler, implementation/metal, implementation/runtime, contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: [AGENTS.md]
tags: [spike, rust-api, const-generics, shapes]
---
# Spike nightly arbitrary-rank static shape evidence

## Goal

Implement the retained conformance and compiler-upgrade harness for ADR 0067's
accepted pinned-nightly, arbitrary-rank dependent-array shape evidence.

## Candidate surface

Prioritize the dependent-array form identified by the research:

```rust,ignore
pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;

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

The harness passes when the accepted dependent-array form demonstrates
canonical cross-crate identity, arbitrary rank through one family, no authority
leak, retained diagnostics, proc-macro-compatible generation, and bounded
compile cost on the exact governed pin. The dated nightly and incomplete-
feature allowance must be explicit repository policy, not incidental developer
setup.

A failure blocks the shaped-value implementation or compiler-pin migration and
reports the exact violated invariant. It does not silently select borrowed
slices, stable `StaticShapeN` families, or weaker evidence; changing the
accepted form requires an explicit superseding decision.

## Deliverables

- retained source, cross-crate fixtures, compile-pass/fail cases, measurement
  harness, and compact results under `spikes/shapes/`;
- a research update that distinguishes premise, implementation, and measured
  compiler revision;
- an ADR 0067 implementation-status update after the governed pin passes; and
- corresponding toolchain-policy, CI, and implementation-ticket updates.

## Outcome

- Retained a six-crate isolated workspace covering the selected evidence type,
  two independent alias providers, stable proc-macro generation, compile and
  runtime conformance, and generated compile-cost workloads.
- Proved exact structural identity across literals, constants, reexports,
  crates, and generated tokens; representative ranks through 64; private
  refinement authority; evidence-neutral identity; and expected failure for
  unequal shapes, rank mismatch, forgery, and downstream evidence claims.
- Proved that only `min_adt_const_params` and
  `generic_const_parameter_types` are required. Borrowed slices remain an
  isolated comparison behind nonselected features.
- Passed the full suite unchanged on `nightly-2026-07-19` (`eff8269f7`) and
  adjacent `nightly-2026-07-20` (`9f36de775`).
- Measured 1/10/100/1,000 shapes with exact provenance. The governed 1,000-shape
  clean check took 0.132 seconds at 86.2 MiB peak RSS; release took 0.240
  seconds, added 16 bytes over one shape, and retained the same 331 global
  symbols.
- Migrated contributor bootstrap, CI, validation, and the workspace to the
  exact governed pin and removed the misleading stable `rust-version` claim.
