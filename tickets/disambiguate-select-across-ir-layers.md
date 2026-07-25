---
id: disambiguate-select-across-ir-layers
title: Disambiguate the four different Select constructs in the corpus
status: done
priority: p2
dependencies: []
related: [own-operation-family-support-matrix]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, glossary, coherence]
claimed_from: todo
assignee: agent-foundation
lease_expires_at: 1784996298
---
The name `Select` denotes four unrelated constructs in this repository, and nothing tells a reader which one is meant.

- **Shape-metadata `Select`.** `docs/research/shapes/shape-environment-contract.md` defines a ternary `Select` over shape expressions and states explicitly that it "is not tensor `where`".
- **ABI expression `Select`.** `ExprNode::Select` in `crates/tiler-artifact/src/program/expr.rs` is a host-side launch/ABI scalar expression with lazy branch evaluation, verified in `program/verify.rs` and exposed as `AbiExprView::Select`.
- **Proposed kernel-level `Select`.** The structured-kernel-IR verifier research lists `Select` in its bounded initial operation set. That research is `disposition: adopted` but `implementation_status: spike-only`; the implemented vocabulary in `crates/tiler-ir/src/kernel/model.rs` has no `Select`.
- **Tensor `Select`.** Named in one row of the adopted [operation conformance matrix](../docs/research/numerics/operation-conformance-matrix.md) and nowhere else. No ADR, normative contract, or key defines it.

`docs/glossary.md` has no entry for any of them. The hazard is concrete and was hit while auditing operation breadth: a `grep -rnw Select crates/` returns fourteen `tiler-artifact` hits and reads as evidence that a selection operation exists, when the tensor family is at the bottom rung of the [support matrix](../docs/roadmap.md#operation-family-support-matrix).

Add glossary entries that name each construct and its owning layer, or rename the ones that can still be renamed cheaply. Whichever is chosen, a future reader must be able to tell from the name or one glossary lookup which layer a `Select` belongs to. Note that `AGENTS.md` warns specifically against concluding support from a substring search; this is the case that produces that error.

## Outcome

**Done.** `docs/glossary.md` carries four rows — *Select (ABI expression)*, *Select (shape expression)*, *Select (structured kernel operation)*, and *Select (tensor)* — each naming its owning layer, its maturity, and the others as distinct. A closing advisory paragraph forbids an unqualified `Select` in normative text and diagnostics.

**All four claims were verified against source before being written into a governed contract, and one of the ticket's own citations was wrong.**

**Retraction — the ABI path.** This ticket placed `ExprNode::Select` in `crates/tiler-artifact/src/program/expr.rs`. It is not there. `relocate-abi-expressions-into-tiler-ir` moved the domain to `crates/tiler-ir/src/program/abi.rs:443`; what survives at the old path is a 24-line re-export shim that says so in its own module documentation. Reproduce: `grep -n 'pub enum ExprNode' crates/tiler-artifact/src/program/expr.rs` returns nothing. The glossary row names the real location and the shim, so a reader following either path lands correctly. `docs/roadmap.md` carries the same stale citation; it is `contracts/navigation`, which this ticket does not hold, and is split into `correct-the-stale-abi-expression-path-in-the-roadmap`.

**Retraction — the hit count.** This ticket said a `grep -rnw Select crates/` returns "fourteen `tiler-artifact` hits". At `43f685f` it returns twenty-two hits: twelve in `tiler-artifact`, nine in `tiler-ir`, one in `tiler-compiler`. The count moved because the relocation moved the definition, which is exactly how a number in a durable document goes stale, so the glossary states the invariant that matters — *every* hit is the ABI construct — and records no count.

**Confirmed unchanged.** The shape `Select` remains unimplemented: `crates/tiler-ir/src/shape/` defines `ShapeError` and the shape-symbol authority, and no `ShapeExpr`. The structured-kernel `Select` remains unimplemented: `crates/tiler-ir/src/kernel/model.rs` has `BinaryOp`, `CompareOp`, `ConvertOp`, `Builtin`, `KernelConstant` and no `Select`. The tensor `Select` is defined nowhere: `grep -rnw Select docs/decisions/` returns only ADR 0049's title, which uses the ordinary English verb, and the only governed type key registered outside tests is `tiler::f32@1`, so the predicate value type the family would need does not exist either.

**The one place two of them meet is stated, not just the four definitions.** The accepted decision on domain-specific expression IRs makes `ShapeExpr` and `AbiExpr` distinct newtyped domains that share arithmetic components without sharing identity, and specifies that lowering between them is explicit, typed, and checked. That lowering is the only contact point between the shape and ABI `Select`s, and the rows say so — otherwise a reader who noticed both are ternary, lazy, and host-evaluated could reasonably conclude they are one node seen twice.

**Rename was considered and rejected on the merits, not on cost.** Each of the four is the correct name for its construct within its own layer, and two of them are fixed by accepted decisions — the shape `Select` by the typed-lazy-shape-selection decision and the kernel `Select` by the structured-kernel operation set. Renaming would fight those. It is also unavailable here regardless: the only implemented `Select` spans `implementation/ir`, `implementation/artifact`, and `implementation/compiler`, none of which `contracts/foundation` holds.

**Sibling sweep — the defect class is wider than this term, and the rest is split rather than skipped.** `AGENTS.md` treats one name with two definitions as a class, so closing `Select` meant enumerating the operation vocabulary of all four expression layers. Six further exact-spelling collisions survive: `Minimum`/`Maximum` (tensor NaN-propagating versus `AbiBinaryOp`'s documented "Unsigned minimum" — the same grep-proves-support failure mode as `Select`, and the most hazardous of them), `Equal`, `Not`, `Constant`, `Unary`/`Binary`, and `Add`. The sweep also found that a disambiguating spelling convention already exists implicitly and is applied unevenly: the ABI language says `CheckedAdd` where the tensor family says `Add`, and the shape language says `Min`/`Max` where the tensor family says `Minimum`/`Maximum`, yet `AbiBinaryOp::Minimum` took the tensor spelling anyway. Deciding whether to qualify in the glossary or to make spellings carry the layer is a convention decision affecting three implementation scopes, so it is `disambiguate-operation-names-shared-across-expression-layers` with the full enumeration attached, not an unscoped expansion of this ticket.

**Evidence.** `uv run --locked python scripts/docs.py render`; full repository gate green.
