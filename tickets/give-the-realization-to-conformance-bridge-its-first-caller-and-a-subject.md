---
id: give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject
title: Give the realization-to-conformance bridge its first caller and a subject
status: done
priority: p2
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, wire-the-bf16-reference-to-the-realization-it-is-told, apply-the-declared-numerical-conformance-on-every-reference-evaluation-path]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, fail-closed]
---
## The gap

**Fact — the checked bridge has no caller.** `ReferenceNumericalConformance::from_realization` (`crates/tiler-reference/src/conformance.rs:166`) is the designed, documented path from a region's declared `NumericalRealization` to the conformance a reference evaluation runs under. Three separate module headers cite it as the bridge — `standard.rs:32`, `registry.rs:190`, `evaluate.rs:71`, `oracle.rs:1364`. **Nothing calls it.** Verified 2026-08-07 by the coordinator: `grep -rn "from_realization" crates/ prototypes/` returns only doc-comment references. Re-verified by the worker at base `43e9b9af`.

**Correction, 2026-08-07 (worker) — the rest of that Fact was false, and the false half named the missing caller.** The ticket said "every construction site in `crates/` and `prototypes/` is `ReferenceNumericalConformance::strict()` or a test's `new()`". It is not. `crates/tiler-conformance/src/bf16_vertical.rs:463-466` is a **non-test** `declared_conformance()` that hand-rolls `ReferenceNumericalConformance::new(contract.input_subnormals(), contract.result_subnormals())` from the same `NumericalContract` accessors its sibling `declared_realization()` (`:433`) builds the region's `NumericalRealization` from. That function *is* the first real caller this ticket asks for: a region's declared realization and the oracle's conformance, sourced from one contract, transcribed twice instead of bridged once.

**Correction — the stated consequence was also false.** The ticket said "a region declaring a flushing realization is compared against a preserving oracle today". `bf16_vertical.rs`'s own module header (`:135-146`) states the opposite and the code does what it says: the comparison is performed against the reference evaluated under `declared_conformance()`, the **flushing** reading, because the measured macOS row flushes and a preserving oracle could not agree with it. What is actually bypassed is the *bridge*, not the flush — so `from_realization`'s six refusals never run on that route, and a permissive contract would reach the oracle as two silently-ignored subnormal modes rather than as a typed refusal.

**Fact — the bridge discards the format subject.** `from_realization`'s destructuring reads `canonical_arithmetic_nan_bits: _` (`:171`). The resulting object is structurally format-agnostic while `registry.rs:181` documents it as being for "a capability that performs host binary32 arithmetic". That mismatch is the boundary at which the subject is lost. **Both halves verified true at the base.**

**Correction — `canonical_arithmetic_nan_bits` is not "the one field identifying the region's arithmetic type".** It carries the canonical arithmetic *NaN payload* of that type, zero-extended (`crates/tiler-ir/src/schedule/numerics.rs:211-238`). The payload determines the type only over the formats that declare one — `f32` and `bf16` do, `f16` and `f64` do not — so it is a checkable *corroboration* of a stated subject, not a total function onto `ArithmeticType`. That is how it is now used.

## What this owes

- **The first real caller**, so a region's declared realization reaches the evaluation performed under it, and the existing refusals become reachable rather than merely tested.
- **The subject carried across**, drawn from the region rather than from a new field on `NumericalRealization` — the arithmetic type is already a total function of the region's scalar program (`region_arithmetic_type`, `crates/tiler-ir/src/schedule/model.rs:1333`).

  **Correction, 2026-08-07 (worker) — `region_arithmetic_type` is `pub(super)` and cannot be called from `tiler-reference` or `tiler-conformance`.** The function is total, as stated; the ticket's implied route to it is not open. The public route is `RealizationWitness::of(&region)` (`crates/tiler-ir/src/schedule/witness.rs:93`), whose `realization()` and `accumulation()` hand back both arguments from one object — and `accumulation()` returns `region_arithmetic_type(program)` for every topology that does not declare a width, while the intrinsic verifier refuses a declared width that differs from it (`witness.rs:209-235`, `builder.rs:1516`, `:1640`). So the subject is an explicit `ArithmeticType` argument to `from_realization`, and the two are read off one witness rather than assembled from two sources.
- **A capability-side agreement check**: a capability applies a conformance only when its subject matches the capability's own format, and returns a typed refusal otherwise. **This is the obligation Tom's 2026-08-07 arm-A decision deferred to here** — see [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md). It is placed here rather than in the BF16 family because this is where the subject is lost, and because unlike the mixed-width refusal that decision rejected, **this check is reachable**: handing a BF16 capability an `f32`-derived conformance is constructible in a test and can be watched failing.

