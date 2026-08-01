---
id: evaluate-bf16-reference-semantics
title: Evaluate BF16 reference semantics from an exact-rational oracle
status: todo
priority: p1
dependencies: [register-the-bf16-semantic-operation-signatures]
related: [spike-bf16-through-the-second-dtype-seams, preserve-primary-dtype-standards-evidence]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, reference, numerics]
---
## User-visible outcome

`tiler-reference` evaluates a pure-BF16 program to exact bits, so every later BF16 claim — a kernel, a lowering, a device result — has an independent oracle to be wrong against. Without it a BF16 backend would have nothing to compare to, which is how a silently wrong tensor survives.

## Why the oracle must be exact rational

**Measurement.** Finding 24 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records that no single operation can separate `f32`-precision evaluation from native `bfloat` arithmetic, because `f32`'s 24-bit significand exceeds the 18 bits that would make a second rounding to BF16's 8-bit significand innocuous.

**Inference.** An oracle that computed in host `f32` and rounded to BF16 would therefore agree with a double-rounding implementation *because it shares the defect*. Host-native arithmetic is not normative evidence for this dtype.

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) built exactly this oracle out of tree — exact rational arithmetic over `num-bigint`, one rounding at the observable materialization — and it agreed on all 65,536 encodings and on 24 hand-derived witnesses across six categories. The spike's `src/bf16.rs` and `src/corpus.rs` are the working draft this ticket productionizes; its exhaustive round-trip and its overflow-boundary check are the shape the tests should take.

## Implementation keys

- A reference evaluator registered for each of the three BF16 operation keys, refusing any operand whose resolved type is not `tiler::bf16@1`.
- A `ReferenceValueValidator` for `tiler::bf16@1` checking element width against the registered descriptor. `ReferenceElement` is already a width-generic byte carrier and needs no change — the spike confirmed it holds a 2-byte element and refuses an empty payload.
- Exact rational evaluation, rounded once, round-to-nearest-ties-to-even. Reuse `tiler_ir::semantic::accuracy::ExactRational` if a descriptor-parameterized ingress is added; otherwise state why a local exact type is kept. **Do not** route BF16 values through `ExactRational::from_f32` and a host `f32` without recording that the widening is exact and total for BF16 specifically and does not generalize to F64 or F128.
- Exceptional values decided, not inherited: both zeros and their signs, subnormals preserved, `inf * 0` and `inf - inf` as the canonical NaN, overflow at the midpoint above the largest finite value, and NaN canonicalization to the realization's stated payload.
- The arithmetic NaN canonicalization is shared with the crate's existing rule rather than redefined.

## Required evidence

- Every one of the 65,536 BF16 encodings round-trips decode-then-round unchanged, except NaNs, which canonicalize. This is `exhaustive-finite` evidence and should be stated as such.
- A hand-derived witness corpus covering zeros and signs, subnormals and underflow, ties, ordinary rounding, overflow, and infinities/NaN. Every expected value derived from the format's parameters, **not** by running the implementation and recording what it said.
- The overflow boundary checked on both sides: the midpoint below it rounds to the largest finite value, the threshold itself overflows.
- A perturbation that changes only the tie rule and is watched failing — the spike used ties-away-from-zero and saw 2 of 24 witnesses disagree while 0 disagreed under the normative rule. Without this the corpus could pass while measuring nothing.
- An F32 program still evaluates identically, pinned by the existing reference registry identity.

## Closes when

The three BF16 operations evaluate to exact bits, the exhaustive and witness populations both pass with their counts reported, the tie perturbation is observed failing, the reference registry identity's movement is recorded, and `docs/dtype-support.md`'s BF16 `Reference evaluation` cell moves with its boundary stated.

## Graph maintenance

- Depends on the operation keys existing; there is nothing to register an evaluator against otherwise.
- Gates `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which needs an oracle to compare a lowered kernel against.
- The reference registry identity is a durable number that this ticket moves. Record the before and after; a spike or prototype citing the old one is drift, not a failure.
- Exhaustive enumeration is available here because the format is 16 bits. `docs/dtype-support.md`'s dtype-addition recipe records that F64 and F128 are not exhaustively enumerable and need a stated bounded profile instead; do not let this ticket's method be read as the general one.
