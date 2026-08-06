---
id: enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle
title: Enumerate the freedom sites a physical plan must pin for a permissive conformance oracle
status: in-progress
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, apply-the-declared-numerical-conformance-on-every-reference-evaluation-path]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, reference, conformance, scheduling]
claimed_from: todo
assignee: agent-freedom-sites
lease_expires_at: 1786029868
---
## User-visible outcome

A complete enumeration of the places a physical plan can spend a categorical numerical permission, and for each one whether the plan already declares the choice it made — so the order witness an oracle consumes can be built from facts that exist, or the gaps are named.

## Why this exists

**Fact — [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) settled the object and left exactly this open.** The surviving oracle takes `(program, contract, realization witness)` and compares bit for bit. Its Part 2.4 records that most of the witness is already carried — `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs:563`) carries `ContributorPartition`, `ContributorOrder`, `accumulation`, `arrival`, and both permission fields on every folding variant, and the schedule verifier cross-checks them against the region's declared realization. What it does not record is whether that is *all* of them.

**Fact — it is not all of them, and one counterexample is already compilable.** [Numerical semantics](../docs/numerical-semantics.md) records that a one-input, one-output, three-leaf same-family `f32` add or multiply chain compiles through the `PointwiseF32Expression` projection when the contract admits one of the implemented reassociations. Such a region carries `ReductionTopology::None`, so no field of the plan names which grouping it emitted. That is the derivation's `OrderNotPinned` refusal class with a non-empty population today.

**Inference — the witness is an aggregation over freedom sites, not over reductions**, and nobody has counted the sites. Today the aggregation is done by hand, per test: `crates/tiler-compiler/src/pipeline/tests.rs` reads `partition` out of the region and passes it to `strict_partitioned_sum`; `prototypes/serial-sum-run/src/proof.rs` reads it from the plan's published launch geometry. Neither generalizes.

## What this ticket must produce

- **The enumeration**, over the eleven canonical dimensions and every construct a plan can carry, of where a categorical permission is *spendable* — read at source and stated with the exact path, not inferred from a type's name. The derivation's Part 3 table is the starting classification and is to be checked rather than inherited.
- **Per site, whether the plan declares its choice**, with the field named or the absence stated as an exact check a reader can rerun.
- **What a witness would have to determine**, stated so it can be refuted: two plans agreeing on the witness must agree in bits, and the way to refute the enumeration is to exhibit two that do not.
- **The public boundary identified and taken to Tom, never self-accepted.** A witness type, any change to `ReferenceNumericalConformance`'s construction, and any new plan-side field are each a public boundary under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). This ticket produces the derivation and the exact surface; acceptance is Tom's.

## Explicit non-goals

Implementing a witness or an oracle; editing `crates/`; changing a contract sentence; deciding whether the `PointwiseF32Expression` projection should carry a grouping field, which is a consequence of the enumeration rather than an input to it.

## Closes when

Every freedom site is enumerated at source, each is marked declared or undeclared with a reproducible check, the witness's determination property is stated refutably, and the public surface is written out for Tom without being adopted.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as the second of its two ownerless gaps.
