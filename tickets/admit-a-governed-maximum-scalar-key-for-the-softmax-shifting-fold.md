---
id: admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold
title: Admit a governed maximum scalar key for the softmax's shifting fold
status: in-progress
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-softmax-law
lease_expires_at: 1786070865
---
## User-visible outcome

`tiler.scalar` carries a governed per-point binary32 `maximum`, so the softmax's row-maximum fold has a scalar operation to be realized as. Without it the softmax's realization law cannot be written at all: its first stage is a fold whose combiner no registered scalar spells.

## Why it was split out rather than landed with the reciprocal square root

**Fact.** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) landed `rsqrt-f32` and deliberately did not land this one. The reciprocal square root's fact record is fully determined by already-registered facts; this key's is not, and the undetermined field is `SCALAR_FACT_NAN_RESULT_RULE`.

**The open question, stated so it can be answered by reading rather than guessed.** `SCALAR_FACT_NAN_RESULT_RULE`'s own documentation (`crates/tiler-ir/src/index/scalar.rs`) says every governed scalar states the field and that the absence of `SCALAR_FACT_CANONICAL_NAN_BITS` never carries meaning on its own. The two existing values are `CANONICAL_ARITHMETIC_NAN_PROFILE` -- the operation installs the governed payload -- and `DECLARED_PAYLOAD_PRESERVED`, which is a statement about a *constant's* declared payload. A maximum fits neither:

- `BinaryOp::F32Maximum`'s documentation (`crates/tiler-ir/src/kernel/model.rs:396-418`) states that it "performs no arithmetic" and "selects one of its operands' bit patterns rather than computing a new value", which is why it carries no rounding obligation. A rule naming the canonical arithmetic NaN would claim it *installs* a payload it never computes.
- `tiler::softmax-f32@1`'s own `SOFTMAX_F32_FACT_NAN_BEHAVIOUR` (`crates/tiler-ir/src/semantic/softmax.rs:619-621`) says a quiet NaN "propagates through both folds and poisons the whole row and every arithmetic-NaN result is canonicalized" -- a statement about the *composition*, whose canonicalization comes from the arithmetic steps downstream of the maximum rather than from the maximum itself.

So the key needs a third value in that vocabulary -- an operand-payload-selecting rule -- and minting one is a decision about the published fact vocabulary rather than a mechanical registration. Guessing it would put a wrong claim into a record an out-of-crate reference capability reads through the published field identifiers.

## What must be settled, and where the evidence is

1. The NaN-result rule's spelling, and whether a *signalling* NaN operand is in scope. IEEE 754-2019 `maximum` and ADR 0023's two-family separation are the authorities; `crates/tiler-metal/src/emit.rs` already emits the exact fixup built from ordered comparisons, and `crates/tiler-reference/src/softmax/tests.rs` carries `the_two_extrema_families_are_indistinguishable_through_the_pinned_formula`.
2. Whether the key's name encodes the family. `tiler::softmax-f32@1` pins `Maximum` and deliberately not `MaximumNumber` (`softmax.rs:261`, with the D-2 elimination in that module's header). A bare `maximum-f32` naming one of two families is admissible only if the number-preferring sibling can never later be registered under a name that reads as its complement.
3. The signed-zero ordering. Both Tiler extrema families order `-0.0 < +0.0`; the reference model's `torch.max` does not, and the record at `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv` carries the four `torch_max_of_signed_zeros_*` rows. Nothing in the decision rests on the reference's behaviour; the fact record must state Tiler's own.

## Closes when

A governed `maximum` scalar key is registered with a fact record whose NaN-result rule is derived from the authorities above rather than chosen, the new vocabulary value is documented beside `SCALAR_FACT_NAN_RESULT_RULE`, and the key lands as a labelled draft with its own acceptance node parked for Tom.
