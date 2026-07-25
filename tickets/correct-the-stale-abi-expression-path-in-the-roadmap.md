---
id: correct-the-stale-abi-expression-path-in-the-roadmap
title: Correct the stale ABI expression path in the roadmap support matrix
status: todo
priority: p3
dependencies: []
related: [disambiguate-select-across-ir-layers, own-operation-family-support-matrix, relocate-abi-expressions-into-tiler-ir]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, coherence]
---
**Fact.** The `Select` row of the operation-family support matrix in `docs/roadmap.md` cites `ExprNode::Select` as living in `crates/tiler-artifact/src/program/expr.rs`. It does not. `relocate-abi-expressions-into-tiler-ir` moved the ABI expression domain to `crates/tiler-ir/src/program/abi.rs`, where `ExprNode::Select` is defined at line 443. What remains at the old path is a 24-line re-export shim whose own module documentation says so: "The domain type, its admitted roots, validation, canonical identity, and authoritative checked evaluation live in `tiler_ir::program::abi`."

**Reproduce in one line:** `grep -n 'pub enum ExprNode' crates/tiler-artifact/src/program/expr.rs` returns nothing; the same grep against `crates/tiler-ir/src/program/abi.rs` returns the definition.

**Why it is worth correcting rather than leaving.** The row's whole purpose is to stop a reader concluding tensor `Select` support from a substring search, and it does that by naming precisely where each unrelated `Select` really lives. A citation that resolves to a re-export shim weakens the one paragraph in the corpus whose job is precision about location. The row is otherwise correct and should not be rewritten.

**What closes this.** Repoint the citation at `crates/tiler-ir/src/program/abi.rs`, and reference the four `Select` glossary entries added by `disambiguate-select-across-ir-layers` rather than restating their content, so the roadmap stops being a second place where the four constructs are enumerated.

**Scope.** `docs/roadmap.md` is `contracts/navigation`. Found while closing `disambiguate-select-across-ir-layers`, which holds `contracts/foundation` only, and split out rather than reached for.
