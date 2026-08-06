---
id: correct-the-dtype-ledger-bf16-abi-cell-for-the-landed-producer-path
title: Correct the dtype ledger's BF16 ABI cell now that a producer-built artifact exists
status: done
priority: p2
dependencies: []
related: [carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence, carry-bf16-through-the-artifact-encoding-and-identity, admit-a-bf16-index-realization-law-and-refinement-contract]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtypes]
---
## User-visible outcome

The BF16 `ABI and materialization` cell in `docs/dtype-support.md` states that a producer can build a BF16 artifact, so a reader is not told a landed composition is still walled off.

## The stale text

**Fact, at `docs/dtype-support.md:136`.** The cell's most recent correction reads: "What bounds the guarantee: no producer can build a BF16 artifact — no BF16 index-realization law or refinement contract exists, so the test envelopes are assembled directly — and [`admit-a-bf16-index-realization-law-and-refinement-contract`] owns that wall."

**Fact.** Every clause is now false. `admit-a-bf16-index-realization-law-and-refinement-contract` is merged and registered the three BF16 index-realization laws and the `Bf16NumericalContractKey` route into `NumericalContractIdentity`; `carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence` then carried a pure-BF16 constant/multiply/add program from semantic construction through verified coverage, a `VerifiedKernelProgram`, a `VerifiedArtifactProgram`, its encode/decode round trip, and its identity re-derivation, with an F32 twin built from the same parameterized construction to show the two are distinct artifacts.

**Inference.** The measurements in the surrounding prose stay verbatim. The 97,060-byte fixture encoding, the forty differing byte positions, and the four differing identity bytes are dated observations of the *carrier-only forged* pair and remain accurate for that pair; they are not the producer-path pair's numbers and must not be relabelled as such.

## What to do

Rewrite the bounding clause to state what now bounds the guarantee — a producer-built BF16 artifact exists and round-trips, and what remains unmeasured is device execution and conformance, which `validate-bf16-at-the-runtime-routing-boundary` and `conform-the-bf16-vertical-end-to-end` own. Reassess whether the cell's maturity qualifier should move now that the composition is a tested guarantee rather than an encoding-only one. Preserve every existing measurement verbatim and keep the carrier-only pair labelled as such.

## Closes when

The cell and its supporting paragraph describe the landed producer path, the retained measurements are unchanged and correctly attributed, and no sentence in `docs/dtype-support.md` still asserts that a BF16 artifact cannot be produced.

## Outcome

Executed inline by the coordinator at the packaging landing's integration — the correction was fully specified above. The ABI cell moves to "tested guarantee, producer path through round trip; no execution", the bounding clause states the landed composition with the producer-path pair's own measurements, the forged pair's measurements stay verbatim and attributed, and the remaining unmeasured half points at the two execution owners.
