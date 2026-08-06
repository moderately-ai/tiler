---
id: correct-the-one-region-premise-in-the-concatenate-absence-check
title: Correct the one-region premise in the concatenate lowering record's absence check
status: todo
priority: p3
dependencies: []
related: [lower-a-two-region-occurrence-through-one-index-access-capability, correct-the-one-region-per-occurrence-claim-in-the-records]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## What is stale

`docs/research/indexing/concatenate-fusion-role-and-lowering.md:182` opens absence check 5 with "One index-access capability emits one region, and resolution is by exact signature, so a variadic family needs one capability per arity." The first clause stopped being true on 2026-08-06: [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) gave `IndexAccessLoweringProvider` a defaulted `lower_sequence`, and `GovernedRootMeanSquareScaleF32` is a shipped provider that emits an ordered chain of regions for one occurrence.

The check's *conclusion* — resolution is by exact signature, so a variadic family needs one capability per arity — is unaffected and still holds. What is wrong is the premise it is stated from, which now reads as a general claim about index-access capabilities that the source refutes.

## Why it is a separate ticket

[`correct-the-one-region-per-occurrence-claim-in-the-records`](correct-the-one-region-per-occurrence-claim-in-the-records.md) swept `docs/` for this claim and corrected every site it could reach. This one sits in `research/indexing`, a scope that ticket does not hold and that three open tickets do, so it was reported rather than edited.

## What this must do

Restate check 5's premise so the arity conclusion is derived from signature-exact resolution alone, and verify the reworded check against `crates/tiler-compiler/src/capability.rs` rather than against this description. Confirm the surrounding absence checks in that block still say what their positive controls demonstrate.

## Closes when

Absence check 5 states a premise the source supports, its conclusion about per-arity capabilities is unchanged, and `grep -rn "emits one region" docs/` returns nothing that contradicts `lower_sequence`.
