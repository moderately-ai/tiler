---
id: pin-the-strict-contraction-simdgroup-refusal
title: Pin the strict-contraction simdgroup realization refusal
status: done
priority: p1
dependencies: [qualify-the-simdgroup-matrix-contraction-realization]
related: [research-an-explicit-seeded-fused-contraction-operation, realize-the-contraction-through-the-appendable-direct-path]
scopes: [implementation/metal, implementation/compiler, implementation/ir, contracts/artifacts, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, contraction, correctness, refusal, strict]
---
## Outcome

The owning realization boundary gives `simdgroup_multiply_accumulate` a named, typed incompatibility with `tiler::strict-tensor-contraction-f32@1`. The existing separately rounded, first-product-seeded lowering remains the only admitted Metal spelling. No target fact, fallback, default, or empirical runtime probe can turn the refused construct into that operation.

## Facts to re-read at the claimed base

These were re-read at `761f6802414cb98b68999ef85c87610460ac844a` before any edit.

- **Verified.** The registered `@1` operation requires separate multiplication/addition rounding, a first-product accumulator seed, and canonical NaN after every combine and at the result boundary. `strict_tensor_contraction_f32_facts` in `crates/tiler-ir/src/semantic/contraction.rs` installs `CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED = false`, `CONTRACTION_F32_FACT_SEED = none-the-accumulator-starts-at-the-first-product`, and `CONTRACTION_F32_FACT_NAN_CANONICALIZATION = after-every-combine-and-at-the-result-boundary`.
- **Verified, with a precision.** The retained `contraction_pair` observation distinguishes fused from separately rounded arithmetic by one result bit, and `negative_zero_seed` distinguishes a `+0.0` seed from the first-product seed. Finding 16 of `docs/research/apple-targets/numerical-behaviour.md` records the triple `scale = 1.5`, `bias = 1.0`, operand `3eb97ef9` as separately rounded `3fc58f9e` versus fused `3fc58f9d`. The L3 record `docs/research/scheduling/first-metal-contraction-realizations.md` reproduces that pair at `simdgroup_multiply_accumulate` under the same governed flags. `the_first_product_seed_and_a_positive_zero_seed_disagree_on_signed_zero` in `crates/tiler-reference/src/contraction/tests.rs` documents "This is the `negative_zero_seed` vector" and compares `0x80000000` against `0x00000000`.
- **Verified.** The current production Metal regression forbids `simdgroup`, `multiply_accumulate`, `fma`, and `mad` on the strict accumulation path. `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` in `crates/tiler-metal/src/tests.rs` forbids the substrings `fma(`, `metal::fma`, `precise::fma`, `simdgroup`, `multiply_accumulate`, and `mad(`.
- **Verified.** The retained finite attribution does not prove the instruction's contributor order, intermediate precision, or internal NaN behavior. The L3 record attributes `simdgroup` to `fma_zero_seed_fold+ftz` over eight cases and twenty-two named topologies and states that widening the corpus cannot turn that elimination into a universal hard guarantee. Apple specifies the matrix operation as `d = a * b + c` and does not publish those internals.

### Realization and feasibility census

- **No simdgroup candidate is enumerated.** The only production contraction lowering is the appendable `direct` path: `emit_contraction` in `crates/tiler-ir/src/kernel/lower.rs` seeds at the first product (`start: 1`), emits a separate multiply, canonicalize, add, and canonicalize, and `BinaryOp` has no fused multiply-add construct. `GovernedStrictTensorContractionF32` in `crates/tiler-compiler/src/governed.rs` states the same three properties. `crates/tiler-compiler/src/target/feasibility.rs` has no simdgroup realization predicate.
- **No typed feasibility or explain refusal already names this incompatibility.** The existing refusals that a contraction can reach (`reduction-contract`, `empty-contracted-domain`, `contraction-operands`, `unrealized-contraction` under a *permitting* contract) do not classify `simdgroup_multiply_accumulate`. Adding one would require inventing an executable candidate merely to reject it.
- **The structural prohibition is the only honest owning classification.** KIR verification requires `start == 1` for a contraction topology (`verify_contributor_loop`). Metal emission writes one operator per statement and the production regression forbids fused spellings. Explain output of the admitted `direct` plan therefore cannot name a simdgroup alternative, because none was formed. That absence is the feasibility/explain visibility this ticket can honestly own.

This is not a claim about a future distinct seeded fused operation. [`research-an-explicit-seeded-fused-contraction-operation`](research-an-explicit-seeded-fused-contraction-operation.md) remains deferred.

## Required delivery

Choose the narrowest owning typed refusal that remains visible in feasibility/explain output without adding a fake executable candidate merely to reject it. Reproduce the exact `contraction_pair` and `negative_zero_seed` observations in the owning check, retain the no-simdgroup emitted-source regression, and perturb fusion, seed, and NaN/order subjects independently so each load-bearing refusal fails with its own message. Preserve the distinction between “not a realization of `@1`” and any claim about a future distinct operation.

Do not revise `@1`, add a permissive target fact, infer semantics from the finite corpus, or widen artifact fixed records without a separately justified delivered fact. If the source census shows the current structural prohibition already provides the only honest owning classification, record and test that fact rather than creating duplicate error vocabulary.

## Closes when

The current strict operation has a reproducible, typed, explainable simdgroup incompatibility; both retained distinguishing observations and the emitted-source prohibition are guarded by subject perturbations; and every current identity/schema claim is re-derived at the implementation base.

## Outcome

The census chose the structural prohibition as the only honest owning classification. No public refusal variant, feasibility class, or explain code was added: `simdgroup_multiply_accumulate` is never enumerated as a candidate of `tiler::strict-tensor-contraction-f32@1`, so a typed feasibility rejection would have been a fake executable candidate. Explain and retained-plan output stay silent about that construct; `the_direct_contraction_plan_never_enumerates_a_simdgroup_realization` pins that absence.

The two retained observations are reproduced as IEEE host facts and, for `negative_zero_seed`, on the compiler-owned KIR interpreter. The no-simdgroup emitted-source regression is retained and split so fusion, seed, and NaN fail independently. `@1` was not revised. Artifact fixed records were not widened. Identity blast radius is none.

This is not a claim about a future distinct seeded fused operation.

### Quoted subject perturbations

Production subjects were broken independently and restored. Each load-bearing refusal named its own obligation:

- **Seed.** `emit_contraction` loop start moved from `1` to `0`. `the_contraction_lowers_to_a_first_product_separately_rounded_fold` failed with `the direct contraction lowers: Verification(ReductionContract)`. The hand-built sibling `a_positive_zero_seeded_contraction_loop_is_reduction_contract` quotes the stable rule `reduction-contract`.
- **Fusion.** A `// simdgroup` comment was inserted beside the emitted serial-loop marker. `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` failed with `simdgroup must not appear on a path whose contract forbids contraction:`.
- **NaN.** `emit_contraction_product` stopped canonicalizing the product. `the_contraction_kernel_canonicalizes_after_every_combine` failed with `each emitted product and sum commits the canonical payload:` and `left: 2` / `right: 4`.

`fusion_seed_and_nan_subjects_fail_independently` keeps those three messages watchable on copies of the emitted text so a later change cannot make one assertion cover the other two.

Coordinator `make full` on `9f8cc083`: 3475 passed. Merged to `main` at `eb64ff0397454e37139ea108e9fb0b09cdc52d0c`. No new public refusal surface; no additional Tom packet.
