---
id: research-nightly-const-shape-parameters
title: Research nightly arbitrary-rank const shape parameters
status: done
priority: p0
dependencies: [research-the-public-static-shape-evidence-spelling]
related: [prototype-shaped-value-api]
scopes: [research/shapes]
shared_scopes: [project/tickets, contracts/decisions, contracts/navigation]
paths: []
tags: [tiler-research, rust-api, const-generics, shapes]
---
# Research nightly arbitrary-rank const shape parameters

## Goal

Determine whether Rust's unstable const-parameter roadmap is semantically the
facility Tiler wants for canonical, arbitrary-rank static shape evidence, and
separate that premise from the maturity of any current nightly implementation.

## Work

- trace `adt_const_params`, `unsized_const_params`, const-parameter structural
  equality, and the newer generic-const-argument work through official feature
  documentation, tracking issues, RFCs, compiler guidance, and tests;
- enumerate the viable public forms, including fixed arrays, dependent arrays,
  borrowed slices, Tiler-owned structural shape values, and type-const aliases;
- determine which forms provide value-based cross-crate type identity, arbitrary
  rank, usable diagnostics, stable symbol identity, and a plausible
  stabilization path;
- distinguish intended language semantics, implemented nightly behavior,
  unresolved design questions, and compiler defects; and
- update the retained public-shape-spelling research without accepting an API
  before the evidence is reviewed.

## Acceptance

The report links primary Rust sources, states the purpose and stabilization
direction of each required feature, records exact compiler revisions for local
probes, compares the forms point-by-point, identifies any premise-level mismatch
with Tiler, and ends with a recommendation plus a bounded spike specification.

## Outcome

The feature premise is aligned with Tiler. A dependent extent array combines
the intended array const-parameter and generic-dependent parameter-type
capabilities without reference identity or per-rank families. The borrowed
slice is rejected as the leading form because Rust explicitly preserves the
option to forbid references in const generics. Compiler readiness remains open
and is isolated in `spike-nightly-arbitrary-rank-shape-evidence`.
