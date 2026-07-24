---
id: execute-governed-refined-regions-against-the-oracle
title: Execute the governed lowerings' own refined regions against the scalar oracle in the compile path
status: todo
priority: p1
dependencies: []
related: [register-governed-scalar-reference-evaluation, reconcile-single-contributor-strict-sum-nan-canonicalization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, reference, testing, milestone-0b]
---
`register-governed-scalar-reference-evaluation` shipped `tiler_reference::FrozenScalarReferenceRegistry::standard()`, so a region over `tiler.scalar::{constant,multiply,add}-f32@1` is now executable by the independent `IndexRegionEvaluator`. It could not run the regions `tiler_compiler::governed` *actually emits*, and that residue is this ticket.

**The boundary that blocked it, stated exactly.** `tiler-reference` depends only on `tiler-ir`; `tiler-compiler` dev-depends on `tiler-reference`. So `tiler-reference` cannot import `tiler_compiler::governed`, and the module is `pub(crate)` besides. Adding the reverse dependency would invert the layering `AGENTS.md` requires — the reference oracle must not depend on the compiler — so the test genuinely cannot live in `tiler-reference`. Its natural home is `crates/tiler-compiler`, which is `implementation/compiler` and outside the originating ticket's declared scope.

**What exists today, and its precise limitation.** `crates/tiler-reference/tests/governed_scalar_reference.rs` builds hand-written *mirrors* of each governed family's emission — read at reduced offset zero as the seed, fold the remaining contributors read at `tail + 1`, one rank-zero apply and write for the constant, one broadcast read for the pointwise families — and compares them against `ReferenceEvaluator` bit for bit. The mirrors were written by reading `governed.rs` step for step, and they answer the *numerical* questions completely. What they cannot catch is a mirror that has drifted from the emission: if `GovernedStrictSerialSumF32` changed its fold shape tomorrow, the mirror would keep passing.

**What this ticket must produce.** In `crates/tiler-compiler`, run `legality::refine_index_region` for each of the four governed families, take the resulting `IndexRefinement::region()`, and evaluate it through `IndexRegionEvaluator::new(FrozenReferenceRegistry::standard(), FrozenScalarReferenceRegistry::standard())` against `ReferenceEvaluator` on the equivalent semantic program. `legality::tests::refinement_output_is_checkable_against_the_reference_oracle` already does exactly this for one ad-hoc provider and is the shape to follow; the difference is that it must now be the *governed* provider set and the standard scalar oracle rather than a one-off `multiply_reference`.

Use the conformance vectors from `pipeline::tests::structured_fused_body_interpreter_matches_reference_evaluator` and the sign-of-zero, subnormal, and non-canonical-NaN vectors `governed_scalar_reference.rs` adds. A vector built from `f32::NAN` cannot discriminate anything, because `f32::NAN` is already `CANONICAL_F32_ARITHMETIC_NAN_BITS`.

**Closing evidence.** Each governed family's refined region is executed by the oracle and agrees with the semantic evaluator bit for bit, on vectors that include at least one non-canonical NaN payload and both signed zeros. A deliberate defect injected into any governed lowering's emitted arithmetic fails the test.

**Ordering.** `reconcile-single-contributor-strict-sum-nan-canonicalization` records a measured disagreement on a single-contributor reduction over a non-canonical NaN. Until it is decided, exclude that one case explicitly and cite that ticket, rather than choosing a vector that happens to avoid it.
