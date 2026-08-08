---
id: qualify-lowering-registry-pooling-measurement-as-historical
title: Qualify the lowering-registry pooling measurement as historical
status: todo
priority: p3
dependencies: []
related: [intern-the-lowering-registry-s-shared-authority-identities]
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

## Starting evidence, stale until re-read at this ticket's base

- The comment records five governed capabilities, a 1,496-byte semantic
  registry snapshot repeated five times, and a 15,583-byte lowering-registry
  identity.
- [`intern-the-lowering-registry-s-shared-authority-identities`](intern-the-lowering-registry-s-shared-authority-identities.md)
  records those figures as the **before** measurement that justified the v2
  pooled encoding; they should not be silently replaced as though the original
  experiment had produced today's population.
- The audit for
  [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md)
  reported twenty current governed capabilities, a 43,741-byte standard
  semantic-registry snapshot, and a 137,779-byte lowering-registry identity on
  its exact base. These are discovery evidence only: re-run the owning byte
  budget and population checks before using them.

## Outcome

Qualify the source comment so the five-capability numbers are unmistakably the
pre-pooling landing measurement, while preserving why the pool exists and its
injectivity argument. If current figures add durable value, record them as a
separate dated measurement produced by the owning checks; do not substitute
them into the historical result or change identity bytes, budgets, or pooling
logic.

## What closes this

The complete rationale is true in both temporal directions, every number is
either tied to its original ticket/commit or re-measured and dated, and the
compiler package gates, `make citations`, `tkt lint`, `git diff --check`, and
exact-base `tkt guard` pass.
