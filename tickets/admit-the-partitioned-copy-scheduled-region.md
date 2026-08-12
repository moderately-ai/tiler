---
id: admit-the-partitioned-copy-scheduled-region
title: Admit the partitioned-copy scheduled region
status: todo
priority: p1
dependencies: [admit-an-explicit-non-arithmetic-region-and-delivery-state, repair-the-scheduled-vocabulary-census-and-concatenate-law-standing]
related: [lower-the-partitioned-copy-region-through-kernel-ir, lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, schedule, concatenate, ownership, public-boundary]
---
## Outcome

One accepted partitioned concatenate index region projects into one canonical non-arithmetic scheduled `PartitionedCopy` region. It carries ordered operand members, distinct input bindings, one output, and exact source/destination subdomains while retaining zero-extent members and repeated operand occurrences.

## Strict construction

Re-derive the accepted index law rather than copying unchecked offsets. Prefix offsets use checked arithmetic. Member rectangles must be pairwise disjoint and jointly exhaustive; source and destination maps must refine their bounds; rank, dtype, axis, and result shape must agree exactly. Zero-extent members stay in identity and execute no access. Deduplicate boundary input bindings, never ordered members.

Keep output ownership and source-partition coverage as distinct proof obligations. Do not weaken `ContributorPartition`, global launch-tail rules, generic single-write rules, or the accepted index ownership law. No pointwise identity spelling, fabricated scalar operation, default numerical record, or multi-kernel fallback is admitted.

## Closes when

Arity 2 and 8, every axis position, unequal/all-zero members, operand reorder, and `concat(x, x)` are covered; overlap, gap, overflow, wrong member, and wrong prefix perturbations fail by distinct typed rules; and old scheduled-region bytes remain identical or the owning domain moves coherently.
