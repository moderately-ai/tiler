---
id: correct-the-stale-abi-expression-path-in-the-roadmap
title: Correct the stale ABI expression path in the roadmap support matrix
status: done
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

## Outcome

The `Select` row of the operation-family support matrix in `docs/roadmap.md` now cites `crates/tiler-ir/src/program/abi.rs` and defers to [the glossary](../docs/glossary.md) instead of enumerating the other three constructs itself. `uv run --locked python scripts/check_repository.py` passes. `docs/roadmap.md` is `contracts/navigation`; nothing outside it was touched, and `crates/` was read only.

**Every claim in the ticket was reproduced before acting, and one is corrected.** `grep -n 'pub enum ExprNode' crates/tiler-artifact/src/program/expr.rs` returns nothing and the same grep against `crates/tiler-ir/src/program/abi.rs` returns `423:pub enum ExprNode {`, with the `Select` variant at line 443 exactly as stated. The old path is a re-export shim carrying the module documentation the ticket quotes. The ticket calls it "a 24-line re-export shim"; `wc -l` reports 23. That is a one-line discrepancy in an incidental figure, so the outcome states no line count rather than repeating either number.

**The replacement strengthens the row rather than only shortening it.** The row exists to stop a reader concluding tensor `Select` support from a substring search. Deleting the enumeration alone would have removed the row's answer to "then what *are* those hits?", so the row now states the fact that does the work directly: exactly one of the four constructs exists in compiled code, and it is the ABI expression. That claim was verified rather than copied from the glossary — `grep -rnw Select crates/` returns 22 hits across 9 files, and each was read: 9 in `crates/tiler-ir/src/program/abi.rs`, the rest `ExprNode::Select` or its `AbiExprView::Select` projection in `tiler-artifact`'s `program/{builder,codec,model,verify}.rs` and one in `crates/tiler-compiler/src/session.rs`. No hit is a tensor, shape-expression, or kernel `Select`, which also re-confirms the row's retained claim that `crates/tiler-ir/src/kernel/model.rs` has no `Select`.

**A defect the removal introduced, caught by reading the resulting row rather than the diff.** The row's **Proposal.** sentence opened "That research is `disposition: adopted`…", whose antecedent was the structured-kernel-IR verifier research named in the enumeration being deleted. Removing the enumeration left the pronoun dangling. The sentence now names that research explicitly, which restores the reference without reintroducing the four-way enumeration the ticket asked to remove. No gate would have caught this: the row still parsed, still linked, and still validated.

**Measurement boundary.** This corrects one citation and removes one duplicated enumeration at this commit. No check ties a documentation path citation to the file it names, so the same class of staleness can recur on the next relocation; `validate_links` only resolves Markdown link targets, and this citation is a code span, not a link. Making inline `crates/...` path citations checkable is a distinct piece of work and is not claimed here.

**Scope.** `docs/roadmap.md` is `contracts/navigation`. Found while closing `disambiguate-select-across-ir-layers`, which holds `contracts/foundation` only, and split out rather than reached for.
