---
id: establish-hard-exceptional-value-evidence-for-metal-elementary-realizations
title: Establish hard exceptional-value evidence for Metal elementary realizations
status: done
priority: p1
dependencies: [declare-elementary-realizations-on-a-target-profile]
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, require-both-elementary-evidence-halves-before-target-admission]
scopes: [research/apple-targets, research/numerics, implementation/compiler, implementation/build, contracts/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, evidence]
---
## User-visible outcome

The Metal target declares SiLU `exp`, RMSNorm `rsqrt`, and softmax `exp` only where each operation's exceptional-value behaviour has evidence strong enough to discharge the hard numerical contract. Unsupported rows remain unavailable with a typed reason instead of inheriting permission from empirical tests.

## Why this is separate

**Fact — the retained exceptional-value corpora are empirical.** They are valuable regression evidence, but [`ADR 0042`](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md) does not allow empirical evidence to discharge hard feasibility.

**Fact — the three elementary occurrences are not one interchangeable claim.** SiLU and softmax both use `exp`, but under different surrounding semantics; RMSNorm uses `rsqrt`. Their dtype, compiler mode, normal and exceptional domains, and required result behaviour must be stated independently.

**Inference — restoring availability requires evidence work, not an admission exception.** [`require-both-elementary-evidence-halves-before-target-admission`](require-both-elementary-evidence-halves-before-target-admission.md) intentionally makes the current rows refuse. This ticket owns the evidence needed to make any one of them admissible again.

## Required research and delivery

- Audit each operation occurrence against the exact accepted semantic contract, compiler flags, dtype, Metal language/toolchain row, and exceptional inputs it owes.
- Seek only admissible hard evidence classes: a sound proof, exhaustive finite evidence for a genuinely finite domain, or an applicable normative guarantee. Record `Unknown` where none is available.
- Keep empirical device observations as bounded qualification evidence and never rename them as normative or exhaustive evidence.
- If a primary source is inaccessible, record the exact citation, identifier, access failure, affected claim, and what disagreement could change; do not bypass the publisher's access control.
- After the public declaration path exists, install only the individually supported rows through that path. Unsupported operations remain absent and fail with `no-installed-realization` or `undischarged-evidence` as appropriate.
- Perturb every admitted exceptional-evidence subject independently and retain the typed refusal proving the check reaches that row.

## Stops

- A proposal to narrow or change the semantic contract is a separate numerical decision, not an evidence repair.
- A new compiler/toolchain/device environment creates a new bounded claim; it does not inherit this evidence.
- If no qualifying evidence exists, close the affected row as unsupported and preserve the reconsideration trigger rather than adding a fallback.

## Closes when

Each of the three Metal elementary occurrences has either a qualifying exceptional-value evidence record admitted through the validated public declaration or an explicit typed unsupported outcome with a reproducible reconsideration trigger.

## Source-first audit at `4275c14bb3c5fb1d73f8ae41cdc803d871742481`

Tickets are written against a tree that has since moved. Re-read at this exact base before any edit.

**Fact — verified. The retained exceptional-value corpora are empirical and cannot discharge.** `metal_f32_exceptional_value_evidence`, `metal_f32_normalization_exceptional_value_evidence`, and `metal_f32_softmax_exceptional_value_evidence` in `crates/tiler-compiler/src/target/accuracy.rs` each construct `ConformanceEvidenceClass::EmpiricalQualification`. [`ADR 0042`](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md) still says `Proof, exhaustive evidence, or an applicable normative guarantee may discharge` a hard accuracy feasibility requirement, that empirical results `do not prove an unmeasured worst-case bound`, and that `Unknown behavior remains unknown and cannot satisfy a hard contract`. `ConformanceEvidenceClass::discharges_hard_requirement` is still the exhaustive match `FormalProof | ExhaustiveFinite | NormativeGuarantee => true` / `EmpiricalQualification | Self::Unknown => false`. Reproduce: search those three function names for `EmpiricalQualification`; search ADR 0042 for `do not prove an unmeasured worst-case bound`.

**Fact — verified. The three elementary occurrences are not one interchangeable claim.** `installed_elementary_realizations` stores three `ElementaryRealization::recorded` rows. The two exponentials share `metal_f32_exponential_bound_evidence` and carry distinct exceptional records because `the two operations reach different arguments`. SiLU's ordinary domain closes at `SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` (`0x42b17217`) and reaches finite overflow; softmax's closes at `SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` (`+0.0`) and makes overflow vacuous; RMSNorm is `rsqrt` in the `Faithful` form. Reproduce: search `installed_elementary_realizations` for the three `recorded` calls and the comment `a shared corpus record would qualify a population neither measured`.

No Fact was repaired. Purpose is unchanged: this ticket owns the exceptional-value evidence, not a contract narrowing and not crate admission in this wave.

**Scope addition.** `contracts/navigation` is required by the already-authorized catalog delivery: adding a research record must add its row in `docs/research/README.md` in the same change, and that path is the navigation catalog rather than `research/numerics`.

## Worker delivery

Research-only at the base above. Coordinator owns merge and close. No file under `crates/` was edited.

- Retained record: [`docs/research/numerics/metal-elementary-exceptional-value-evidence.md`](../docs/research/numerics/metal-elementary-exceptional-value-evidence.md). Catalog row added beside the bound-half sibling in [`docs/research/README.md`](../docs/research/README.md). Gap 3 of the accuracy record now points at the per-occurrence close-out.
- Per occurrence, hard class is `Unknown`. SiLU `exp`, RMSNorm `rsqrt`, and softmax `exp` are closed as unsupported for hard Metal-target admission. The three empirical corpora stay `EmpiricalQualification`.
- Citations attempted: both retained MSL revisions (hashes reproduced), the 2025-10-20 feature-set tables, OpenCL C 3.1.1 §7.5 fetched from the public Khronos HTML (contrast only; not retained; not a Metal guarantee), and the already-quoted IEEE 754-2019 Clause 9.2 recommendation in the numerics source record. IEEE bytes were not re-opened (metadata-only, IEEE copyright). No CAPTCHA, login, or paywall was hit.
- What would reverse each `Unknown`: a Metal sentence that prescribes that function's four exceptional rules under the governed precise flags; a Metal citation of IEEE 754 recommended operations or C99 F.9 for that function the way Table 6.4 already cites IEEE 754-2008 for `fma`; an exhaustive finite evaluation over every binary32 encoding on a *named* device/toolchain/mode row, which would still not inherit to another environment; a sound proof of the selected intrinsic; or a separately accepted contract narrowing. A bounded sample, an OpenCL citation, or a re-label of an existing digest does not reverse it.
- Crate-admission remainder: the public path (`TargetProfileBuilder::declare_elementary_realization`) already exists and was not used. `TargetProfile::governed` still declares no elementary row. `installed_elementary_realizations` still *records* the three empirical-exceptional rows; `ElementaryRealization::declare` and `assess_elementary_accuracy` still refuse them as `undischarged-evidence`. A later ticket may install one occurrence only after a trigger produces a discharging half. Installing `Unknown` or empirical evidence through the public path would regress [`require-both-elementary-evidence-halves-before-target-admission`](require-both-elementary-evidence-halves-before-target-admission.md).

Commands: `tkt lint`; `git diff --check`; `tkt guard --base origin/main`; `make citations`.
