---
id: correct-the-ir-contract-after-sourced-semantic-result-shapes
title: Correct the IR contract after sourced semantic result shapes
status: in-progress
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, correction]
claimed_from: todo
assignee: w-sol-foundation
lease_expires_at: 1786210891
---
## Why this exists

The semantic boundary has moved farther than the foundation contract says. `ValueFact` stores a `SourcedShape`, `SemanticProgramBuilder::push_operation` preserves inferred sourced result shapes, and `BuildError::SymbolicOperandUnsupported` no longer exists. [`docs/ir.md`](../docs/ir.md) still presents the former static-result boundary as current fact.

The coefficient-era audit that found this remainder could not edit `contracts/foundation`; it deliberately left this contract for an owning correction rather than making an off-scope change.

## Starting evidence, stale until re-read at this ticket's base

- `docs/ir.md`, anchor `the offset is a literal`, correctly says the current Slice selection carries a literal offset, but incorrectly derives the whole gap from static semantic shapes. `SliceAxisSelection::Window` itself has only `offset: u64`, and `decode_axis` rejects `symbolic-window` before inference.
- `docs/ir.md`, anchor `What it does not reach is an operation`, says no inferred result extent may be symbolic and names `BuildError::SymbolicOperandUnsupported`.
- Later dated corrections in the same section, including source-safe anchor `gap it names is unchanged`, preserve that obsolete boundary as present-tense fact.
- `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, and `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, are the construction and carriage authorities.
- `crates/tiler-ir/src/semantic/slice.rs`, anchors `offset: u64`, `SLICE_RELATION_SYMBOLIC_WINDOW`, and `request.static_operand_shape`, separate the missing source-bearing selection schema from Slice's later literal-operand bounds restriction.

The worker's first deliverable is a per-Fact verdict at its exact base. Read `docs/ir.md` in full, the complete construction and consumption sites, and the governing accepted shape decisions before editing. Repair this ticket if any premise has moved.

## Fact audit — 2026-08-08 at `96e867a3ec2d370ccac42ebb1273073d45f1effa`

- **Verified as stale contract:** `docs/ir.md`, source anchor `the offset is a literal`, still gives Slice's literal offset the obsolete general-static semantic-shape rationale. `SliceAxisSelection::Window` has `offset: u64`; `decode_axis` refuses `SLICE_RELATION_SYMBOLIC_WINDOW` before inference; and `SliceF32::infer` later calls `request.static_operand_shape(0)` for the operand-dependent literal-bounds and result-shape check. Those are separate family boundaries.
- **False as current behavior:** `docs/ir.md`, source anchor `What it does not reach is an operation`, says inferred result extents cannot be symbolic, says `ValueFact` carries `Shape`, and names `BuildError::SymbolicOperandUnsupported`. At this base `ValueFact` stores `SourcedShape`; `SemanticProgramBuilder::push_operation` supplies stored sourced operand shapes and the program's `ExtentSources` to inference, then stores the returned sourced result; and the complete `BuildError` definition has no such variant.
- **False, with this ticket's anchor repaired:** the rendered phrase `a semantic value's shape is still a Shape` did not occur byte-for-byte because the source marks `Shape` as code. The shortest source-safe anchor is `gap it names is unchanged`; its static-`ValueFact` conclusion is false, as is the adjacent claim that the former input-only limitations “remain true for an inferred result.”
- **Verified:** `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, and `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, are the construction and carriage authorities. The registry entry point `infer_operation_with_extent_sources` shows that the environment is available while each operation family retains its own inference decision.
- **Verified:** Slice has no source-bearing offset in its typed selection schema and refuses the reserved symbolic relation before inference; independently, its inference requires the operand shape to be static. This supports a family-specific correction, not a new Slice schema or inference rule.
- **False adjacent maturity claim:** the relocation note says the five `tiler_ir::shape` paths “are not accepted.” [`accept-the-sourced-extent-vocabulary-at-its-shape-module-paths`](accept-the-sourced-extent-vocabulary-at-its-shape-module-paths.md) records Tom's acceptance on 2026-08-07. The newer sourced-result inference surface remains a labelled draft in [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md), which is still `awaiting-decision`; these are different boundaries.
- **Verified conclusion, false rationale:** no symbolic semantic program reaches a physical plan or packaged artifact because the physical and artifact construction paths refuse symbolic interfaces (and compiler normalization refuses the program), not because semantic results are generally static. That conclusion remains; only its premise changes.
- **Verified separate remainder:** Slice's identity-bearing normative definition still contains coefficient-era language. [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md) already owns the required definition and identity recomputation, so this contract-only ticket must not edit it.

## Outcome

Add dated corrections that retire the static-`ValueFact` and removed-error account without erasing the historical reasoning. State the actual current boundary: general semantic values can carry sourced result shapes; each operation family still decides whether its schema and inference accept them; Slice presently lacks a source-bearing offset selection and separately asks for a static operand shape when checking literal bounds.

Keep maturity explicit. This is a contract correction, not authorization for a new Slice relation, inference rule, or public boundary. Re-scan the complete contract for other live conclusions derived from the same obsolete premise and classify every hit.

## Closes when

Every live static-result-shape claim in `docs/ir.md` is either corrected beside its historical text or shown current with source evidence; removed diagnostics are no longer named as current behavior; the Slice explanation distinguishes selection-schema construction from operand bounds inference; `make citations`, `tkt lint`, and `git diff --check` pass; and `tkt guard` shows no undeclared scope.
