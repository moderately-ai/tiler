---
id: execute-governed-refined-regions-against-the-oracle
title: Execute the governed lowerings' own refined regions against the scalar oracle in the compile path
status: done
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

## Outcome

Three cases in `crates/tiler-compiler/src/governed.rs` now execute the region `refine_index_region` **actually returned** for each governed family, through `IndexRegionEvaluator::new(FrozenReferenceRegistry::standard(), FrozenScalarReferenceRegistry::standard())`.

**The gap this closes.** `crates/tiler-reference/tests/governed_scalar_reference.rs` answers the numerical questions completely, but against hand-written *mirrors* of these emissions. A mirror that drifted from `governed.rs` would keep passing. These cases take `IndexRefinement::region()` from the governed provider set itself, so a change to a lowering's emitted arithmetic has nowhere to hide.

They live in `tiler-compiler` for the reason the ticket states: `tiler-reference` depends only on `tiler-ir`, and the oracle must not depend on the compiler. `tiler-compiler` dev-depends on `tiler-reference`, which is the one direction that composes. Scope is `implementation/compiler` alone, as declared.

**Comparison is on exact bit patterns, not `f32` equality.** `-0.0 == 0.0` is true and a NaN equals nothing, so float comparison would silently accept exactly the results a numerical contract exists to pin.

**What is checked.** The constant lowering reproduces its declared payload for `1.0`, `-0.0`, the least subnormal, and a non-canonical NaN — a constant is bit-preserving, so the NaN must survive uncanonicalized. Multiply and add over the same vector canonicalize the NaN they produce to `0x7fc00000` while `-0.0` keeps its sign and the subnormal survives. The strict serial sum folds `1 + 2 + 3 = 6`, canonicalizes a lone non-canonical NaN at its result boundary, leaves a lone `-0.0` alone — which is what distinguishes the boundary conversion from an addition — and preserves a subnormal across a two-element fold.

**The ordering note is discharged rather than honoured.** It said to exclude the single-contributor NaN case and cite `reconcile-single-contributor-strict-sum-nan-canonicalization`. That ticket is now `done`: a lone contributor canonicalizes at the reduction's result boundary, all three implementations agree, and the case discriminates. It is included.

**Measurement — the cases are shown to discriminate, not assumed to.** Deleting the result-boundary canonicalization from `GovernedStrictSerialSumF32` (replacing the `canonicalize_nan_f32_scalar_op` apply with the bare `seed`) fails `the_governed_serial_sum_region_executes_its_declared_contract` with `left: [2143294004]` against `right: [2143289344]` — the input's `0x7fc01234` leaking through where `0x7fc00000` is required. The defect was reverted and the suite is green.

`cargo nextest run --workspace --no-fail-fast` — 628 passed, 0 skipped; clippy and fmt clean.
