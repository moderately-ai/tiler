---
id: qualify-lowering-registry-pooling-measurement-as-historical
title: Qualify the lowering-registry pooling measurement as historical
status: todo
priority: p3
dependencies: []
related: [intern-the-lowering-registry-s-shared-authority-identities, correct-the-slice-normative-definition-and-recompute-compiler-identities]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, measurement]
---

`crates/tiler-compiler/src/capability.rs`, anchor `Why a pool and not an inline
copy`, still gives the original five-capability measurement as the rationale
for interning. That measurement is useful historical evidence, but the governed
population and current byte sizes have since moved enough that an undated
reader can mistake it for the live profile.

## Per-Fact audit — 2026-08-09

- **Verified historical Fact.** The comment under source anchor `Why a pool and not an inline copy` records five governed capabilities, a 1,496-byte semantic
  registry snapshot repeated five times, and a 15,583-byte lowering-registry
  identity.
- **Verified historical Fact.** [`intern-the-lowering-registry-s-shared-authority-identities`](intern-the-lowering-registry-s-shared-authority-identities.md)
  records those figures as the **before** measurement that justified the v2
  pooled encoding; they should not be silently replaced as though the original
  experiment had produced today's population.
- **False as a current measurement.** The audit for
  [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md)
  reported twenty current governed capabilities, a 43,741-byte standard
  semantic-registry snapshot, and a 137,779-byte lowering-registry identity on
  its exact base. Those byte values had already moved on a later audit and are
  not current authority. The population and byte-budget test must be rerun at
  the implementation base; no replacement number belongs in this source
  comment.

## Outcome

Qualify the source comment as **Historical measurement — before the v2 pool,
2026-07-27**, so the five-capability numbers are unmistakably the pre-pooling
landing measurement, while preserving why the pool exists and its
injectivity argument. If current figures add durable value, record them as a
separate dated measurement produced by the owning checks; do not substitute
them into the historical result or change identity bytes, budgets, or pooling
logic.

## What closes this

The complete rationale is true in both temporal directions, every number is
either tied to its original ticket/commit or re-measured and dated, and the
compiler package gates, `make citations`, `tkt lint`, `git diff --check`, and
exact-base `tkt guard` pass.
