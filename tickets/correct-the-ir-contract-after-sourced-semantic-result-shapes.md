---
id: correct-the-ir-contract-after-sourced-semantic-result-shapes
title: Correct the IR contract after sourced semantic result shapes
status: todo
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, correction]
---
## Why this exists

The semantic boundary has moved farther than the foundation contract says. `ValueFact` stores a `SourcedShape`, `SemanticProgramBuilder::push_operation` preserves inferred sourced result shapes, and `BuildError::SymbolicOperandUnsupported` no longer exists. [`docs/ir.md`](../docs/ir.md) still presents the former static-result boundary as current fact.

The coefficient-era audit that found this remainder could not edit `contracts/foundation`; it deliberately left this contract for an owning correction rather than making an off-scope change.

## Starting evidence, stale until re-read at this ticket's base

- `docs/ir.md`, anchor `the offset is a literal`, correctly says the current Slice selection carries a literal offset, but incorrectly derives the whole gap from static semantic shapes. `SliceAxisSelection::Window` itself has only `offset: u64`, and `decode_axis` rejects `symbolic-window` before inference.
- `docs/ir.md`, anchor `What it does not reach is an operation`, says no inferred result extent may be symbolic and names `BuildError::SymbolicOperandUnsupported`.
- Later dated corrections in the same section, including anchor `a semantic value's shape is still a Shape`, preserve that obsolete boundary as present-tense fact.
- `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, and `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, are the construction and carriage authorities.
- `crates/tiler-ir/src/semantic/slice.rs`, anchors `offset: u64`, `SLICE_RELATION_SYMBOLIC_WINDOW`, and `request.static_operand_shape`, separate the missing source-bearing selection schema from Slice's later literal-operand bounds restriction.

The worker's first deliverable is a per-Fact verdict at its exact base. Read `docs/ir.md` in full, the complete construction and consumption sites, and the governing accepted shape decisions before editing. Repair this ticket if any premise has moved.

## Outcome

Add dated corrections that retire the static-`ValueFact` and removed-error account without erasing the historical reasoning. State the actual current boundary: general semantic values can carry sourced result shapes; each operation family still decides whether its schema and inference accept them; Slice presently lacks a source-bearing offset selection and separately asks for a static operand shape when checking literal bounds.

Keep maturity explicit. This is a contract correction, not authorization for a new Slice relation, inference rule, or public boundary. Re-scan the complete contract for other live conclusions derived from the same obsolete premise and classify every hit.

## Closes when

Every live static-result-shape claim in `docs/ir.md` is either corrected beside its historical text or shown current with source evidence; removed diagnostics are no longer named as current behavior; the Slice explanation distinguishes selection-schema construction from operand bounds inference; `make citations`, `tkt lint`, and `git diff --check` pass; and `tkt guard` shows no undeclared scope.
