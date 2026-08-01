---
id: admit-the-contraction-normative-reference
title: Admit the contraction normative reference and its exceptional-value corpus
status: done
priority: p1
dependencies: [admit-the-contraction-semantic-profile]
related: [implement-parallel-reduction-strategies, reduction-semantics-contract]
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, numerics, contraction, reductions]
---
## User-visible outcome

The reference evaluator computes the admitted contraction, so every later realization has a target-independent answer to be bit-compared against instead of being compared to itself.

## The formula, and the part of it that is easy to get wrong

**Proposal — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** For each output coordinate `(t, o)`, over the canonical ascending contributor sequence `d = 0 .. K-1`:

```text
p_d = fl(A[t, d] * B[o, d])      # one rounding each, round-to-nearest ties-to-even
acc = p_0                        # the FIRST product, not +0.0
for d in 1 .. K-1: acc = fl(acc + p_d)
```

**Measurement — the seed is observable and the idiomatic loop gets it wrong.** `fl(+0.0 + x)` equals `x` for every binary32 `x` except `x = -0.0`, where it is `+0.0`. On the spike's `negative_zero_seed` case, where every product is `-0.0`, a first-product-seeded fold returns `0x80000000` and a `+0.0`-seeded one returns `0x00000000`. [Numerical semantics](../docs/numerical-semantics.md) states the same rule for the registered strict sum and gives the same counterexample under reduction padding. **This must be a regression test that fails before the evaluator is written correctly**, not a comment.

**Fact — a `+0.0` seed is a different operation, not a defect.** It is a reduction carrying an explicit `initial`, which the reduction contract admits as one logical contributor. The evaluator must be able to express both and must not silently supply one.

## Required delivery

- A `tiler-reference` evaluator for the admitted key, with the accumulator dtype, the contributor order, the empty-domain declaration, and the seed all read from the operation's own signature rather than defaulted.
- Per-combine and result-boundary canonicalization to `tiler::canonical-arithmetic-nan-f32@1`, on the same rule the registered strict serial sum already carries. **Open decision D-8** of the L3 record asks whether per-combine canonicalization is required of a contraction or only at its boundary; this ticket is where it gets answered, because the answer changes what a matrix instruction could ever satisfy.
- The exceptional-value corpus, at minimum the spike's eight cases: an execution witness, order absorption, a separately-rounded-against-fused discriminator, the signed-zero seed, a non-canonical NaN payload, `inf * 0` formed inside the reduction, a subnormal product, and a vector separating the contiguous from the strided split. Their exact operand bit patterns are retained in `spikes/scheduling/metal_contraction_vertical/results/.../semantics-candidates.tsv`.
- A statement of which conformance level the evaluator's own results claim.

## Non-goals

Any schedule, any backend, any tolerance for a model-level comparison.

## Closes when

The evaluator reproduces every retained candidate value for `strict_fold` in the spike's semantics record, the signed-zero seed test was watched failing against a `+0.0`-seeded implementation, and D-8 is answered in the operation's declared signature rather than left to the implementation.

## Outcome

**Fact — the ticket's D-8 framing was already superseded when this work started.** `admit-the-contraction-semantic-profile` landed `tiler::strict-tensor-contraction-f32@1` with a fourteen-field numerical signature in which `CONTRACTION_F32_FACT_NAN_CANONICALIZATION` already declares `after-every-combine-and-at-the-result-boundary`. D-8 was therefore not answered here; it was *implemented and read* here. The semantic definition was not touched.

**Delivered.** `crates/tiler-reference/src/contraction.rs` registers a reference evaluator for the admitted key, over an arbitrary admitted binary index structure rather than only `td,od->to`. It decodes the operation's own fourteen-field signature — via the new `tiler_ir::semantic::strict_tensor_contraction_f32_facts()`, which returns the same value `register_standard_contraction` installs, from the same constructor — and refuses, by field ID, any declaration it does not realize. Four fields are read as values (accumulator type, result type, canonical NaN payload, seed); the other ten are verified against the one reading the fold implements. A refused declaration fails the *registration* (`ReferenceRegistryError::UnsupportedContraction`), so no capability is bound for a contract the reference cannot compute.

**Measurement — the corpus reproduces the spike's `strict_fold` column exactly, first run, with no adjustment.** Operand vectors transcribed from `spikes/scheduling/metal_contraction_vertical/contraction_probe.py::semantic_cases()`; expectations from the `strict_fold` rows of `results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/semantics-candidates.tsv`.

