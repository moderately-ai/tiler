---
id: require-both-elementary-evidence-halves-before-target-admission
title: Require both elementary evidence halves before target admission
status: todo
priority: p1
dependencies: []
related: [declare-elementary-realizations-on-a-target-profile, carry-the-elementary-numerical-dimensions-in-the-region-realization]
scopes: [implementation/compiler, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, numerics, evidence]
---
## User-visible outcome

An elementary realization can satisfy hard target feasibility only when both its numerical bound and its exceptional-value behaviour are backed by evidence that the numerical contract permits to discharge a hard obligation. Empirical or `Unknown` evidence remains useful qualification evidence, but it cannot silently become permission to compile.

## Source-first audit at `c0922dfcc86283e2ad05b41f74a11d2940087513`

**Fact — the governing evidence rule is already accepted.** [`ADR 0042`](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md) and `ConformanceEvidence::discharge` distinguish evidence that can discharge a hard requirement from empirical evidence that can only detect regressions and from `Unknown`, which cannot discharge anything.

**Fact — current elementary admission does not enforce that rule.** `ElementaryRealization::discharge` computes separate bound and exceptional-value answers, but `assess_elementary_accuracy` admits solely when the declared `AccuracyContract` refines the requested contract. `assess_program_elementary_accuracy` then discards the discharge result. The tests explicitly retain the contradictory state `bound = true`, `exceptional = false` while admission succeeds.

**Fact — all three current governed rows are affected.** The installed SiLU `exp`, RMSNorm `rsqrt`, and softmax `exp` rows pair normative bound evidence with empirical exceptional-value evidence. None currently has hard-discharge evidence for both halves.

**Inference — exposing the current private rows through a public target declaration before this repair would widen a correctness defect.** A caller-vouched contract assertion is not evidence that the implementation satisfies it. Public construction must not convert that assertion, a profile key, or a backend label into hard feasibility.

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
