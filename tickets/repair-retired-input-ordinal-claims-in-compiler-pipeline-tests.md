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

## Required work

Preserve the behavioral claims and fixtures; attribute declared association and canonical ordering to the retained checked request subject and exact `AccessOrdinal` projection.

Run the named epilogue tests, a source-phrase negative perturbation, `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard`.
