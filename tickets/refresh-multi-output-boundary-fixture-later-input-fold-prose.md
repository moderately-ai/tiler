---
id: refresh-multi-output-boundary-fixture-later-input-fold-prose
title: Refresh the multi-output boundary fixture's retired later-input-fold prose
status: todo
priority: p3
dependencies: []
related: [refresh-multi-output-correctness-row-after-access-ordinal-reconciliation, admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, multi-output]
---
At exact base `0f04651e8a0f2c132ce37cab6b86552edc46044c`, the complete 589-line `crates/tiler-compiler/tests/multi_output_boundary.rs` still describes a later-input strict serial fold as refusing under `sum-contributor-ordinal` at anchors `Its remainder is a fold` and `The fused single-region alternative is withheld`. That claim was retired by the completed `admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`: the current positive owners are `request::tests::a_fold_over_a_later_declared_input_retains_its_ordinal` and the fused later-input conformance fixture, while this module itself owns ordered-output admission, disjoint elementwise input subsets, sharing refusals, and semantic output-order identity. Re-read the complete fixture and the current fold construction/verification/assembly consumers; update only the stale module prose to point at the landed positive owners and preserve the fixture’s actual support/refusal claims. Add a source-subject check that can fail, perturb the stale prose independently, run the relevant later-input-fold and multi-output tests, then fmt, compiler check/Clippy/rustdoc, `tkt lint`, `make citations`, `git diff --check`, and exact-base `tkt guard`. No production behavior, public surface, identity, schema, or test population changes.
