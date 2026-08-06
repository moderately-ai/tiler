---
id: derive-the-oracle-for-a-permitted-divergence-candidate
title: Derive the oracle for a permitted-divergence candidate
status: in-progress
priority: p2
dependencies: []
related: [research-region-accuracy-contracts-and-analyzable-error-budgets, connect-certified-rounding-error-bounds-to-rewrite-permissions, register-a-flush-and-reassociate-numerical-contract, calibrate-and-activate-parallel-reduction-selection, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, reference, conformance]
claimed_from: todo
assignee: agent-divergence-oracle
lease_expires_at: 1785987904
---
## User-visible outcome

A derivation of what checks a candidate compiled under a *permissive* numerical contract, so that a bit-different result is qualifiable rather than unqualifiable. Today the reference layer can qualify only the strict contract, and it says so honestly by refusing to be constructed for anything else — which is the right behaviour and leaves a real hole nobody owns.

## Why this exists, and why its trigger has already fired

**Fact — the whole-program oracle refuses a permissive realization outright.** `ReferenceNumericalConformance::from_realization` (`crates/tiler-reference/src/conformance.rs:166`) destructures a `NumericalRealization` and returns an error for each of `contraction`, `reassociation`, `permutation`, and `signed_zero` resolved `Permitted`, and for either exceptional-value assumption resolved `AssumeAbsent`. Its module header states the ground exactly: the evaluator computes a separately rounded multiply and add and a strict left fold, which is one legal realization of a permissive contract, and "an oracle that returned a single value would assert a bitwise equality the contract does not promise. … Refusing names each gap instead of hiding it."

**Fact — this is not a flash-class problem and does not wait on any permission.** `NumericalContract::FLUSH_AND_REASSOCIATE_F32` (`crates/tiler-compiler/src/session.rs:1490`) is a registered contract; `crates/tiler-compiler/src/request.rs:393` resolves it, and [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md)'s 2026-08-02 sweep measured under it. So a reassociating candidate is compilable today and unqualifiable today. That is why this ticket is filed `todo` rather than `deferred`.

**Fact — the machinery a bounded oracle would use exists one layer down and is the right shape.** `CertifiedEnclosure`, `decide_predicate` (`crates/tiler-reference/src/accuracy.rs:772`), and the fail-closed three-way `ConformanceDecision` (`:735`) already answer "does this exact-rational candidate lie inside this enclosure" for one evaluation's accuracy predicate.

**Inference — what is missing is the lift from one evaluation to a program, and it is a derivation rather than a wiring job.** A whole-program enclosure is not the composition of pointwise ones — [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) decision 7 says exactly that of the accuracy contract — and [the certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)'s third obligation says a bound derived for a sequential fold does not admit a tree fold. So the object that bounds a program under a permissive contract has to be derived before anything is built.

## What this ticket must produce

- **What the oracle's subject is.** The contract's result *set* is larger than one value; state whether the checkable object is an enclosure derived from the contract and the schedule, a per-candidate bound the schedule carries, or an exhaustive enumeration over a bounded realization space — with the elimination run against correctness, cost, and maintainability rather than preference.
- **How it composes with what already exists.** Whether `decide_predicate` and `CertifiedEnclosure` lift, or whether a program-level object is a different type, and what each answer costs. State it so the answer can be refuted at the type level.
- **The refusal that survives.** `Undecided` must remain reachable and must remain a non-admission; an oracle that always answers is the failure mode this whole layer distrusts. Name at least one candidate class the derived oracle would refuse to qualify and say what closes that refusal.
- **What this eliminates, with grounds.** Per-contract golden regeneration is the obvious alternative and appears to fail on the same ground the refusal above exists for — a golden pins one member of a set the contract does not promise, moving the wrong claim from the oracle into the fixtures where it is harder to see. Run that elimination properly rather than inheriting this sentence.
- **The measurement boundary**, if any bounded probe is run: exact environment, procedure, and what the run cannot separate.

## Non-goals

Implementing an oracle; changing `crates/tiler-reference/`; admitting any numerical permission; deriving the online-softmax fold's own bound (owned elsewhere); qualifying any device row.

## Closes when

The corpus states what checks a candidate compiled under a permissive contract, the elimination among the candidate oracle shapes is stated so a reader can refute it, the surviving shape's relation to the existing enclosure machinery is derived from a source read, and at least one class the oracle refuses is named with what would close the refusal.
