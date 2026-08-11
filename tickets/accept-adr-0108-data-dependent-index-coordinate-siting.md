---
id: accept-adr-0108-data-dependent-index-coordinate-siting
title: Accept or revise ADR 0108 on siting a data-dependent index coordinate
status: blocked
priority: p1
dependencies: [revise-adr-0108-with-a-complete-data-dependent-index-vertical]
related: [admit-the-indirect-access-class-into-the-index-layer, admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-the-selected-data-dependent-index-representation, admit-an-invocation-scoped-gather-index-validation-receipt, emit-the-indirect-gather-on-metal, accept-adr-0107-indirect-gather-semantic-family]
scopes: [contracts/decisions, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, indexing, ir, gather, verification, needs-tom]
---
## Decision outcome — returned for revision on 2026-08-08

Tom delegated the coordinator in the 2026-08-08 interactive orchestration
session to make the correct decision after independent audit. The resulting
decision is **do not accept ADR 0108 as written**. Keep it `proposed`, return it
for the revision owned by
[`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](revise-adr-0108-with-a-complete-data-dependent-index-vertical.md),
and preserve the current no-index-layer admission and typed request refusal.
This is revision provenance, not acceptance provenance.

The ticket is `blocked` on that revision rather than `done` or
`awaiting-decision`. A terminal acceptance ticket would satisfy dependents and
make backend emission look dependency-ready even though no representation has
been selected, accepted, or admitted.

## Source-first Fact audit at `ceda5be0be458e527b7cf1ed604f3c503db12015`

The draft's Facts were re-read at the claimed base before edits.

- **Verified — current negative boundary.** In
  `crates/tiler-ir/src/index/model.rs`, `pub(super) enum IndexNode` has five forms
  and public `IndexExprClass` has three. In
  `crates/tiler-ir/src/index/predicate.rs`, `IndexDomainUnknownReason` has three
  variants. `the_index_expression_vocabulary_admits_no_data_dependent_form`
  sizes the live census from those types. No node reads tensor data, and a gather
  reaches no index region or scheduled relation.
- **False — every access-level representation moves old identity bytes.**
  `encode_region` in `crates/tiler-ir/src/index/builder/identity.rs` already
  writes an explicit leading `AccessMode` tag: `1` for reads and `2` for writes.
  A fresh tag `3` with a framed payload can preserve all old bytes. This does not
  select that representation; it keeps the candidate open for comparison.
- **Imprecise — seven `zip` sites silently lose axes.** The seven sites exist in
  `crates/tiler-ir/src/index/builder/proof.rs`, but `IndexRegionBuilder::prepare_access`
  first checks `coordinates.len() == tensor_data.shape.rank()` and returns
  `IndexBuildError::AccessRank` otherwise. The current consumers receive an
  established same-length invariant. A future representation must state its own
  rank rule, but the census does not choose a representation.
- **False — every existing unknown reason promises later closure.** The docs on
  `IndexDomainUnknownReason` say that admitted facts permit models on both sides,
  that the current engine does not decide a fragment, or that a deterministic
  lane hit a resource limit. They do not promise that more facts, a stronger
  engine, or a larger budget will close every obligation.
- **False — a gather bound is undecidable in principle.** ADR 0107 permits static
  proof or validation at a named boundary. `decide_gather_index` in
  `crates/tiler-ir/src/semantic/gather.rs` is factored for reuse by a future
  host-side pre-dispatch validator. The current shape-only verifier lacks the
  tensor element; that does not make the semantic precondition undecidable in
  every environment and does not require a fourth unknown reason.
- **Incomplete — the proposed expression node is a complete additive form.**
  `mark_expr`, `visit_expression_dimensions`, `remap_node`, the alpha-key
  construction, both identity encoders, proof evaluation, and the reference
  oracle are exhaustive over nodes whose children are index expressions or
  sourced extents. A tensor-read node would be a nested logical read and needs a
  source tensor and coordinate bounds, reachability, resolved type and `u32`
  semantics, proof ownership, compaction, identity, authoring, reference, and
  compiler explanation that the draft did not define.
- **False — the public boundary consists of four named widenings.** `IndexNode`
  is private to the index implementation. The public inspection form is
  `IndexExprView`, while construction would need an `IndexRegionBuilder` method
  and error surface. The previous list counted a private enum and omitted public
  authoring and validation consequences.
- **False — emission can trigger its own prerequisite.** The Metal emission
  ticket was blocked on this acceptance and on an admitted IR. It therefore
  cannot be the event that justifies the prior design decision. The order is
  revision/design, acceptance, separate IR admission, then emission; the integer
  storage carrier is an independent prerequisite.

## Standing boundary

ADR 0107 remains accepted: gather is a semantic family and nothing below it.
ADR 0046's rejection of tensor-data-derived indices in the current index language
remains in force. The exact 5/3/3 population checks remain useful regression
guards, without reserving an expression route or a fourth unknown reason.

## What closes this ticket

The revision dependency must first deliver a source-audited, complete vertical
comparison. This ticket can then record the resulting decision and accurate
provenance. If a representation is accepted, the coordinator must file and add
the separate IR-admission implementation dependency to
[`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) before
that ticket can leave `blocked`. Acceptance alone must not make emission ready.
