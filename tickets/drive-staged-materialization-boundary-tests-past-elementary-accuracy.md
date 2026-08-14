---
id: drive-staged-materialization-boundary-tests-past-elementary-accuracy
title: Drive staged materialization boundary tests past elementary accuracy
status: todo
priority: p1
dependencies: [declare-elementary-realizations-on-a-target-profile]
related: [account-for-a-staged-realization-stage-in-the-kernel-program, admit-a-scheduled-region-for-a-staged-elementary-family, admit-a-staged-family-that-reads-a-materialized-intermediate, admit-a-materialized-producer-in-a-serial-reduction-contributor, admit-a-scheduled-region-that-reads-two-materialization-edges, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, numerics, testing, compiler]
---
## User-visible outcome

Staged-family boundary tests reach the recognition, region, program-assembly, and numerical paths they claim to measure. A missing elementary-accuracy declaration remains a separate fail-closed governed-profile control instead of making every deeper structural test green before its subject runs.

## Exact-base Fact audit — 2026-08-13, base `c9da757ec4717000296a6b0d3c70a2e0be345c23`

This audit was read against the source, capability tickets, accepted caller-profile declaration, and live tests before this ticket was written.

**Fact 1 — verified.** `TargetProfile::governed()` publishes no elementary-realization rows. `TargetProfile::declared_elementary_realizations` says the slice is empty for the governed profile until later Metal evidence discharges both halves; the builder path does not synthesize a default. Therefore any program containing `tiler::rms-norm-f32@1` refuses during request verification as `accuracy.elementary.no-installed-realization` before recognizer, region, cover, program, or kernel evidence can run.

**Fact 2 — verified.** The accepted caller-profile surface already supplies the missing test authority. `TargetProfileBuilder::declare_elementary_realization` stores one whole validated `ElementaryRealization`; `crates/tiler-compiler/tests/caller_target_profile.rs`, anchor `a_caller_built_profile_with_both_halves_compiles_silu`, constructs two discharging evidence halves and proves a declared elementary row can pass request verification. This ticket needs no new public type, method, schema, identity rule, or governed target claim.

**Fact 3 — false in the live test names and comments.** `pipeline::tests::a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit` and `the_staged_regions_compute_the_normalization_bit_for_bit` say they compile or compare bits, but each body compiles `CompilationRequest::governed` and asserts only `UnrealizedElementaryAccuracy`. Neither test currently constructs, verifies, interprets, or compares the staged regions or assembled program its documentation describes.

**Fact 4 — false in the external boundary fixtures.** `staged_family_over_a_materialized_intermediate.rs` says its declared-input control compiles under the same request and that the subject reaches the scheduled-region vocabulary. Both tests instead assert the same missing-accuracy refusal. `recognized_chain_depth_boundary.rs` similarly claims the two-boundary RMS chain reaches `staged-operand-depth`, while its live assertion stops at missing accuracy; only its RMS-free one-boundary control genuinely compiles.

**Fact 5 — verified and independent of elementary accuracy.** `materialized_intermediate_epilogue_wall.rs`, anchor `The admission is one materialization boundary wide`, misclassifies `sum(contract(a, b) * 2)` as requiring two intermediate reads. The recognizer actually discovers one materialized producer, an optional continuation region, and the fold; each consumer region reads at most one intermediate. The live refusal is the missing serial-sum producer carrier, `reduction-contributor-materialization`, owned by [`admit-a-materialized-producer-in-a-serial-reduction-contributor`](admit-a-materialized-producer-in-a-serial-reduction-contributor.md).

**Fact 6 — imprecise in the completed capability records.** [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md) and [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md) landed real region/program vocabulary and identity changes. Their current source remains present. Their historical statements that the governed request compiles and agrees bit for bit are no longer evidence at this base, however, because the named tests stop at request accuracy. The implementation nodes remain complete; this ticket repairs their invalidated proof boundary rather than reimplementing them.

This repair does not change the ticket's purpose: all affected tests need the same existing caller-declared authority before their distinct structural subjects become observable.

## Required work

- Build one test-only caller `TargetProfile` that preserves the governed profile's relevant quantitative, dispatchability, numerical, synchronization, and cost facts while declaring a verified `tiler::rms-norm-f32@1` realization whose bound and exceptional-value evidence both discharge at compile-profile phase. Derive the contract from `rms_norm_f32_rsqrt_accuracy_contract`; do not invent a looser contract, borrow an undischarged Metal row, or alter `TargetProfile::governed()`.
- Route the two pipeline measurements through that profile and restore what their names promise: one region-level spelling/verification/interpreter comparison and one ordinary `compile()`/assembled-program/reference bit comparison. Keep a separate governed-profile negative that asserts `accuracy.elementary.no-installed-realization` and proves the positive is not bypassing assessment.
- Route `staged_family_over_a_materialized_intermediate.rs` through the declared profile so its declared-input control genuinely progresses and the materialized-operand subject reaches its intended scheduled-region boundary. Re-derive the expected rule and contract-dependent population from the live path; do not copy the pre-accuracy prose.
- Route the RMS subject in `recognized_chain_depth_boundary.rs` through the declared profile so the depth assertion actually reaches recognition. Keep the RMS-free one-boundary control and repair every stale name, comment, count, and trigger statement.
- Correct `materialized_intermediate_epilogue_wall.rs`'s nested-contraction explanation from a two-edge-width claim to the one-producer carrier wall. This is a prose correction only; the carrier remains owned by its separate decision ticket.
- Append exact correction records to the completed capability tickets, naming which implementation claims remain supported and which end-to-end measurements this ticket now owns.

## Evidence and negative controls

- Perturb only the caller profile by dropping the RMS row: every RMS subject must refuse as `accuracy.elementary.no-installed-realization`, with the failure text recorded.
- Perturb only the declared row's contract so it does not refine the required RMS contract: request verification must refuse as `accuracy.elementary.unrefined-realization` rather than reaching the structural assertion.
- After restoring the row, perturb each structural subject independently: drop the staged program declaration and observe `UncoveringStage`; restore the old staged-family spelling wall and observe the vocabulary refusal; open the depth admission and observe the depth test change; keep the declared-input/RMS-free neighbours unchanged.
- Count the intended population explicitly. No test may pass solely because a loop ran zero subjects or every case stopped at the shared request gate.

## Boundaries and non-goals

No governed Metal accuracy row, new public surface, target descriptor change, elementary-contract widening, carrier decision, multi-edge schedule widening, or backend-performance claim. Test evidence is bounded to the caller-declared fixture and must not be restated as a governed-target guarantee.

## Closes when

Every named staged-family test reaches the layer its name and documentation claim, the governed missing-row refusal remains independently pinned, the completed capability tickets withdraw their stale governed/end-to-end evidence, and the subject perturbations above demonstrate that request accuracy and each structural boundary can fail independently.
