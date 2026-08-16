---
id: refresh-multi-output-correctness-row-after-access-ordinal-reconciliation
title: Refresh the multi-output correctness row after access-ordinal reconciliation
status: done
priority: p2
dependencies: []
related: [repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation]
scopes: [contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: []
---
The live paragraph in docs/correctness-and-testing.md at anchor `The multi-output row is now positive` still says a fold over a later declared input refuses under sum-contributor-ordinal and says contraction consumers rely on dense declaration ordinals zero and one. Exact current source includes request::tests::a_fold_over_a_later_declared_input_retains_its_ordinal and the completed AccessOrdinal/DeclaredInputOrdinal reconciliation. Read the complete document and current owning tests/tickets, audit every present-tense support claim in that paragraph, then correct only claims superseded by landed work while preserving dated measurements and historical corrections. Run a source-claim negative perturbation, tkt lint, make citations, git diff --check, and exact-base tkt guard.

## Exact-base Fact audit — 2026-08-16, `6ee6c133b2d56e9a403dc126190558b168c754ce`

- **Verified:** the complete 433-line `docs/correctness-and-testing.md` still said, in the paragraph anchored `The multi-output row is now positive`, that a fold over a later declared input refused under `sum-contributor-ordinal` and that contraction required exactly two declared inputs whose consumers relied on dense declaration ordinals zero and one.
- **Verified:** `request::tests::a_fold_over_a_later_declared_input_retains_its_ordinal` distinguishes input-zero and input-one subjects and retains the later contributor's compiler-private declared-input ordinal. `pipeline::conformance::outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read` compiles the later-input fold beside an independent pointwise output, retains the no-materialization fused alternative reading declared input one, and bit-compares both outputs with the reference. The named fold ticket records this support landing on 2026-08-10.
- **Verified:** `request::tests::contraction_subjects_separate_all_two_input_subsets_of_three_declarations` distinguishes ordered read maps `[0, 1]`, `[0, 2]`, and `[1, 2]`. `pipeline::conformance::a_contraction_over_an_input_subset_compiles_and_matches_the_reference` compiles the `[0, 2]` subset beside an independent output reading input one and bit-compares both outputs with the reference. The named contraction ticket records the replacement of the dense declared-ordinal assumption with a complete interface and explicit two-read map.
- **Verified:** the supported contraction remains binary: its registered `OperationArity::exact(2)` and its normalized and scheduled two-read forms exclude multi-operand contraction, and repeated use of one declared input still refuses under `contraction-operands`.
- **Verified:** after the completed access-coordinate reconciliation, shared `TensorRole::Input` is fieldless. The compiler-private association is recovered by projecting each `AccessOrdinal` through `VerifiedScheduledRegion::declared_input_at`; no public interface ordinal is assigned to the shared role.
- **False at review commit `962e7fd54b56a74580bd673788251a8799864dbf`; repaired here:** the initial audit said every other present-tense support statement matched its owner, but it missed the claim that `crates/tiler-compiler/tests/multi_output_boundary.rs` was the executable form of the whole paragraph. The complete 589-line fixture contains positives for independent ordered outputs, disjoint elementwise input subsets, and semantic output-order identity, but no later-input-fold or contraction positive; its module prose still contains the retired `Its remainder is a fold` and `sum-contributor-ordinal` refusal. The contract now assigns only the fixture's three proved rows to its named tests and leaves later-input fold and binary-subset contraction with their true request and conformance owners. The paragraph's dated measurements, historical correction provenance, and unrelated present-tense claims remain preserved.

The first Fact was true about the stale source at this exact base but no longer described landed behavior. The remaining Facts establish the narrow replacement boundary; they do not change this ticket's purpose and require no production edit.

Exact-base `tkt guard` maps the authorized `docs/correctness-and-testing.md` edit to `contracts/numerics`, so that required scope is declared alongside the ticket's original `contracts/foundation` scope. This is scheduling metadata for the same two-file prose repair, not an outcome or path expansion.

## Source-claim check

Run from the repository root:

```sh
if rg -n -o '\*\*What it still does not cover\*\* is a fold|A contraction still requires exactly two declared inputs|rely on dense declaration ordinals zero and one|refuses at the request boundary under `sum-contributor-ordinal`|is the executable form of this paragraph' docs/correctness-and-testing.md; then
    exit 1
fi
test "$(rg -c '\*\*The later-input strict-serial-fold row is now positive\.\*\*' docs/correctness-and-testing.md)" -eq 1
test "$(rg -c '\*\*The binary contraction row now admits any ordered two-input subset of a wider declared interface\.\*\*' docs/correctness-and-testing.md)" -eq 1
test "$(rg -c 'The caller-boundary evidence in `crates/tiler-compiler/tests/multi_output_boundary\.rs` is narrower' docs/correctness-and-testing.md)" -eq 1
test "$(rg -c 'fn a_fold_over_a_later_declared_input_retains_its_ordinal' crates/tiler-compiler/src/request.rs)" -eq 1
test "$(rg -c 'fn contraction_subjects_separate_all_two_input_subsets_of_three_declarations' crates/tiler-compiler/src/request.rs)" -eq 1
test "$(rg -c 'fn outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read' crates/tiler-compiler/src/pipeline/conformance.rs)" -eq 1
test "$(rg -c 'fn a_contraction_over_an_input_subset_compiles_and_matches_the_reference' crates/tiler-compiler/src/pipeline/conformance.rs)" -eq 1
test "$(rg -c 'fn an_ordered_two_output_program_compiles' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 1
test "$(rg -c 'fn two_outputs_reading_disjoint_declared_inputs_compile_binding_only_what_they_read' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 1
test "$(rg -c 'fn two_programs_differing_only_in_output_order_have_distinct_identities' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 1
```

The unchanged check passed on the repaired source. As a negative control, the contract subject was temporarily changed from `The binary contraction row now admits any ordered two-input subset of a wider declared interface` back to the retired claim `A contraction still requires exactly two declared inputs`; the check failed with status 1 and:

```text
118:A contraction still requires exactly two declared inputs
```

Restoring the repaired subject made the same check pass again. This perturbs the documented support claim, not the check or its assertion.

The reviewer repair added the fixture-ownership clause and its three named test owners to that same check. Temporarily restoring the overbroad documentation subject `is the executable form of this paragraph` made the unchanged check fail with status 1 and:

```text
118:is the executable form of this paragraph
```

Restoring the narrowed subject made the check pass again. The fixture itself was read but not edited: its executable tests support the narrower delegation, while its retired later-input-fold prose is no longer treated as the owner of the positive fold or contraction clauses.

## Verification record

- `cargo nextest run -p tiler-compiler -E 'test(/(a_fold_over_a_later_declared_input_retains_its_ordinal|outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read|contraction_subjects_separate_all_two_input_subsets_of_three_declarations)/)'` — 3 passed.
- `cargo nextest run -p tiler-compiler -E 'test(a_contraction_over_an_input_subset_compiles_and_matches_the_reference)'` — 1 passed.
- Reviewer repair: one nextest selection containing the three narrowed fixture owners and all four positive fold/contraction owners — 7 passed, 926 skipped.
- The published `make full` result at `4fc98a79` carries because this delta changes only this ticket and `docs/correctness-and-testing.md`, none of the gate-invalidating paths. This delta reruns `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard` as required.
