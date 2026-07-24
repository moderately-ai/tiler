---
id: reconcile-single-contributor-strict-sum-nan-canonicalization
title: Reconcile single-contributor strict-serial-sum NaN canonicalization across the three implementations
status: done
priority: p0
dependencies: []
related: [register-governed-scalar-reference-evaluation]
scopes: [implementation/reference, implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, reference, correctness, milestone-0b]
---
A `tiler::strict-serial-sum-f32@1` reduction whose reduced domain holds **exactly one** contributor is computed three ways, and the normative oracle is the odd one out.

**Measurement (`crates/tiler-reference/tests/governed_scalar_reference.rs::a_lone_non_canonical_nan_contributor_diverges_between_the_two_oracles`, macOS arm64, pinned nightly).** For a `[1]`-shaped input reducing axis 0, with input bits `0x7fc01234`, `0xffc00000`, or `0x7f800001`, the index-region oracle running the governed lowering shape returns the input bits unchanged and `ReferenceEvaluator` returns `0x7fc00000`. The two disagree on every non-canonical NaN payload and agree on every other payload tested, including both signed zeros, both infinities, the least subnormal, and the canonical NaN itself.

**Fact — what each implementation does.**

- `tiler_reference::strict_sum` (`crates/tiler-reference/src/lib.rs`) seeds its accumulator with the first contributor and then writes `f32_element(canonicalize_arithmetic_f32(accumulator.unwrap_or(0.0)))`. That final canonicalization is unconditional, so a fold that performed *zero* additions still reports the canonical arithmetic payload.
- `tiler_compiler::governed::GovernedStrictSerialSumF32` returns `seed` directly when `plan.reduced_points == 1`. The seed is a tensor read; no scalar operation is applied, so nothing canonicalizes.
- `tiler_ir::kernel::lower::emit_reduction` stores `seed` unchanged when `plan.contributors == 1`, and says so in a comment: "A single contributor is the whole strict-serial result".

**Inference — the oracle is the one that deviates.** The governed operation facts for `multiply-f32`, `add-f32`, and `strict-serial-sum-f32` all carry `CANONICAL_F32_ARITHMETIC_NAN_BITS` as a fact *of an arithmetic result*. A zero-step fold produces no arithmetic result, so the fact does not reach it. `ReferenceEvaluator::evaluate` documents the rule the other two implement — "a strict left fold ... starts with the first contributor" — and then canonicalizes a value that never entered an arithmetic operation. That is the reading this ticket recommends, but it is not the only defensible one and the decision is not this ticket's to assume.

**This is live in the compiled product, not hypothetical.** `crates/tiler-compiler/src/physical.rs` builds a bare `ScalarProgram::StrictSerialSum` for the materialized alternative, and a reduced axis of extent 1 is admissible — `pipeline`'s own `[4, 1]` conformance fixture is one. So a compiled materialized kernel and `ReferenceEvaluator` disagree today on a program the compile path accepts.

**Why no existing test catches it.** `pipeline::tests::structured_fused_body_interpreter_matches_reference_evaluator` reduces a `[4, 1]` input containing `f32::NAN`, and `f32::NAN` *is* `CANONICAL_F32_ARITHMETIC_NAN_BITS`, so canonicalizing it is a no-op. The fused fixture also carries a scale/bias prologue, whose `emit_scale_bias` canonicalizes the seed, masking the pure `StrictSerialSum` path. `crates/tiler-reference/src/lib.rs::strict_sum_preserves_non_nan_singletons_and_canonicalizes_nan_results` deliberately pins the current oracle behaviour with `0x7fc01234`, so this is a decided behaviour on one side and a decided behaviour on the other, not an oversight on either.

**What closes this.** One decision, applied to whichever side moves, plus the tests that pin it:

1. Decide whether a zero-arithmetic-step strict serial sum canonicalizes its NaN payload. Record it as a durable numerical contract in `docs/numerical-semantics.md` and, if the answer changes `strict_sum`, supersede the pinned expectation in `strict_sum_preserves_non_nan_singletons_and_canonicalizes_nan_results` explicitly rather than editing it silently.
2. Make all three implementations agree, then convert `a_lone_non_canonical_nan_contributor_diverges_between_the_two_oracles` from a divergence pin into an agreement assertion.
3. Add a non-canonical NaN to a single-contributor conformance vector in `pipeline::tests` **without** a scale/bias prologue, so the materialized `ScalarProgram::StrictSerialSum` path is compared against the oracle rather than only the fused one.

Requires `implementation/ir` and `implementation/compiler` alongside `implementation/reference`, because whichever side moves, the conformance vector that keeps it honest lives in `tiler-compiler`.

## Outcome

