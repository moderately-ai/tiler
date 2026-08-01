---
id: bound-the-reference-contraction-comparison-for-the-profile-cells
title: Bound the reference contraction comparison for the profile cells
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, realize-the-contraction-through-the-appendable-direct-path, bound-the-reference-contraction-iteration-space]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, contraction, language-model]
claimed_from: todo
assignee: worker-oracle-bound
lease_expires_at: 1785589250
---
## User-visible outcome

The reference evaluator can serve as the bit-exact oracle for every one of the L3 profile's six contraction correctness cells, or the comparison procedure for the cells it refuses is stated and owned — so no realization ticket has to quietly shrink its evidence to the cells that happen to fit.

## The blocker, exactly

**Fact — from `realize-the-strict-contraction-on-metal`'s recorded stop (2026-08-01).** `MAX_REFERENCE_TENSOR_ELEMENTS = 16 * 1024 * 1024` (`crates/tiler-reference/src/lib.rs:90`) bounds `output_count * contracted_count` in the contraction evaluator (`contraction.rs:450-456`). Four of the six cells exceed it: `w_prefill_q` at 20,971,520 (1.2×), `w_prefill_o` at 268,435,456 (16×), `w_prefill_mlp_in` and `w_prefill_mlp_out` at 402,653,184 (24×). Only `w_decode_kv` and `w_vocab_slice` fit.

## What this must decide, with the elimination stated

Whether the bound moves or the comparison stages. Raising the bound must derive the memory and time cost at the largest cell rather than picking a bigger number; the existing `IterationStepsExceeded { limit, actual }` refusal and the bound's own rationale are the authorities to read first — the bound exists so a malformed program cannot ask the host for an unbounded fold, and a raise that discards that protection is not an option. A staged comparison — evaluating the contraction in output slabs the bound admits and comparing slab digests — keeps the bound and changes the procedure; it must state why slab boundaries cannot change any folded value (each output element's fold is independent, which is a property of the registered signature to cite, not to assume). A third candidate, comparing only the two admitted cells and calling the four others covered by the retained L3 `result_sha256` values, converts a live oracle into a frozen golden and must say so explicitly if chosen.

## Closes when

Every profile cell has a stated, executable comparison route — through the evaluator directly, through a staged procedure whose independence argument is written where the procedure lives, or through an explicitly-frozen golden with its drift boundary named — and the choice's elimination is recorded in the ticket outcome.
