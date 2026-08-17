---
id: refresh-multi-output-boundary-fixture-later-input-fold-prose
title: Refresh the multi-output boundary fixture's retired later-input-fold prose
status: done
priority: p3
dependencies: []
related: [refresh-multi-output-correctness-row-after-access-ordinal-reconciliation, admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, multi-output]
---
## Exact-base Fact audit — 2026-08-16

Audited before edits at exact base
`c38c96a6f5886ba76d51e59d7b44751f2bee5c46`. Read in full: repository
`AGENTS.md`; this ticket; the completed
`admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`
ticket; the complete 589-line
`crates/tiler-compiler/tests/multi_output_boundary.rs`; and the current
`NormalizedSerialSum` recognition, verified scheduled-region association,
cover assembly, request test, and conformance fixture/test owners.

- **False provenance; repaired without changing purpose.** The ticket named
  historical base `0f04651e8a0f2c132ce37cab6b86552edc46044c`, not this
  checkout's exact base above. The stale prose and its 589-line fixture remain
  present at this base.
- **Verified.** The fixture says `Its remainder is a fold`, says `The fused
  single-region alternative is withheld`, and assigns the refusal to
  `sum-contributor-ordinal`. Its complete `rg -n '^fn '` census has no positive
  later-input-fold test.
- **Verified.** `request::tests::a_fold_over_a_later_declared_input_retains_its_ordinal`
  proves that `recognize_reduction` retains the contributor's true
  `DeclaredInputOrdinal` and gives input-zero and input-one subjects distinct
  bytes.
- **Verified.**
  `pipeline::conformance::outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read`
  retains the no-materialization affine-fold alternative reading declared
  input one and compares both outputs bit for bit.
- **Verified.** `VerifiedScheduledRegion::declared_input_at` projects a local
  `AccessOrdinal` through the already-verified request subject;
  `CoverAssembly::from_plan` consumes that checked association before forming
  `AssemblyBinding::Input`.
- **Imprecise only if read as exhaustive; repaired here.** This fixture owns
  ordered-output admission, disjoint elementwise input-subset binding,
  shared-publication refusal, the same-shaped split stage-key collision,
  semantic output-order identity, and the `ValueRole` publication/consumption
  refusal. The narrow prose repair preserves all those rows; it assigns no
  later-input-fold guarantee to this fixture.

The completed fold ticket retired the stale refusal. Update only the module
prose to point at the landed request and conformance owners and preserve this
fixture's actual support, refusal, collision, and identity claims. No production
behavior, public surface, identity, schema, or Rust test population changes.

## Source-subject check

Run from the repository root:

```sh
set -eu
if rg -n 'Its remainder is a fold|The fused single-region alternative is withheld|sum-contributor-ordinal' crates/tiler-compiler/tests/multi_output_boundary.rs; then
    exit 1
fi
test "$(rg -c 'The later-input fold row is now positive' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 1
test "$(rg -c 'fn a_fold_over_a_later_declared_input_retains_its_ordinal' crates/tiler-compiler/src/request.rs)" -eq 1
test "$(rg -c 'fn outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read' crates/tiler-compiler/src/pipeline/conformance.rs)" -eq 1
test "$(rg -c 'a_fold_over_a_later_declared_input_retains_its_ordinal' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 1
test "$(rg -c 'outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read' crates/tiler-compiler/tests/multi_output_boundary.rs)" -eq 2
```

The first command reaches the retired source subject and says no by printing
the offending source line before exiting nonzero. The count checks pin the one
new ownership statement, both true owner definitions, and this fixture's exact
references to them without adding a Rust test.

After the prose and check pass, perturb the source subject independently by
restoring one retired sentence, quote the unchanged check's failure, and
restore the corrected prose. Run the named later-input-fold and multi-output
tests, then fmt, compiler check/Clippy/rustdoc/doctests, `tkt lint`,
`make citations`, `git diff --check`, and exact-base `tkt guard`.

## Negative control — 2026-08-16

Temporarily replacing the corrected source anchor with `Its remainder is a
fold` made the unchanged first check print the reached subject and exit 1:

```text
79://! **Its remainder is a fold, but this fixture does not own that
```

The corrected prose was restored before the gates below.
