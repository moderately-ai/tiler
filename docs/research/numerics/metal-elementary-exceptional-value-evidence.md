---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.metal-elementary-exceptional-value-evidence"
kind: "research"
title: "Metal elementary exceptional-value evidence"
topics: ["numerics", "transcendentals", "accuracy", "metal", "apple-targets", "exceptional-values"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["unknown"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.numerics.metal-elementary-function-accuracy", "tiler.research.numerics.transcendental-accuracy-precedents", "tiler.research.apple-targets.numerical-behaviour"]
ticket: "establish-hard-exceptional-value-evidence-for-metal-elementary-realizations"
---

# Metal elementary exceptional-value evidence

**Status:** a per-occurrence close-out of the exceptional-value half for the three Metal elementary realizations. Each row is independently `Unknown`. None of the three admissible hard classes discharges it. This record installs nothing on a target profile.

## Traceability

- **Work record:** [`establish-hard-exceptional-value-evidence-for-metal-elementary-realizations`](../../../tickets/establish-hard-exceptional-value-evidence-for-metal-elementary-realizations.md).
- **Bound-half sibling:** [Metal elementary-function accuracy guarantee](metal-elementary-function-accuracy.md), which already names Gap 3 — "the table states accuracy and no exceptional-value contract" — and stops there. This record is that gap applied to the three registered occurrences rather than restated as one interchangeable claim.
- **Vocabulary:** [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md). Proof, exhaustive finite evidence, or an applicable normative guarantee may discharge a hard accuracy feasibility requirement. Empirical qualification "detects regressions and characterize implementations but does not prove an unmeasured worst-case bound". `Unknown` "remains unknown and cannot satisfy a hard contract".
- **Carrier of that line:** `ConformanceEvidenceClass::discharges_hard_requirement` in `crates/tiler-ir/src/semantic/accuracy/evidence.rs` is the exhaustive match `FormalProof | ExhaustiveFinite | NormativeGuarantee => true` / `EmpiricalQualification | Self::Unknown => false`.
- **Contracts the three occurrences owe:** `silu_f32_exponential_exceptional_contract` in `crates/tiler-ir/src/semantic/silu.rs`, `rms_norm_f32_rsqrt_exceptional_contract` in `crates/tiler-ir/src/semantic/rms_norm.rs`, and `softmax_f32_exponential_exceptional_contract` in `crates/tiler-ir/src/semantic/softmax.rs`.
- **Installed Metal rows this record is about:** `metal_f32_exceptional_value_evidence`, `metal_f32_normalization_exceptional_value_evidence`, and `metal_f32_softmax_exceptional_value_evidence` in `crates/tiler-compiler/src/target/accuracy.rs`. All three are `EmpiricalQualification`.

Claims are labelled **Fact** when traced to a retained specification at a verified digest or to inspected source, **Inference** when derived from stated facts, **Proposal** when not yet accepted, and **Measurement** when tied to an exact environment and procedure. This record takes no new device measurement.

## Why three rows rather than one `exp` claim

**Fact — the compiler already refuses to treat them as one record.** `installed_elementary_realizations` stores three `ElementaryRealization::recorded` rows. The two exponentials share `metal_f32_exponential_bound_evidence` — "the same quoted Table 8.1 entry at the same digest" — and carry distinct exceptional records "because the two operations reach different arguments and a shared corpus record would qualify a population neither measured". The normalization row is a different function.

**Fact — the three contracts are not interchangeable even where three of the four rule tags agree.** Each of `silu_f32_exponential_exceptional_contract`, `softmax_f32_exponential_exceptional_contract`, and `rms_norm_f32_rsqrt_exceptional_contract` is `CanonicalNan` / `SignedInfinity` / `CanonicalNan` / `SignedInfinity`. `refines` compares the exceptional-value contract as one record, but the *inputs those rules govern* differ:

| Occurrence | Operation key | Subordinate function | Ordinary domain of that function | What the exceptional half actually reaches |
| --- | --- | --- | --- | --- |
| SiLU `exp` | `tiler::silu-f32@1` | `exp` of `-x` | unbounded below, closed above at `SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` (`0x42b17217`) | NaN operand; `exp(+inf)` from `x = -inf`; finite overflow on the whole `-88.73` band and below, which the activation then divides into `-0.0` |
| RMSNorm `rsqrt` | `tiler::rms-norm-f32@1` | `rsqrt` of the mean-square-plus-`eps` | open at zero, unbounded above | NaN operand; `+inf` from `1/sqrt(+0)` (unreachable while `eps` is positive); domain error on a negative argument (unreachable for a sum of squares); finite overflow (the header's range derivation makes this vacuous) |
| softmax `exp` | `tiler::softmax-f32@1` | `exp` of `s_i - m` | unbounded below, closed above at `+0.0` (`SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`) | NaN from a poisoned `s_i - m`; infinite-reference and finite-overflow rules are *vacuous* on this domain because `e^t <= 1` |

**Inference — discharging one row would not discharge the others.** A normative `exp(+inf) = +inf` sentence would speak to SiLU's overflow band and would not speak to RMSNorm's negative-argument NaN. A normative `rsqrt(x<0) = qNaN` sentence would not speak to either exponential. Softmax's vacuous overflow rule still has to match exactly, because `refines` refuses `DifferentExceptionalValueContract` rather than treating a vacuous rule as optional.

## The row each occurrence is asking about

Every row below is F32, Metal shading language, precise math selection. The applicability inference that `-fno-fast-math` equals `-fmetal-math-fp32-functions=precise` and `-fmetal-math-mode=safe` is Gap 2 of the accuracy record; this record inherits it and does not re-litigate it. The governed baseline adds `-ffp-contract=off`. The language revisions in scope are the two retained specifications:

| Short name | Document | Retained path | SHA-256 |
| --- | --- | --- | --- |
| **MSL 4.1** | Metal Shading Language Specification, Version 4.1, dated 2026-06-04 | `docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` | `41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5` |
| **MSL 4** | Metal Shading Language Specification, Version 4, dated 2025-10-23 | `docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf` | `eed87a82d4d2d475423b91b3c529c5313a85433f83e22b7fe3ec50e90254f44a` |

The hashes were reproduced this session with `shasum -a 256 docs/research/apple-targets/sources/*.pdf`. Page numbers below are the printed page numbers of MSL 4.1 unless stated.

The compiler-mode / intrinsic identity the bound half already uses, and which this half would have to cover if it ever discharged, is `air.exp.f32` selected by `precise::exp` and `air.rsqrt.f32` selected by `precise::rsqrt` under `-std=metal4.0`. That identity is compile-side evidence from the [Metal transcendental emission probe](../../../spikes/numerics/metal_transcendental_emission/README.md), not an exceptional-value guarantee.

## Sources attempted

### Retained Metal specifications — read

**Fact — chapter 8 still has no per-function exceptional-value table.** Re-read this session from both retained PDFs via `pdftotext -layout`. §8.1 (MSL 4.1 page 368) states in full: "INF must be supported for single-precision, half-precision, and brain floating-point numbers. NaNs must be supported for single-precision, half-precision, and brain floating-point numbers (with fast math disabled). If fast math is enabled the behavior of handling NaN or INF (as inputs or outputs) is undefined. Signaling NaNs are not supported. Denormalized single-precision, half-precision, or brain floating-point numbers passed as input to or produced as the output of single-precision, half-precision, or brain floating-point arithmetic operations may be flushed to zero." §8.3 is the one sentence "Floating-point exceptions are disabled in Metal." §8.4's Table 8.1 gives `exp` as `<= 4 ulp` and `rsqrt` as `Correctly rounded` and nothing else. §8.5 (pages 375–376) is titled "Edge Case Behavior in Flush to Zero Mode" and lists the four flush-permission results plus "if an operand or result is flushed to zero, the sign of the zero is undefined."

**Fact — Table 6.4 describes `exp` and `rsqrt` by name only.** MSL 4.1 page 206: `T exp(T x)` is "Exponential base e function." Page 207: `T rsqrt(T x)` is "Compute inverse square root of x." The same table's `fma` row is the contrast: "Edge case behavior is per the IEEE 754-2008 standard." Apple knows how to attach that sentence to a math function and attached it to `fma` only.

**Fact — the one IEEE-754-NaN sentence in chapter 6 is about `clamp` and `saturate`, not about math functions.** MSL 4.1 page 201: "For single precision floating-point, Metal also supports a precise and fast variant of the following common functions: clamp and saturate. The difference between the Fast and precise function variants handle NaNs differently. In the fast variant, the behavior of NaNs is undefined, whereas the precise variants follow the IEEE 754 rules for NaN handling." The functions named are `clamp` and `saturate`. `exp` and `rsqrt` are not in that sentence.

**Fact — neither retained revision mentions C99, C11, Annex F, or section F.9.** `pdftotext -layout` over each PDF, then a search for `\bF\.9\b|Annex F|\bC99\b|\bC11\b`, returns no match. A search for `shall conform` returns no match. The same census on MSL 4 agrees.

**Inference — §8.1 makes `+inf` and a NaN *representable*, not *prescribed*.** "INF must be supported" and "NaNs must be supported (with fast math disabled)" are existence claims about the format. They do not say `exp` of a finite overflow is that infinity, they do not say `exp(NaN)` is Tiler's canonical arithmetic NaN payload, they do not say `rsqrt` of a negative is that NaN, and they do not say `rsqrt(+0)` is `+inf`. Writing any of the four `ExceptionalValueContract` rules from §8.1 would be stating a rule the specification does not contain. The accuracy record already drew this inference for Gap 3; the census above is the independent re-read at this base.

**Inference — §8.5 cannot discharge a signed-zero or subnormal rule either.** Its four clauses are permissions ("may return", "may be flushed") and its last sentence makes the sign of a flushed zero undefined. A permission is not a prescribed result. Softmax's underflow band and RMSNorm's subnormal row reach this section, as the accuracy record already traces; they reach it as an *undefined sign*, which is the opposite of a discharging rule.

### Metal Feature Set Tables — read, empty for this question

**Fact.** `docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf`, SHA-256 `cee50f0c32a9af4a3cc4eeb8ab0d3d5d6444173f15800771ad2316f48603e07e`, contains no `rsqrt` row, no `NaN` row, no `INF` row, no "edge case" row, and no "math function" row. The seven `exp` hits are the English word "explicit" / "express" / "EXPRESS". Capability tables do not state elementary exceptional values.

### OpenCL C 3.1.1 — fetched as contrast, not as a Metal guarantee

**Fact — OpenCL specifies math-function edge cases by incorporation, and Metal does not.** The public [OpenCL C Specification](https://registry.khronos.org/OpenCL/specs/3.0-unified/html/OpenCL_C.html) version v3.1.1 (Khronos OpenCL Working Group, 2026-05-22, git `9f68efb5d80e77a4e437c5b8ee67d581666d044c`) was fetched this session from that URL without an access barrier. §7.5 opens: "The edge case behavior of the math functions shall conform to sections F.9 and G.6 of the C99 Specification, except where noted below." §7.5.1 then lists additional prescribed results. §7.5.3 is titled "Edge Case Behavior in Flush to Zero Mode" and is the section Metal §8.5 matches in structure and wording.

**Inference — sharing a flush-to-zero paragraph with OpenCL is not inheriting OpenCL's C99 incorporation.** Metal copied the flush-permission section and omitted the sentence that binds math functions to C99 F.9. Treating Metal `precise::exp` as an OpenCL `exp` would invent the omitted sentence. OpenCL is therefore contrast, not an applicable normative guarantee for any of the three Metal rows.

The OpenCL specification is not a retained document in this repository. The fetch is dated 2026-08-13 against the live Khronos HTML. A later Khronos edit would have to be re-read; it still could not become a Metal guarantee.

### IEEE Std 754-2019 — not re-opened; existing digest-backed quotes used

**Fact — `exp` and `rSqrt` are recommended operations, not required ones.** The [numerics source record](sources/README.md#ieee-754-2019) already quotes the purchased copy (SHA-256 `2fe5f245fa6fd027a64067e2d91d9000f51e9c61ad23fe1914d8cae41f2b0fb4`) at Clause 9.2 / Table 9.1: "Language standards should define, to be implemented according to this subclause, as many of the operations in Table 9.1 as is appropriate to the language." And: "A conforming operation shall return results correctly rounded for the applicable rounding direction for all operands in its domain." The same paragraph names `rSqrt` as sitting in that table under the same recommendation. The bytes are metadata-only under IEEE copyright and were not re-opened this session.

**Fact — Metal Table 8.1 does not claim that recommended `exp`.** It states `exp <= 4 ulp`, not correctly rounded. A 754-conforming recommended `exp` would be correctly rounded. The two claims contradict, so Metal's precise `exp` is not an IEEE 754 recommended `exp`.

**Inference — IEEE 754-2019 cannot discharge any of the three exceptional-value halves.** Metal's own covering sentence is "Metal is compliant to a subset of the IEEE 754 standard" (MSL 4.1 page 368). A subset claim plus a recommended-only clause is the opposite of an applicable guarantee. Even if Clause 9 also prescribed exceptional values for recommended `exp` and `rSqrt` — that exceptional-value table was **not re-read** from the purchased bytes this session — applying those prescriptions to Metal would still require Metal to claim the recommended operations. Table 8.1 refuses that claim for `exp`. For `rsqrt`, Table 8.1's "Correctly rounded" matches the recommended *accuracy* form and still does not cite IEEE 754 the way the `fma` row does; Gap 4 of the accuracy record additionally leaves the rounding mode unfixed. The residual without a Clause 9 exceptional-value re-read is therefore inert for these three rows: disagreement in that table could not bind Metal without a Metal sentence this census did not find.

No access-control barrier was hit. The IEEE document is already purchased and quoted; this session declined to hunt the purchased file on disk because the retained quotes plus Table 8.1 already reject applicability.

### What was not treated as a source

- LLVM AIR or `air.exp.f32` implementation notes. No retained AIR specification states exceptional values, and an implementation note would be empirical or informal, not one of the three hard classes.
- Device observations, including [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) finding 35 (`precise::exp` of both signed zeros returns exact `1.0` on one Apple9 runtime row). Finding 35 is ordinary-domain exactness at zero, not an exceptional-value rule, and it is a bounded measurement.
- The three retained exceptional corpora in `crates/tiler-reference`. They remain `EmpiricalQualification` and are described per row below rather than promoted.

## The three hard classes, applied once so the rows can reuse the rejection

**Sound proof.** There is no formal model of `air.exp.f32` or `air.rsqrt.f32` in this repository, and none was found in the retained Metal text. A pencil argument from IEEE 754 recommended operations is not a proof about Metal, for the reasons above.

**Exhaustive finite evidence.** Binary32 is finite, so in principle every encoding can be evaluated. That would be a named device, named toolchain, named compiler-mode row — a new bounded claim under this ticket's own stop, "A new compiler/toolchain/device environment creates a new bounded claim; it does not inherit this evidence." It would not be a guarantee about the Metal shading language the installed rows are attributed to. The retained corpora are not exhaustive in any case: they are enumerated boundaries plus a 4,096-argument walk (SiLU), a 512-argument sweep plus named rows (RMSNorm), and named rows at reduced extents 0, 2, 3, 4, and 10 (softmax). ADR 0042 forbids renaming a sample, however carefully chosen, as exhaustive evidence. This session takes no device measurement and does not reclassify those corpora.

**Applicable normative guarantee.** An applicable guarantee would have to prescribe, for the named function under the governed precise flags, each of the four `ExceptionalValueContract` rules that occurrence states. The retained Metal text supplies none of the four for `exp` or `rsqrt`. OpenCL's C99 incorporation is not Metal's. IEEE 754 recommended operations are not Metal's. The `fma` citation of IEEE 754-2008 is the existence proof that Apple writes that sentence when it means it.

## Row 1 — SiLU `exp` at F32, precise Metal

**Contract owed.** `silu_f32_exponential_exceptional_contract`: NaN reference → canonical NaN; infinite reference → signed infinity (`exp(+inf) = +inf`, which SiLU reaches as `Exp(-x)` at `x = -inf`); domain error → canonical NaN (vacuous for values binary32 can produce); finite overflow → signed infinity. The last rule is live: the ordinary domain closes at `0x42b17217` because that is the last binary32 argument whose exponential is finite, and the activation's `-88.73` band is the overflow case the contract comment names — "`1 + inf` is `inf` and a finite negative divided by `inf` is exactly `-0.0`".

**Hard class.** `Unknown`.

**Why the three classes fail here, specifically.** No Metal sentence prescribes `exp(+inf)`, `exp(NaN)`, or `exp` of a finite overflow. §8.1's INF-is-supported clause makes `+inf` a *possible* outcome of that overflow, which is what the accuracy record already uses to explain why the `-88.73` band is not a flush case; it does not make `+inf` the *required* outcome. Canonical-NaN payload is a Tiler identity dimension (`NanReferenceRule::CanonicalNan` is "the operation's canonical arithmetic NaN payload") and Metal never names a payload. Exhaustive evaluation of the overflow band on one GPU would still be a device-scoped measurement.

**Retained empirical record, not reclassified.** `metal_f32_exceptional_value_evidence` is `EmpiricalQualification` over "the boundary corpus of `crates/tiler-reference/src/silu/tests.rs`: fourteen enumerated binary32 arguments plus a contiguous 4,096-argument walk across the overflow band", toolchain "Apple metal version 32023.883, macOS 27.0, `-std=metal4.0`", device "Apple M4 Max", digest `corpus:silu-f32-boundary-v1`. It remains qualification evidence. `ElementaryRealization::discharge` already reports that it cannot discharge, and `assess_elementary_accuracy` refuses admission on that half.

**Unsupported close-out.** This row is unsupported for hard Metal-target admission. The governed profile continues to declare no elementary row for `tiler::silu-f32@1`.

## Row 2 — RMSNorm `rsqrt` at F32, precise Metal

**Contract owed.** `rms_norm_f32_rsqrt_exceptional_contract`: NaN reference → canonical NaN; infinite reference → signed infinity, "the `+inf` of `1/sqrt(+0)`"; domain error → canonical NaN on a negative argument; finite overflow → signed infinity. The header comments state the infinite-reference and domain-error cases are unreachable while `eps` is positive and the input is a sum of squares, and that the rules are stated because `refines` compares whole contracts.

**Hard class.** `Unknown`.

**Why the three classes fail here, specifically.** Table 6.4's `rsqrt` description is "Compute inverse square root of x." Table 8.1's `rsqrt` entry is "Correctly rounded" and is an ordinary-domain accuracy claim — the bound half already records it as a `NormativeGuarantee` of a *faithful* result set, after Gap 4's unfixed rounding mode. Correct rounding of a defined finite reference does not prescribe `rsqrt(-1)`, `rsqrt(NaN)`, `rsqrt(+0)`, or `rsqrt(-0)`. No Metal sentence cites IEEE 754 for `rsqrt` the way the `fma` row does. Unreachability of the negative and zero cases on this operation's admitted inputs does not discharge the function-level rules the contract still states.

**Retained empirical record, not reclassified.** `metal_f32_normalization_exceptional_value_evidence` is `EmpiricalQualification` over "the bounded corpus of `crates/tiler-reference/src/rms_norm/tests.rs`: the retained worked example, a zero row, a signed-zero row, a subnormal row, a row above the squaring-overflow threshold, both workload extent classes at 1024 and 128, and a contiguous 512-argument sweep of the reciprocal square root", same toolchain and M4 Max device, digest `corpus:rms-norm-f32-boundary-v1`.

**Unsupported close-out.** This row is unsupported for hard Metal-target admission. The governed profile continues to declare no elementary row for `tiler::rms-norm-f32@1`.

## Row 3 — softmax `exp` at F32, precise Metal

**Contract owed.** `softmax_f32_exponential_exceptional_contract`: the same four rule tags as SiLU's exponential. The *live* rule is NaN reference → canonical NaN, "the rule a poisoned row travels: `s_i - m` is NaN whenever either operand is". Infinite-reference and finite-overflow are vacuous because the ordinary domain closes at `+0.0` and `e^t <= 1` everywhere on it. Domain error is vacuous for values binary32 can produce.

**Hard class.** `Unknown`.

**Why the three classes fail here, specifically.** Vacuity of overflow does not manufacture a NaN-payload rule. Metal still does not prescribe `exp(NaN)` or name a canonical payload. Sharing Table 8.1's `exp <= 4 ulp` with SiLU — already shared as bound evidence — does not share an exceptional-value guarantee that neither occurrence has. Softmax's underflow band reaches §8.5's undefined-sign permission, which is a reason a golden must not pin `0x00000000` there, not a discharging signed-zero contract.

**Retained empirical record, not reclassified.** `metal_f32_softmax_exceptional_value_evidence` is `EmpiricalQualification` over "the bounded corpus of `crates/tiler-reference/src/softmax/tests.rs`: the retained worked example, a row of equal large scores, the underflow band at 87, 88, and 104 below the maximum, a fully masked row under both mask conventions, a NaN row, a signed-zero row, and the empty reduced axis, at reduced extents 0, 2, 3, 4, and 10", same toolchain and M4 Max device, digest `corpus:softmax-f32-boundary-v1`.

**Unsupported close-out.** This row is unsupported for hard Metal-target admission. The governed profile continues to declare no elementary row for `tiler::softmax-f32@1`.

## Reconsideration triggers

Each trigger is independent per row. Firing one does not fire the others.

1. **Normative — a new Metal language revision, or an Apple citation this census missed, prescribes the four rules for that function under the governed precise flags.** Reproduce the absence first: `pdftotext -layout docs/research/apple-targets/sources/<pdf> -` then search for `F.9`, `Annex F`, `shall conform`, and `Edge case behavior is per the IEEE`. Today those searches find the `fma` citation and no `exp` / `rsqrt` citation. A hit that names `exp` or `rsqrt` and states NaN, infinity, domain-error, and finite-overflow results is a candidate `NormativeGuarantee` for that row only, still scoped to the named revision, digest, and compiler-mode. A new PDF is a new row; it does not inherit this close-out or the bound-half derivation.

2. **Normative — Metal cites IEEE 754 recommended operations, or C99 F.9, for the precise math functions the way it already cites IEEE 754-2008 for `fma`.** That would reverse the "subset" reading for the cited function. It would still need a metric / rounding-mode derivation before the *bound* half moved, which is not this record's job.

3. **Exhaustive finite — a named device, toolchain, compiler-mode, and Metal language row evaluates every binary32 encoding of that subordinate function, against a named oracle, and the complete result set refines the occurrence's exceptional-value contract.** That evidence would discharge only that exact implementation row. It would not become a Metal-language guarantee and would not be inherited by another compiler, OS, or GPU. ADR 0042 would classify it `ExhaustiveFinite` only if the universe is the complete admitted finite input space rather than a sample. The current corpora do not meet that bar.

4. **Sound proof — a machine-checked or accepted pencil proof that the selected intrinsic, under the governed flags, produces exactly the contracted exceptional set.** There is no such proof today.

5. **Semantic-contract change — a separately accepted numerical decision narrows what the occurrence must prove.** This ticket's own stop forbids doing that here. A narrowed contract is a different requirement and a different evidence question.

A trigger that fires with only a bounded device sample, a vendor blog, an OpenCL citation, or a re-label of an existing empirical digest does **not** fire. Those remain `EmpiricalQualification` or `Unknown`.

## Crate-admission remainder

The public declaration path already exists and is out of this wave's edit set. `TargetProfileBuilder::declare_elementary_realization` stores a whole `ElementaryRealization`. `TargetProfile::declared_elementary_realizations` is a borrowed view of those stored rows. `TargetProfile::governed` currently declares none — `a_profile_declaring_no_elementary_row_encodes_like_a_build_without_the_family` asserts `governed.declared_elementary_realizations().is_empty()`. `installed_elementary_realizations` still *records* the three Metal rows with empirical exceptional halves via `ElementaryRealization::recorded`; `ElementaryRealization::declare` and `assess_elementary_accuracy` both require `require_discharged_halves`, so those recorded rows cannot be admitted.

This wave does not construct a discharging `ConformanceEvidence` for any of the three exceptional halves, and it does not call the public declaration path. The fail-closed outcomes stay as they are: a profile that declares nothing refuses `no-installed-realization`; a profile that declared one of the current empirical rows would refuse `undischarged-evidence` naming the exceptional half and `EmpiricalQualification`.

When a trigger above produces a hard-class record, a later ticket may build that `ConformanceEvidence`, pass it through `ElementaryRealization::declare` with the already-discharging bound half, and declare the one supported occurrence on the governed profile. Until then the three rows stay absent from the governed profile. Installing an `Unknown` or empirical exceptional half through the public path would be a regression against [`require-both-elementary-evidence-halves-before-target-admission`](../../../tickets/require-both-elementary-evidence-halves-before-target-admission.md), not an admission.

## The checks, and that they can say no

Each check was run this session against a case that must fail, and failed.

1. **The `fma` contrast is content-sensitive.** `pdftotext -layout` of MSL 4.1, then a search for `Edge case behavior is per the IEEE`, hits the `fma` row on page 206 and does not hit `exp` or `rsqrt`. Rewriting the extracted `fma` sentence to name `exp` would create a hit this check currently lacks. A check that reported "no IEEE edge-case citation anywhere" would have been wrong in the direction that hides Apple's own contrast.

2. **The C99-incorporation absence is content-sensitive.** The same extraction searched for `\bF\.9\b|Annex F|\bC99\b|\bC11\b` and for `shall conform`; both are empty on both retained Metal PDFs. The OpenCL fetch of the same day contains both "shall conform to sections F.9 and G.6 of the C99 Specification" and the §7.5.3 flush paragraph Metal did keep. A checker that treated the shared flush paragraph as the incorporation sentence would have reported a Metal C99 guarantee this text does not contain.

3. **The three exceptional records are still empirical.** Inspect `metal_f32_exceptional_value_evidence`, `metal_f32_normalization_exceptional_value_evidence`, and `metal_f32_softmax_exceptional_value_evidence`: each calls `ConformanceEvidence::new` with `ConformanceEvidenceClass::EmpiricalQualification`. Changing any one to `NormativeGuarantee` without a Metal sentence this record does not have would be the re-label ADR 0042 and the evidence-halves ticket both forbid.

4. **The governed profile still declares no row.** `TargetProfile::governed().declared_elementary_realizations()` is empty. A later admission ticket that installed a discharging row would move this check; this record must not.

## What this record does not decide

- The ordinary-domain bound half. Table 8.1's `exp <= 4 ulp` and `rsqrt` correctly-rounded-as-faithful remain the accuracy record's and `metal_f32_exponential_bound_evidence` / `metal_f32_reciprocal_square_root_bound_evidence`'s subject.
- Whether Tiler should narrow any exceptional-value contract so a weaker Metal sentence would suffice. That is a numerical decision, not an evidence repair.
- Whether a future device-exhaustive binary32 sweep should be taken, on which host, or against which oracle.
- Anything about `half` or `bfloat` elementary functions.
- Installation of any row through `declare_elementary_realization`.
