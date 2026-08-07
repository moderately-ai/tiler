---
id: correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed
title: Correct the fusion-legality-wall claims left in the compiler after BF16 legality landed
status: in-progress
priority: p2
dependencies: []
related: [establish-bf16-optimizer-legality, correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-correct-the-
lease_expires_at: 1786137974
---
## What is false

`establish-bf16-optimizer-legality` landed on 2026-08-07 and widened fusion legality to BF16. Three compiler-side comments still describe the wall it removed, in the present tense, and **two of them cite tests that no longer exist**. All three were verified by reading the files in full on 2026-08-07, not from a scan.

### 1. `crates/tiler-compiler/src/session.rs:1729-1736` — the most consequential, because it documents a public constructor

The doc comment on `pub const fn strict_bf16()` states:

> **It is still not general support.** A `bf16` region covering several occurrences meets `fusion_legality`, whose capability table is keyed by the `f32` operation set, and every cover placing it is ruled out — … `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` is where that boundary is asserted, and `establish-bf16-optimizer-legality` owns it.

Every clause is now false. The cover is no longer ruled out, the ticket named as owner is `done`, and **the cited test does not exist anywhere in `crates/`** — `grep -rn a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall crates/` returns only this citation itself. It was renamed to `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:543`).

This one is first because it is a **rustdoc comment on published API**, so the false statement is what a consumer reads.

### 2. `crates/tiler-compiler/src/pipeline/tests.rs:3990-4000`

> **What keeps this region hand-assembled is the *shape* it needs, not the dtype.** … put to `derive_fusion_legality` before any cover survives — an authority still keyed by the `f32` operation set, so the region is `Unknown` … **Until it lands**, a compiled BF16 region is reachable for a single-occurrence program and this chain is not …

"Until it lands" is the tell: it landed. Note the **first half of this same comment (`:3981-3988`) is already correct** — it is a properly dated correction citing `a_flush_accepting_bf16_contract_reaches_a_selected_plan`, which does exist at `:484`. So this is a half-corrected comment; repair the second half without disturbing the first.

### 3. `crates/tiler-compiler/src/pipeline/tests.rs:3873`

> **The refusal the recognizer's `dtype-f32` rule used to absorb.** That rule …

Past tense, so this is **probably already correct** and is listed only so a worker checks it rather than assuming. Classify it; do not rewrite it reflexively.

## What is true now, so the correction does not overshoot

A multi-occurrence BF16 region **fuses**, under a proof carried at its own width with every obligation derived. Two boundaries survive and must be named rather than dropped:

- **Reassociation is withheld as `Unknown`** — it is not proved, merely not required.
- **The four reduction obligations are discharged vacuously, over an empty population.** A vacuous discharge is not evidence the reductions are correct; it means none were present. Stating it as "reductions are legal" would be a worse error than the text being replaced.

A surviving wall test exists and should be cited in place of the dangling name: `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:691`).

## Why this is its own ticket

Found while assessing `correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate`, whose scope is `implementation/conformance` — it cannot reach these files. These are `implementation/compiler` and were unowned.

## Required evidence

- Every test name cited in a comment under `crates/tiler-compiler/` **resolves** against `cargo nextest list`. This is the check that would have caught two of the three defects, and it is the one worth keeping.
- No comment states the multi-occurrence fusion wall as current behaviour.
- Wherever the fusion is stated as reachable, the reassociation-`Unknown` and vacuous-reduction boundaries are stated with it.

## Closes when

Each of the three sites is classified as **live false claim** or **already-dated correction**, and each live one is repaired. Report the classification per site with the evidence — a count cannot distinguish the two, which is the failure mode this repository keeps hitting. `cargo nextest list` resolves every cited test name in the crate, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler` passes (this one **does** exercise `session.rs`, unlike the conformance crate whose modules are all `#[cfg(test)]`).


> **Correction, 2026-08-07 — the coordinator's "reassociation is withheld as `Unknown`" was over-general and is struck.** Found by the worker on [`correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed`](correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed.md), which declined to write the claim into the code rather than repeating it, and verified by the coordinator at `crates/tiler-compiler/src/fusion_legality.rs:1641-1653`.
>
> The obligation is discharged **`SoundProof`** when `!has_reduction || reassociation == Forbidden`. A multi-occurrence **pointwise** BF16 region has no reduction, so its `ReductionReassociation` records `SoundProof` **vacuously** — not `Unknown`. The `Unknown { "unproven-reassociation" }` branch requires a reduction **and** a permitting contract, which is precisely the surviving wall `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:691`).
>
> **The substance stands and only the mechanism was wrong:** reassociation is *not proved* for these regions, merely *not required*, because the region carries no reduction order to preserve. Say that, grounded on `BF16_FACT_REASSOCIATION_PERMITTED` being `false` and no BF16 family declaring an algebraic capability. **Writing "the obligation records `Unknown`" would be a new false claim** — the exact defect these tickets exist to remove.
