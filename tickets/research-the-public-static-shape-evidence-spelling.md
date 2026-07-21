---
id: research-the-public-static-shape-evidence-spelling
title: Research the public static-shape evidence spelling
status: done
priority: p0
dependencies: [prototype-shape-evidence-spike]
related: [prototype-shaped-value-api]
scopes: [research/shapes]
shared_scopes: [project/tickets, contracts/decisions, contracts/navigation]
paths: []
tags: [tiler-research, rust-api, shapes, precedents]
---
# Research the public static-shape evidence spelling

## Goal

Determine the safest, most ergonomic, and forward-compatible public Rust
spelling for exact static shape evidence before `prototype-shaped-value-api`
changes `tiler-ir`.

## Work

- verify stable Rust 1.89 const-generic and associated-constant limits from
  primary language documentation;
- compare open descriptor traits, sealed built-in rank families, tuple/type-list
  encodings, and macro-assisted spellings;
- inspect mature Rust tensor/numerics libraries for the actual public contracts
  and semver consequences they chose;
- adversarially review authority, forgery, denial-of-service bounds,
  diagnostics, monomorphization, coherence, and future extension; and
- update the retained shape-evidence research with facts, rejected alternatives,
  and one recommendation or an explicit unresolved product choice.

## Acceptance

The report distinguishes facts, inferences, proposals, and measurements; links
primary sources; demonstrates any critical stable-Rust claim against the 1.89
spike; passes documentation validation; and leaves the implementation ticket
with an unambiguous reviewed prerequisite.

## Outcome

The completed comparison recommends sealed, library-owned `StaticShapeN`
families. They preserve canonical cross-crate type identity; downstream
descriptors do not, even though checked refinement makes those descriptors
sound. A retained Rust 1.89 harness compares descriptor, owned-family, and
tuple spellings through 1,000 distinct shapes. The recommendation remains
explicitly pending Tom's review, and the implementation ticket cannot begin
until acceptance is recorded in ADR 0061.
