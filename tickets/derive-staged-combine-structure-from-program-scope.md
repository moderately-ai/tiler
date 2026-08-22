---
id: derive-staged-combine-structure-from-program-scope
title: Derive staged combine structure from program scope
status: todo
priority: p1
dependencies: []
related: [accept-the-exact-composed-reference-session-and-event-surface, encode-identity-bearing-staged-combine-structure]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [research, spike, reference, conformance, numerics]
---
## User-visible outcome

A bounded, reproducible answer to one question: can a kernel's staged intra-workgroup combine structure be derived from program scope alone, or does it require an identity-bearing encoding? The answer decides whether the composed-reference decision has one candidate or two, so it is worth answering before that packet reaches Tom.

## Why this exists

Filed 2026-08-22 by the coordinator from the composed-reference session re-gate, which found that ADR 0112's landed witness shape is a fifth materially distinct candidate the original packet predates. That candidate carries a real prerequisite, and this spike is what tells us how large it is.

**Fact — the witness refuses a staged kernel today, and says why.** `crates/tiler-ir/src/program/contraction_witness.rs` refuses any kernel that, at anchor `declares workgroup staging`, has an intra-workgroup combine structure the witness cannot see. Its module header states the consequence at anchor `must become identity-bearing in`. Both anchors resolve exactly once in that file; verified by the coordinator at `b3c07259`.

**Fact — the witness itself is a landed public value, not a proposal.** `ContractionF32PlanWitness` appears in `crates/tiler-ir/src/program/contraction_witness.rs`, `crates/tiler-ir/src/program/mod.rs`, `crates/tiler-compiler/tests/contraction_topology_witness.rs`, and `crates/tiler-reference/src/contraction/topology.rs` — four files, so the shape is exercised across the IR, compiler, and reference layers rather than declared in one.

## Required work

- Re-audit both Facts at your own base before starting; the citations above are anchors, not line numbers, and each was grepped against the file it names.
- Answer the question with a bounded spike under `spikes/`, run from a documented command, with explicit inputs, outputs, and stop condition. Do not add a workspace gate.
- State the answer as one of: **derivable** (structure recoverable from program scope with no encoding change — the frontier collapses to one candidate); **not derivable** (an identity-bearing encoding is required — [`encode-identity-bearing-staged-combine-structure`](encode-identity-bearing-staged-combine-structure.md) becomes a real prerequisite); or **undecided**, with exactly what would settle it.
- Whichever answer, say what it costs to be wrong in each direction.

## Non-goals

Implementing either candidate; editing `docs/decisions/`; changing the witness or the evaluator; and deciding the composed-reference surface, which is Tom's.

## Closes when

The question is answered with reproducible evidence, the cost of being wrong is stated in both directions, and the composed-reference packet's frontier is updated to match.
