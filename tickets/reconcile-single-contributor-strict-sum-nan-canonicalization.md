---
id: reconcile-single-contributor-strict-sum-nan-canonicalization
title: Reconcile single-contributor strict-serial-sum NaN canonicalization across the three implementations
status: in-progress
priority: p0
dependencies: []
related: [register-governed-scalar-reference-evaluation]
scopes: [implementation/reference, implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, reference, correctness, milestone-0b]
claimed_from: todo
assignee: agent-reconcile-single-contributor-strict-sum-nan-canonicalization
lease_expires_at: 1784932574
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
