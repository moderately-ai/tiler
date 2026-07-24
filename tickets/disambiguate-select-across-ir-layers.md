---
id: disambiguate-select-across-ir-layers
title: Disambiguate the four different Select constructs in the corpus
status: todo
priority: p2
dependencies: []
related: [own-operation-family-support-matrix]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, glossary, coherence]
---
The name `Select` denotes four unrelated constructs in this repository, and nothing tells a reader which one is meant.

- **Shape-metadata `Select`.** `docs/research/shapes/shape-environment-contract.md` defines a ternary `Select` over shape expressions and states explicitly that it "is not tensor `where`".
- **ABI expression `Select`.** `ExprNode::Select` in `crates/tiler-artifact/src/program/expr.rs` is a host-side launch/ABI scalar expression with lazy branch evaluation, verified in `program/verify.rs` and exposed as `AbiExprView::Select`.
- **Proposed kernel-level `Select`.** The structured-kernel-IR verifier research lists `Select` in its bounded initial operation set. That research is `disposition: adopted` but `implementation_status: spike-only`; the implemented vocabulary in `crates/tiler-ir/src/kernel/model.rs` has no `Select`.
- **Tensor `Select`.** Named in one row of the adopted [operation conformance matrix](../docs/research/numerics/operation-conformance-matrix.md) and nowhere else. No ADR, normative contract, or key defines it.

`docs/glossary.md` has no entry for any of them. The hazard is concrete and was hit while auditing operation breadth: a `grep -rnw Select crates/` returns fourteen `tiler-artifact` hits and reads as evidence that a selection operation exists, when the tensor family is at the bottom rung of the [support matrix](../docs/roadmap.md#operation-family-support-matrix).

Add glossary entries that name each construct and its owning layer, or rename the ones that can still be renamed cheaply. Whichever is chosen, a future reader must be able to tell from the name or one glossary lookup which layer a `Select` belongs to. Note that `AGENTS.md` warns specifically against concluding support from a substring search; this is the case that produces that error.
