---
id: state-a-subject-on-the-contraction-publication-path-s-reference-oracle
title: State a subject on the contraction publication path's reference oracle
status: in-progress
priority: p2
dependencies: []
related: [route-the-realization-conformance-half-into-the-conformance-crate, give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-state-a-s
lease_expires_at: 1786158472
---
## The asymmetry

`crates/tiler-conformance/src/publication/proof.rs` computes every published expectation through `ReferenceEvaluator::standard()` → `under(registry, strict())`. `strict()` produces **`ConformanceSubject::Unstated`**, which reaches every capability **unchecked** — while the artifacts those expectations are compared against are compiled under `FLUSH_SUBNORMALS_TO_ZERO_F32`.

So the oracle is not told the contract the device half runs under. That is the same window `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject` closed for the BF16 vertical, still open on the contraction path.

**Unobservable on the current operands, and that is not a reason to leave it.** The probe stream is `m·2⁻²⁴`, so no subnormal arises and the two readings agree on every value the corpus contains. The agreement is a property of the operands, not of the contract — a corpus that grew a subnormal would silently compare a flushing device against a preserving oracle, which is exactly the failure the subject exists to refuse.

## Why this is a design step and not a rename

Routing it through `from_realization` needs a **`VerifiedScheduledRegion`**, because that is what `RealizationWitness::of` requires — and the contraction publication path does not hold one. The BF16 vertical could do this because it assembles its region directly; the contraction path receives a plan.

So the work is to establish **where the subject comes from on a plan-derived route**, which is a question about what the publication path is handed, not a substitution at the call site. Answer that before writing anything.

`strict()` and `new()` keeping `Unstated` is deliberate and stated — see the bridge's own record. This ticket does not change them; it stops this route relying on them.

## Read in full first

`crates/tiler-conformance/src/publication/proof.rs`; `crates/tiler-conformance/src/bf16_vertical.rs`'s `conformance_of`, which is the closed case to model against; and `crates/tiler-reference/src/conformance.rs` for what `from_realization` requires and refuses. `crates/tiler-reference/**` is **out of scope** — read it, never edit it; if the route needs a signature change there, stop and report, exactly as the two previous workers on this thread did.

## Closes when

The contraction publication path's oracle carries a subject derived from what the plan declares, or the reason it cannot is recorded with the evidence — and a test watches the bridge's refusal **fire** on this route, perturbing the subject rather than an assertion.

## Per-Fact audit, 2026-08-07 at base `68f1ced6`

- **`proof.rs` computes every published expectation through `ReferenceEvaluator::standard()` → `under(registry, strict())`** — *verified*. `reference_bits` called `ReferenceEvaluator::standard()`; `evaluate.rs`'s `standard()` is `FrozenReferenceRegistry::standard().map(Self::new)` and `new` is `Self::under(registry, ReferenceNumericalConformance::strict())`. It is the only oracle on the path: `encoded` is the sole caller and `publish_member` its sole caller.
- **`strict()` produces `ConformanceSubject::Unstated`, which reaches every capability unchecked** — *verified*. `strict()` delegates to `unsubjected`, which sets `subject: ConformanceSubject::Unstated`; `checked_for` returns `Ok(self)` on that arm for every `ArithmeticType`.
- **The artifacts are compiled under `FLUSH_SUBNORMALS_TO_ZERO_F32`** — *verified*, and **wider than stated**: `publication.rs`'s `CONTRACT` governs *every* published member, the six serial-sum ones included, not only the contraction. The fix therefore lands on `proof::encoded`, which all three families share.
- **The probe stream is `m·2⁻²⁴`, so no subnormal arises** — *verified for the L3 family*, and the reasoning is stronger than "no subnormal operand": every product and every exact partial sum is an integer multiple of `2⁻⁷²`, which is normal, so none arises anywhere in the fold.
- **"…and both readings agree on every value the corpus holds"** — *true, but the stated reason is imprecise*. The adversarial corpus **does** hold subnormal operands, which the Fact's reasoning does not cover: `OPERAND_CASES`' `signed-zero-and-subnormal` and `CONTRACTION_CASES`' `negative-zero-fold` each carry `0x00000001`. They agree anyway because the flush is absorbed downstream (`1.0 + 1e-45` rounds to `1.0`; `-0.0 + 1e-45 + 1.0` reaches `1.0` either way), not because no subnormal is present. `tests::stating_the_packaged_contract_moves_no_published_expectation` now counts that population at 2 rather than leaving it to argument.
- **Routing through `from_realization` needs a `VerifiedScheduledRegion`, which the publication path does not hold** — *verified*, and *the conclusion drawn from it is false*. `RealizationWitness::of` does require one, but `from_realization` does not: it takes `(&NumericalRealization, ArithmeticType)`, and a plan can supply both. See the Outcome.
- **`strict()` and `new()` keeping `Unstated` is deliberate and stated** — *verified*, in `conformance.rs`'s module header and on both constructors. Unchanged here.

## Outcome, 2026-08-07

**The subject does come from somewhere on a plan-derived route**, without a region and without touching `crates/tiler-reference/**`. `proof::conformance_of(plan)` reads both of `from_realization`'s arguments off the plan:

- the realization from `PlanAlternative::kernels()[..].numerical()` — `VerifiedKernel` preserves the scheduled region's own `index.numerical`, so this is the plan's choice at the sites its contract left free. Every packaged kernel is compared and a disagreement is refused, because a member is one sidecar over one program.
- the subject from `PlanAlternative::delivered_realization().scalar_arithmetic()` — ADR 0076's evidence, whose subject table is materialized from the selected contract's own arithmetic type, read through `ScalarArithmeticSubject::arithmetic()`. Exactly one subject is required rather than assumed.

The bridge then cross-checks the two against each other through the realization's declared canonical arithmetic NaN payload, so the subject is an agreement rather than an assertion.

**No published byte moved**, and that is measured rather than argued: all 8 gate members and all 4 `#[ignore]`d prefill cells published through the changed encoder, agreed bit-for-bit with the device, and carried their unchanged retained digests (`79810ce4…`, `1c54f5cd…`, `eb382840…`, `124571de…`, `b99eff90…`).

Three device-free tests added in `publication::proof`, each watched failing deliberately:

- `the_published_oracle_carries_the_packaged_plans_own_contract` — subject `Arithmetic(F32)`, both dimensions the contract's flush, realization key `CONTRACT.key()`. Fails `Unstated != Arithmetic(F32)` when `conformance_of` returns `strict()`.
- `a_subject_the_packaged_realization_contradicts_is_refused_on_this_route` — states `Bf16` against the `f32` plan and reads back `DeclaredNanPayloadMismatch { arithmetic: Bf16, declared: 0x7fc00000, expected: 0x7fc0 }`; `F16`/`F64` give `ArithmeticNotEvaluable`. Fails when the stated subject is ignored.
- `stating_the_packaged_contract_moves_no_published_expectation` — 20 case comparisons over a counted subnormal population of 2, plus a probe (the least positive subnormal through the singleton reduction) where the two readings must differ. Reverting the evaluator to `ReferenceEvaluator::standard()` fails exactly at that probe.

`portability.rs`'s `DEVICE_FREE_TEST_FLOOR` moved 64 → 67 with its reasoning: population 65 → 68, the same three, preserving the two-test sensitivity.

**Two corrections recorded rather than worked around.** `NUMERICAL_IDENTITY` is *not* the realization's `profile_key` and cannot be derived from one — it is a governed name in the sidecar's identity domain, while the realization carries the compiler's structural key `tiler.contract.f32.v2.037fc0000001…`; an assertion that they are equal was written, failed, and the claim was corrected at both sites. And the `#[test]` attribute must not be spelled in `portability.rs`'s prose: a first draft of the floor's doc did, inflating the census by one, which is the trap that module's own header names.