## Why this matters beyond BF16

`registry.rs:181` divides capabilities into those performing host binary32 arithmetic, which must consult the conformance, and those performing no host arithmetic, which have nothing to read. The BF16 family is neither — it produces arithmetic results over BF16's value set by exact rational arithmetic. That third case is currently undocumented, and the subject is what distinguishes it. Any future non-binary32 family lands in the same gap.

## Required evidence

- A region declaring a non-strict realization is evaluated under it, end to end, rather than under `strict()`.
- At least one existing `from_realization` refusal watched firing through the new caller, proving the path reaches them.
- The subject mismatch watched failing: a capability handed a conformance resolved for another format refuses, with the refusal observed before restoration.
- Populations counted, so a path that stopped being exercised cannot look green.

## Closes when

A declared realization reaches the evaluation performed under it, the conformance carries its arithmetic subject, a mismatch is a typed refusal observed failing, and `registry.rs`'s two-case division is restated to cover a family that performs non-binary32 arithmetic.

## Worker outcome, 2026-08-07

Landed inside `crates/tiler-reference/**` only. The public boundary below is a **draft under ADR 0075** until Tom accepts it.

**The subject is in the bridge.** `ReferenceNumericalConformance` carries a `ConformanceSubject`, and `from_realization` takes the region's `ArithmeticType` beside its realization. The previously discarded `canonical_arithmetic_nan_bits` is now *read*: the stated subject is refused when the realization's own declared payload contradicts it (`DeclaredNanPayloadMismatch`), and a format this reference performs no arithmetic in is refused before anything else (`ArithmeticNotEvaluable`, `f16` and `f64`). Both refusals precede the six transform refusals, which are unchanged.

**The agreement check is capability-side and reachable.** `ReferenceEvaluationRequest::conformance` and `ScalarReferenceRequest::conformance` are **replaced** by `conformance_for(ArithmeticType)`, so every capability that reads the contract names its own format. Nine reading sites converted (`standard.rs` ×2, `silu.rs`, `rms_norm.rs`, `softmax.rs`, `quantization.rs`, `contraction.rs`, `oracle.rs`, `bf16.rs`). Handing the BF16 family an `f32`-subjected conformance is a typed `ReferenceOperationError::ConformanceSubject`, observed through the whole evaluator.

**`registry.rs`'s two-case division is restated as three**, on `conformance_for` and in `standard.rs`'s header: host-binary32 arithmetic, arithmetic in another format realizing the same declared dimensions over its own value set (the BF16 family), and no arithmetic at all.

**Named boundary, not closed.** `strict()` and `new()` keep their signatures and produce `ConformanceSubject::Unstated`, because `crates/tiler-conformance` calls both and is outside this ticket's scope. So the guarantee is: *a conformance drawn from a declared realization is refused by a capability of another format*. An unsubjected conformance still reaches every capability. That is stated on `ConformanceSubject`, on `conformance_for`, and in `conformance.rs`'s header rather than left to be discovered.

**Not delivered — the production first caller is out of scope.** `tiler-reference` names no region type: it imports nothing from `tiler_ir::schedule` but the numerics vocabulary, and neither `ReferenceEvaluator` nor `IndexRegionEvaluator` takes a scheduled region. The site where a declared realization and a reference evaluation actually meet is `crates/tiler-conformance/src/bf16_vertical.rs`, which this ticket's scope (`implementation/reference`) excludes. Filed as [`route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`](route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge.md). What *is* delivered here is the end-to-end path exercised in-crate: `bf16::tests::a_declared_flushing_realization_reaches_the_evaluation_performed_under_it` carries a region's declared flushing realization through `from_realization` into `ReferenceEvaluator::under` and out of the registered BF16 capability, and asserts the preserving answer absent on all seven counterexamples.

**Also out of scope and left stale:** `docs/correctness-and-testing.md:55` still says the window "is real and currently **unreachable**" and that no capability checks its conformance's subject; `crates/tiler-conformance/src/bf16_vertical.rs:148-157` still says `from_realization` "discards the format its realization was stated about". Both are now false. Neither `docs/**` nor `crates/tiler-conformance/**` is in scope; the follow-up ticket carries them.

## Graph maintenance

Filed 2026-08-07 by the coordinator while assessing the BF16 subnormal carrier fork for Tom. The unused bridge was found by reading rather than reported by any ticket, and it is the reason neither arm of that fork would have delivered its stated outcome alone.
