---
id: correct-the-artifact-abi-contracts-bf16-producer-wall-paragraph
title: Correct the artifact ABI contract's bf16 producer-wall paragraph
status: done
priority: p2
dependencies: []
related: [admit-a-bf16-index-realization-law-and-refinement-contract, carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifacts, bf16]
---
## User-visible outcome

The artifact ABI contract's `bf16` paragraph describes the producer wall as it now stands, so a reader is not told that a reachable composition is unreachable.

## The stale sentence

**Fact.** `docs/artifact-abi.md`, in "**Fact — the carrier reaches artifact identity, and no producer can yet emit a `bf16` artifact**", states: "`tiler_ir::index`'s `NumericalContractIdentity` admits an `f32` contract key alone, and the standard semantic provider registers index-realization laws for nine `f32` and quantization operations and none for the registered `bf16` family, so a `bf16` semantic occurrence cannot obtain executable coverage and a stage covering nothing is refused at whole-program verification."

**Fact.** Every clause of that sentence is now false. `admit-a-bf16-index-realization-law-and-refinement-contract` gave `NumericalContractIdentity` a `bf16` route, registered three `bf16` index-realization laws (twelve rows total), and demonstrated a pure-BF16 program obtaining verified coverage for every occurrence and building a `VerifiedKernelProgram`. The heading's own claim — "no producer can yet emit a `bf16` artifact" — is *narrowly* still true only because the artifact-layer packaging evidence is separate work (`carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence`), not because the refinement layer refuses.

**Inference.** The paragraph's measurements are unaffected and must not be rewritten: the 48,584-byte identities and their four differing offsets are a dated observation of the encoding, and no test asserts them.

## What to do

Rewrite the wall clause to state the current boundary — the refinement layer admits `bf16`, and what remains is the artifact-layer producer fixture — and repoint the trailing ticket link to the artifact-packaging ticket. Preserve the measurements verbatim.

## Closes when

The paragraph describes the current boundary, the measurements are unchanged, and its ticket link resolves to the work that actually remains.

## Outcome

Executed inline by the coordinator on 2026-08-06 — the correction was fully specified above. The wall clause now states the current boundary (refinement admits `bf16`; the artifact-layer packaging fixture is what remains), the ticket link points at the packaging work, the measurements are preserved verbatim, and the direct-assembly fixtures' remaining justification (refusal envelopes no correct producer emits) is stated where the old justification was.
