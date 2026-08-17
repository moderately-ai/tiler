---
id: repair-retired-input-ordinal-claims-in-compiler-pipeline-tests
title: Repair retired input-ordinal claims in compiler pipeline tests
status: in-progress
priority: p2
dependencies: []
related: [repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [defect, documentation, access-ordinal]
claimed_from: todo
assignee: worker-pipeline-doc
lease_expires_at: 1786965597
---
## User-visible outcome

Two present-tense comments in `crates/tiler-compiler/src/pipeline/tests.rs` no longer assign declared-interface ordinal authority to fieldless `TensorRole::Input` or intrinsic schedule verification.

## Exact-base audit obligation

Before editing, read the complete 9,411-line file and re-audit the anchors ``the `TensorRole::Input` ordinal equal`` and `declared input ordinals to ascend strictly` at the exact claimed base. Record per-Fact verdicts before changing source.

## Exact-base Fact audit — 2026-08-17, `e8141d7decbb8204e7930421d0b1acedef9b4dd5`

- **Verified — the required complete-file reading remains exact at this base.**
  `crates/tiler-compiler/src/pipeline/tests.rs` is 9,411 lines and is
  byte-identical between the prior complete audit at
  `f46ac65cc6050c6804f9376f2fb86e44430c8c31` and this branch base. The two
  named anchors each occur once and delimit the only two present-tense
  pipeline-test comment blocks that assign declared-interface authority to a
  fieldless role or to intrinsic schedule verification.
- **False — `TensorRole::Input` supplies an ordinal that can be kept equal to
  an expression leaf.** `crates/tiler-ir/src/schedule/model.rs`, anchor
  `Association with a named program input belongs to the compiler's checked
  request subject`, defines a fieldless category. The actionable failure is
  reusing leaf/access position `1` as declared ordinal `1` instead of projecting
  exact `AccessOrdinal(1)` through the retained checked request subject.
- **Imprecise — intrinsic verification cannot notice the wrong association,
  but not because it compares two valid interface spellings.**
  `crates/tiler-ir/src/schedule/builder.rs`, anchor `Exact declared-input
  association is absent here`, checks local access structure and boundary
  categories. The shared region carries no declared-input association to
  compare; `crates/tiler-compiler/src/physical.rs`, anchor `Projects one local
  input access back to the declared program interface`, owns that projection.
- **False — the mixed staged/input epilogue is the only current shape reaching
  the separation.** The later
  `a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`
  fixture also compiles an outer epilogue over a staged value and a declared
  input. The local fixture remains a discriminating positive, but it no longer
  owns an exclusive population claim.
- **False — intrinsic pointwise verification requires declared input ordinals
  to ascend or refuses repeated/descending associations.** The fieldless helper
  `reads_bind_boundary_tensors_in_order` checks only boundary categories and at
  most one intermediate. `crates/tiler-compiler/src/request.rs`, anchors
  `Orders one walk's leaf reads into the read list` and `The staged read, then
  whichever declared inputs`, owns canonical compiler normalization; program
  assembly later projects each exact access through the checked subject.
- **Verified — the two existing tests exercise the behavioral claims without
  an assertion or fixture change.**
  `an_epilogue_reading_a_staged_value_and_a_declared_input_matches_the_reference`
  makes substituting `b` for `a` observable, while
  `an_epilogue_reaching_declared_inputs_out_of_order_still_compiles` drives both
  leaf-discovery orders and supplies buffers in the compiler's canonical access
  order.

Reproduce:

```sh
test "$(wc -l < crates/tiler-compiler/src/pipeline/tests.rs | tr -d ' ')" -eq 9411
git diff --quiet f46ac65cc6050c6804f9376f2fb86e44430c8c31 e8141d7decbb8204e7930421d0b1acedef9b4dd5 -- crates/tiler-compiler/src/pipeline/tests.rs
git grep -n -E 'the `TensorRole::Input` ordinal equal|declared input ordinals to ascend strictly' e8141d7decbb8204e7930421d0b1acedef9b4dd5 -- crates/tiler-compiler/src/pipeline/tests.rs
```

## Required work

Preserve the behavioral claims and fixtures; attribute declared association and canonical ordering to the retained checked request subject and exact `AccessOrdinal` projection.

Run the named epilogue tests, a source-phrase negative perturbation, `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard`.

## Implementation record — 2026-08-17

- Repaired only the two audited comment blocks. The mixed epilogue now names
  exact `AccessOrdinal` projection through the retained checked
  `VerifiedRequestSubject`, and the ordering fixture names compiler
  normalization rather than an intrinsic declared-ordinal rule. The false
  exclusive-population claim was removed. No fixture, assertion, executable
  code, public surface, identity, schema, or supported population changed.
- Both named exact tests passed. Compiler all-target check and Clippy with
  warnings denied, rustdoc with warnings denied, compiler doctests, formatting,
  ticket lint, citations, diff check, and the exact-base scope guard passed.
- The final source negative rejects either retired phrase and is green with both
  absent. Reintroducing only ``the `TensorRole::Input` ordinal equal`` made it
  fail with:

  ```text
  1477:/// compiler that kept the leaf index and the `TensorRole::Input` ordinal equal
  ERROR: retired input-ordinal authority phrase remains
  ```

  After restoration, reintroducing only
  `declared input ordinals to ascend strictly` made it fail with:

  ```text
  1559:/// rule requiring declared input ordinals to ascend strictly.** The recognizer's
  ERROR: retired input-ordinal authority phrase remains
  ```

Verification:

```sh
cargo test -p tiler-compiler --lib pipeline::tests::an_epilogue_reading_a_staged_value_and_a_declared_input_matches_the_reference -- --exact
cargo test -p tiler-compiler --lib pipeline::tests::an_epilogue_reaching_declared_inputs_out_of_order_still_compiles -- --exact
if rg -n 'the `TensorRole::Input` ordinal equal|declared input ordinals to ascend strictly' crates/tiler-compiler/src/pipeline/tests.rs; then exit 1; fi
cargo fmt --all -- --check
cargo check -p tiler-compiler --all-targets
cargo clippy -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps -p tiler-compiler
cargo test -p tiler-compiler --doc
tkt lint --format json
make citations
git diff --check
tkt guard tkt/repair-retired-input-ordinal-claims-in-compiler-pipeline-tests --base e8141d7decbb8204e7930421d0b1acedef9b4dd5 --format json
```