**The decision was not this ticket's to make: the contract had already made it, the other way round from this ticket's own recommendation.** `docs/numerical-semantics.md` requires strict `f32` Sum to apply the canonicalization "at its result boundary even when the contributor sequence is a singleton", and states exactly why: "The redundant result-boundary rule prevents an uncombined input payload from leaking through an arithmetic reduction." ADR 0055 says the same, "including singleton results". A lone contributor *is* an uncombined input payload, so the rule is not merely applicable to this case — this case is the one it was written for.

That falsifies this ticket's **Inference — the oracle is the one that deviates**. `tiler_reference::strict_sum` was already correct, and the two implementations that returned the seed unchanged were realizing a rule no governed record states. The ticket's framing — "a decided behaviour on one side and a decided behaviour on the other" — was accurate about the code and wrong about the contract, because neither side's decision had been checked against it.

**Consequently, item 1 needed no edit.** The durable contract already states the rule, and since `strict_sum` does not move, `strict_sum_preserves_non_nan_singletons_and_canonicalizes_nan_results` keeps its pinned expectation with nothing to supersede. This ticket holds no `contracts/*` scope and needed none.

**What moved (item 2).** Both non-conforming implementations now canonicalize at the result boundary, through a new governed scalar operation rather than through arithmetic:

- `tiler.scalar::canonicalize-nan-f32@1` was added to `FrozenScalarRegistry::standard` as a unary conversion — the index-region counterpart of the structured kernel's existing `ConvertOp::CanonicalizeF32Nan` — with an executable oracle in `tiler-reference`. It is deliberately *not* arithmetic: every arithmetic realization available here, adding the `+0.0` identity in particular, would turn an observable `-0.0` into `+0.0`, which `a_single_contributor_agrees_on_every_payload_but_a_non_canonical_nan` pins in the other direction. `StandardF32Binary` became `StandardF32Homogeneous` because the operand-type rule is arity independent.
- `GovernedStrictSerialSumF32` applies it on the `reduced_points == 1` path, and the capability's `emitted` set grew to include it. That set is the union over shapes, not what any one occurrence reaches — an empty reduced domain reaches only the identity constant, a lone contributor only the canonicalization, a longer one the add — which is exactly why refinement's conformance rule is containment rather than equality.
- `tiler_ir::kernel::lower::emit_reduction` applies it on the single-contributor path when there is no prologue. The three other boundaries were checked rather than assumed: the fold applies `CanonicalizeF32Nan` after each combine, `emit_scale_bias` ends in one so a prologued seed is already canonical, and the zero-contributor path yields the `+0.0` identity constant, which is not a NaN.

`a_lone_non_canonical_nan_contributor_diverges_between_the_two_oracles` is now `a_lone_non_canonical_nan_contributor_canonicalizes_in_both_oracles`, asserting agreement at `CANONICAL_F32_ARITHMETIC_NAN_BITS` for all three payloads, and its `serial_sum_region` helper mirrors the new lowering.

**Item 3.** `a_singleton_reduction_canonicalizes_a_lone_non_canonical_nan` interprets the *materialized* alternative's bare `StrictSerialSum` kernel — no scale/bias prologue — over `[0x7fc01234, 0xffc00000, -0.0, least-subnormal]`, and compares it against `ReferenceEvaluator` on the same program. It pins both halves of the rule at once: both NaN payloads are rewritten and `-0.0` and the least subnormal survive, which is what distinguishes a conversion from an addition.

**One fixture was rebaselined deliberately.** `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` hard-codes a request-subject digest, which moved from `be70237691f8f507` to `315e14544407d942`. The request subject covers the frozen scalar and lowering-capability authorities, so admitting a governed scalar operation *must* move it — a digest that survived would mean the subject was incomplete. The comment beside it records that reasoning rather than only the new value.

**Verification:** `cargo nextest run --workspace --no-fail-fast` — 599 passed, 0 skipped; `cargo clippy --workspace --all-targets` clean; `cargo fmt --all --check` clean after formatting; `cargo test --doc --workspace` — all doctests pass.

**One public-surface addition, flagged rather than buried.** `tiler_ir::index::canonicalize_nan_f32_scalar_op` is a new `pub fn`, exactly mirroring its three existing siblings `constant_f32_scalar_op`, `multiply_f32_scalar_op`, and `add_f32_scalar_op` in the same already-public module. Under accepted ADR 0075 this is not an always-ask category — not a new publicly reachable namespace, not a new public trait, not a breaking change to an existing signature, and not a `pub(crate)` promotion — so it merged under the coordinator's conditional authority. It is recorded here because it is the one item in this change that adds public surface, and Tom may want the shape reviewed even though the policy does not require it.