| case | expected `strict_fold` | reproduced |
| --- | --- | --- |
| `witness` | `40c00000` | yes |
| `order_absorption` | `4b800007` | yes |
| `contraction_pair` | `3fc58f9e` | yes |
| `split_topology` | `bb1d0482` | yes |
| `negative_zero_seed` | `80000000` | yes |
| `nan_payload` | `7fc00000` | yes |
| `infinity_times_zero` | `7fc00000` | yes |
| `subnormal_product` | `00400000` | yes |

Two further retained candidate values are reproduced as perturbations, by folding the reversed contributor sequence: `order_absorption`/`reversed_fold` = `4b800005` and `split_topology`/`reversed_fold` = `bb1d0494`. Reversal is the permutation this family forbids, so the ascending order is discriminated rather than assumed.

**Measurement — the seed regression was watched failing.** With `ContractionSeed::FirstProduct` temporarily folded as `Some(0.0_f32)`, exactly two of 103 `tiler-reference` tests failed:

```text
thread 'the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero' panicked at
crates/tiler-reference/tests/contraction_conformance.rs:254:5:
assertion `left == right` failed: a `+0.0`-seeded fold returns 0x00000000 here
  left: 0
 right: 2147483648
```

That *only* those two failed is itself the finding: the other seven corpus cases pass under the wrong seed, which is exactly why the `negative_zero_seed` vector has to exist. The sabotage was reverted and the full suite is green.

**Inference — per-combine canonicalization is not observable in this evaluator's own outputs.** Binary32 addition of a NaN accumulator yields a NaN whatever the payload, so no intermediate payload reaches the result by any route other than the boundary, which canonicalizes it. Canonicalizing per combine is strictly stronger than canonicalizing nowhere (`payload_propagating_fold` returns `7fc0dead` where this returns `7fc00000`), but indistinguishable from boundary-only *here*. The declared per-combine rule therefore binds realizations — a matrix instruction, a split fold, a device whose propagation differs — not this fold. The check that can say no about the site is consequently the signature decode, which refuses a boundary-only declaration rather than accepting one it would over-satisfy.

**Conformance level claimed.** `ReferenceNumericalConformance::strict`: both subnormal dimensions preserved, separately rounded multiply and add, strict left fold in the declared contributor order, bit-preserving signed zeros. Its results are *the* value the declared contract names rather than a member of an admitted set — which is only meaningful because the declared signature forbids fusion, reassociation, and permutation. The spike's `+ftz` column is a qualified Apple9/F32 target property, not this operation's contract, which is why `subnormal_product` expects `00400000`. A pass is evidence about the semantic contract and the host evaluator, and about no schedule, lowering, kernel, device, or model tolerance.

**Fact — the explain request digest did not move.** It remains `bddeaf899938ede4`; `cargo nextest run -p tiler-compiler` is green unchanged. Traced: the request subject binds the frozen *semantic* registry snapshot and the *lowering* registry identity. This ticket registers a *reference* capability and adds one pure accessor to `tiler-ir` that registers nothing, so neither subject moved. No rebaseline.

**New public items, for Tom.** `tiler_ir::semantic::strict_tensor_contraction_f32_facts()`; `tiler_reference::UnsupportedContractionDeclaration` (with `MalformedRecord`, `UnrealizableFact { field }`, `unrealizable`, `rule`); `ReferenceRegistryError::UnsupportedContraction { operation, source }`, a variant on a `#[non_exhaustive]` enum. The evaluator itself is crate-private and follows the existing provider idiom. The `tiler.standard-reference` provider revision was **not** bumped, following the precedent this ticket's base records for the dtype catalog, the contraction, and the structural families.

**Deliberately not done.** No schedule, backend, or model-level tolerance. No canonical spelling was invented for a seeded contraction: `ContractionSeed::Initial` is expressible by the fold — which is what the seed regression compares against — but no fact record can declare it, because inventing that spelling would introduce a semantics the normative text has not defined.

**Bounded remainder, filed:** `bound-the-reference-contraction-iteration-space`. The multiply-accumulate work bound added here reuses `ReferenceOperationError::ShapeTooLarge`, whose documented meaning is shape arithmetic rather than iteration-space work. It refuses correctly; the diagnostic is imprecise, and the fix is a public error variant.

**Boundary acceptance (2026-07-31).** Tom accepted the three public items as reviewed — `strict_tensor_contraction_f32_facts()`, `UnsupportedContractionDeclaration`, and the `UnsupportedContraction` variant — and the stated judgment that per-combine NaN canonicalization admits no value-comparison test in this evaluator.
