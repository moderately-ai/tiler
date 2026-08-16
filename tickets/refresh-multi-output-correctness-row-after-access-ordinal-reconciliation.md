---
id: refresh-multi-output-correctness-row-after-access-ordinal-reconciliation
title: Refresh the multi-output correctness row after access-ordinal reconciliation
status: todo
priority: p2
dependencies: []
related: [repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
The live paragraph in docs/correctness-and-testing.md at anchor `The multi-output row is now positive` still says a fold over a later declared input refuses under sum-contributor-ordinal and says contraction consumers rely on dense declaration ordinals zero and one. Exact current source includes request::tests::a_fold_over_a_later_declared_input_retains_its_ordinal and the completed AccessOrdinal/DeclaredInputOrdinal reconciliation. Read the complete document and current owning tests/tickets, audit every present-tense support claim in that paragraph, then correct only claims superseded by landed work while preserving dated measurements and historical corrections. Run a source-claim negative perturbation, tkt lint, make citations, git diff --check, and exact-base tkt guard.
