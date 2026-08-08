---
id: decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights
title: Decide whether the proof payload limit admits the vocabulary-projection weights
status: todo
priority: p2
dependencies: []
related: [route-the-realization-conformance-half-into-the-conformance-crate]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The hard stop, measured

`w_vocab_slice` is the one L3 contraction cell that **cannot route**, and the reason is exact rather than approximate. Its `[8192, 1024]` weights operand is **33,554,432 bytes**; `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` is **16,777,216** (`crates/tiler-artifact/src/proof/mod.rs`, search `MAX_PROOF_PAYLOAD_BYTES`). **Exactly a factor of two** — coordinator-verified by arithmetic and by the constant.

Observed as `Limit(ProofLimitExceeded { kind: PayloadBytes, attempted: 33554432, limit: 16777216 })`.

The conformance work that found this **did not touch `tiler-artifact`**: it derived the exclusion from the constant and pinned it to the doubling arithmetic with a test, so shrinking the cell's `n` to 4096 fails three tests at once. The exclusion is derived, not hand-asserted — which means raising or keeping this limit is a live decision rather than a number someone can quietly edit.

## What is actually being decided

`MAX_PROOF_PAYLOAD_BYTES` is **`pub`**, so it is a public boundary under ADR 0075 and its value is part of the artifact contract. Three readings, and they are genuinely different:

- **The limit is right and this cell is out of scope for proof payloads.** A 32 MB operand embedded in a proof is a different thing from a kernel's own bytes, and the vocabulary projection is the largest tensor in the pinned workload. If so, say what evidence *does* cover that cell, because it is currently the only L3 cell with none.
- **The limit is an arbitrary round number that has not been re-derived since it was set.** Then the question is what it should be derived *from* — a real bound on what a consumer must hold in memory to validate a proof, rather than a doubling.
- **The payload should not carry weights at all.** If a proof can reference an operand rather than embed it, the limit stops binding and the identity question moves instead. That is the largest change and the one that most needs stating before anyone raises a constant.

## Read before deciding

`crates/tiler-artifact/src/proof/` in full — particularly what a payload is required to contain and why a bound exists at all. The constant's own documentation is the first evidence; `AGENTS.md` ranks a reasoned bound above a round number, so establish which this is.

**Do not raise the limit as the default move.** A limit doubled to fit the one case that exceeded it is a limit that will be doubled again, and this repository treats that shape as a defect rather than a fix.

## Closes when

The reading is established with evidence; if the value changes, every pinned identity that folds it is recomputed on the merged tree and reported; and the L3 cell either routes or has its exclusion recorded with the evidence that covers it instead. A change to a `pub` constant in the artifact contract is Tom's under ADR 0075.
