---
id: require-both-elementary-evidence-halves-before-target-admission
title: Require both elementary evidence halves before target admission
status: done
priority: p1
dependencies: []
related: [declare-elementary-realizations-on-a-target-profile, carry-the-elementary-numerical-dimensions-in-the-region-realization]
scopes: [implementation/compiler, contracts/numerics, contracts/decisions, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, numerics, evidence]
---
## User-visible outcome

An elementary realization can satisfy hard target feasibility only when both its numerical bound and its exceptional-value behaviour are backed by evidence that the numerical contract permits to discharge a hard obligation. Empirical or `Unknown` evidence remains useful qualification evidence, but it cannot silently become permission to compile.

## Source-first audit at `611fefee15d8878b9458bd860d09490ec736a17f`

The 2026-08-11 audit dated `c0922dfc` is stale as a *base*, not as a claim set. Re-read at this commit, every Fact still holds. Purpose is unchanged.

**Fact — verified. The governing evidence rule is already accepted.** [`ADR 0042`](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md) still says `Proof, exhaustive evidence, or an applicable normative guarantee may discharge` a hard accuracy feasibility requirement, that empirical results `do not prove an unmeasured worst-case bound`, and that `Unknown behavior remains unknown and cannot satisfy a hard contract`. `ConformanceEvidenceClass::discharges_hard_requirement` is the exhaustive match `FormalProof | ExhaustiveFinite | NormativeGuarantee => true` / `EmpiricalQualification | Self::Unknown => false`. `ConformanceEvidence::discharge` is the only consumer of that line.

**Fact — verified. Current elementary admission does not enforce that rule.** `ElementaryRealization::discharge` still computes separate `bound` and `exceptional` answers from each half's `discharge()`. `assess_elementary_accuracy` still returns `Ok(ElementaryAccuracyAdmission { basis, discharge: realization.discharge() })` on `RefinementOutcome::Refines` alone. `assess_program_elementary_accuracy` still writes `assess_elementary_accuracy(contract, &installed, &registry)?;` and discards the admission. `the_bound_is_normative_and_the_exceptional_behaviour_is_only_empirical` and `the_softmax_exponential_refines_through_the_registered_implication` still assert `!discharge.exceptional_is_discharged()` after a successful `assess_elementary_accuracy`.

**Fact — verified. All three current governed rows are affected.** `installed_elementary_realizations` still pairs `metal_f32_exponential_bound_evidence` / `metal_f32_reciprocal_square_root_bound_evidence` (`NormativeGuarantee`) with `metal_f32_exceptional_value_evidence` / `metal_f32_normalization_exceptional_value_evidence` / `metal_f32_softmax_exceptional_value_evidence` (`EmpiricalQualification`). `each_family_carries_its_own_exceptional_value_corpus` still asserts every exceptional record `discharge().is_err()`.

**Inference — exposing the current private rows through a public target declaration before this repair would widen a correctness defect.** A caller-vouched contract assertion is not evidence that the implementation satisfies it. Public construction must not convert that assertion, a profile key, or a backend label into hard feasibility.

**Scope addition.** `contracts/optimizer` is required by the already-authorized documentation delivery: `docs/compiler/optimizer.md` is the optimizer contract that currently describes elementary admission as contract refinement alone.

## Decision accepted — 2026-08-11

Tom accepted the strict alternative in conversation:

- hard admission requires successful discharge of both the bound evidence and the exceptional-value evidence;
- declaration and assessment both validate this, so neither a malformed retained row nor a future internal caller can bypass the rule;
- a typed refusal identifies which half failed and the evidence class that failed to discharge;
- `no-installed-realization`, `unrefined-realization`, and `undischarged-evidence` remain distinct repairs;
- the three current governed Metal rows fail closed until stronger exceptional-value evidence lands or a separately accepted semantic contract legitimately narrows what must be proved; and
- empirical evidence remains recorded as evidence, never relabelled as proof.

The accepted downside is deliberate: elementary compilation that was previously admitted by contract assertion alone becomes unavailable until its hard evidence is complete.

## Required delivery

- Make both `bound_evidence.discharge()` and `exceptional_evidence.discharge()` necessary for target admission.
- Revalidate the same invariant at the profile declaration boundary and at assessment.
- Preserve the failing half and evidence class through the program assessment, request error, compile-failure mapping, and public structured refusal.
- Update the numerical contract and optimizer documentation so empirical qualification cannot be read as feasibility authority.
- Replace positive tests that currently admit `exceptional = false` with explicit fail-closed evidence, then add positive fixtures whose two halves genuinely discharge.
- Perturb the bound half and exceptional half independently, assertions unchanged, and retain the exact typed failure from each perturbation.

## Non-goals

- Do not fabricate stronger evidence from a target key, backend family, compiler version, or caller assertion.
- Do not weaken an operation's exceptional-value semantics to preserve availability.
- Do not add runtime sampling or a fallback realization.
- Do not decide the public declaration API here; [`declare-elementary-realizations-on-a-target-profile`](declare-elementary-realizations-on-a-target-profile.md) owns that surface after this invariant is healed.

## Closes when

No compile path can admit an elementary realization unless both evidence halves discharge, the three refusal classes remain typed and distinct, the current empirical-exception rows fail closed, and independent subject perturbations prove both evidence checks are load-bearing.

## Worker delivery

Implemented on `tkt/require-both-elementary-evidence-halves-before-target-admission` at the base above. Coordinator owns merge and close.

- `ElementaryRealization::declare` and `assess_elementary_accuracy` both require `bound_evidence.discharge()` and `exceptional_evidence.discharge()`. A retained Metal row can still be recorded with `new`; it cannot be admitted.
- `ElementaryRefusalReason::UndischargedEvidence` names the failing half and `ConformanceEvidenceClass`. Diagnostic code `accuracy.elementary.undischarged-evidence` stays distinct from `no-installed-realization` and `unrefined-realization`.
- `RequestError::UnrealizedElementaryAccuracy` carries the half and class. `CompileFailureClass::UnsupportedCapability` reports the diagnostic code. `TargetCompileRefusal::ElementaryAccuracy` is the public structured refusal. Candidate-declaration provenance is still the public-declaration ticket's surface.
- The three installed Metal rows remain honest records and fail closed as exceptional-value `EmpiricalQualification`. Positive fixtures use labelled test records, not relabelled Metal evidence.
- Independent perturbations: bound half to empirical; exceptional half to empirical; bound half to `Unknown`. Each keeps the same assertion and retains the typed failure.

Commands: `cargo test -p tiler-compiler --offline`; `cargo clippy -p tiler-compiler --all-targets --offline -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps --offline`; `tkt lint`; `git diff --check`.
