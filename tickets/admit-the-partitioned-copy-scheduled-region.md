---
id: admit-the-partitioned-copy-scheduled-region
title: Admit the partitioned-copy scheduled region
status: blocked
priority: p1
dependencies: [repair-the-scheduled-vocabulary-census-and-concatenate-law-standing, decide-the-partitioned-copy-scheduled-region-public-surface]
related: [lower-the-partitioned-copy-region-through-kernel-ir, lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, schedule, concatenate, ownership, public-boundary]
---
## Outcome

One accepted partitioned concatenate index region projects into the first canonical verifier-classified `RegionProgram::PartitionedCopy` region. It carries ordered operand members, distinct input bindings, one output, and exact source/destination subdomains while retaining zero-extent members and repeated operand occurrences.

## Strict construction

Re-derive the accepted index law rather than copying unchecked offsets. Prefix offsets use checked arithmetic. Member rectangles must be pairwise disjoint and jointly exhaustive; source and destination maps must refine their bounds; rank, dtype, axis, and result shape must agree exactly. Zero-extent members stay in identity and execute no access. Deduplicate boundary input bindings, never ordered members.

Keep output ownership and source-partition coverage as distinct proof obligations. Do not weaken `ContributorPartition`, global launch-tail rules, generic single-write rules, or the accepted index ownership law. No pointwise identity spelling, fabricated scalar operation, default numerical record, or multi-kernel fallback is admitted.

## Closes when

Arity 2 and 8, every axis position, unequal/all-zero members, operand reorder, and `concat(x, x)` are covered; overlap, gap, overflow, wrong member, and wrong prefix perturbations fail by distinct typed rules; and old scheduled-region bytes remain identical or the owning domain moves coherently.

## Exact-base dispatch audit — 2026-08-17 at `783e9b5b743afafdf4957396dbcfdb2f4c34565c`

The implementation is **not dispatch-ready**. The accepted dependency fixes one semantic topology — one concatenate occurrence becomes one scheduled copy region and ultimately one kernel — but it does not fix the exact public Rust representation, builder transition, diagnostic vocabulary, proof/member association, supported population, or schedule/request identity tags and framing.

Current source makes those choices consequential rather than mechanical: `IndexRegion` and `ScheduledRegionBuilder` require one `ScalarProgram` and one `NumericalRealization`; `ScheduledRegionDiagnostic` has no partitioned-copy-specific variants; schedule identity is `tiler.schedule.v6\0`; the compiler request subject is `tiler.compiler.request-subject.v6\0`; and no `RegionProgram` or `PartitionedCopyProgram` type exists in `crates/`. The accepted downstream numerical-state record deliberately used the sum spelling conceptually and said exact names follow the source audit.

[`decide-the-partitioned-copy-scheduled-region-public-surface`](decide-the-partitioned-copy-scheduled-region-public-surface.md) now owns the missing Tom decision and complete Pareto audit. This ticket remains blocked until that exact surface is accepted. Do not infer a generic copy population from the name, let callers assert proof authority, assign tags opportunistically, or implement only the narrow fixture while exposing a broader public construct.
