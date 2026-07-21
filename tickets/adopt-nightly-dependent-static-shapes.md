---
id: adopt-nightly-dependent-static-shapes
title: Adopt pinned-nightly dependent static shapes
status: done
priority: p0
dependencies: [research-nightly-const-shape-parameters]
related: [spike-nightly-arbitrary-rank-shape-evidence, prototype-shaped-value-api]
scopes: [research/shapes, contracts/decisions, contracts/foundation, contracts/integrations, research/workspace]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [decision, rust-api, const-generics, shapes]
---
# Adopt pinned-nightly dependent static shapes

## Goal

Record Tom's acceptance of one arbitrary-rank dependent-array shape-evidence
family on a pinned nightly, supersede the stable-only prototype toolchain
decision, and convert the nightly spike from a product-choice gate into the
required implementation and upgrade conformance harness.

## Work

- add an accepted ADR selecting `StaticShape<const RANK: usize, const EXTENTS:
  [u64; RANK]>`, the minimum feature gates, and an exact dated nightly policy;
- preserve graph revalidation, explicit weakening, and exclusion of Rust
  evidence from semantic, artifact, and cache identity;
- supersede the Rust 1.89 stable-only workspace decision without discarding its
  valid cache-locking evidence;
- update the IR, architecture, frontend, research, status, roadmap, and open-
  question contracts; and
- make the retained spike validate the accepted form and every future compiler-
  pin migration rather than choose between public API alternatives.

## Acceptance

The durable documents unambiguously select one arbitrary-rank family, name only
the required unstable features, distinguish the conceptual API contract from
changeable nightly syntax, define the compiler-pin upgrade protocol, retain a
failure path that requires an explicit superseding decision, and pass all
documentation and ticket validation.

## Outcome

- ADR 0067 accepts `StaticShape<const RANK: usize, const EXTENTS: [u64;
  RANK]>` on the exact `nightly-2026-07-19` compiler pin.
- The contract enables only `min_adt_const_params` and
  `generic_const_parameter_types`, localizes nightly syntax churn, and keeps
  Rust evidence outside semantic and artifact identity.
- ADR 0057's stable-only toolchain policy is superseded without discarding its
  valid Rust 1.89 advisory-locking evidence.
- The retained nightly spike is now the implementation and future compiler-pin
  migration harness. It owns the actual workspace, CI, and shaped-value pin
  changes after conformance passes.
