---
id: accept-adr-0099-elementary-family-projection
title: Accept or reject the elementary-family projection ADR
status: done
priority: p2
dependencies: [land-the-elementary-family-projection-adr]
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, complete-the-elementary-projection-adr-frontmatter]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, optimizer, ir, decision, needs-tom]
---
## User-visible outcome

[ADR 0099](../docs/decisions/0099-project-an-elementary-familys-per-point-body-from-one-shared-statement.md) moves from `proposed` to `accepted`, or is rejected.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision` — a parked state `tkt ready` excludes and that never satisfies a dependent; an agent that finds it in `todo` should set it back and do nothing else. It is filed in the shape [`accept-adr-0098-inline-delivery-statement`](accept-adr-0098-inline-delivery-statement.md) established, because the same asymmetry applies: landing a proposed record is a completed outcome, so the carrier ticket goes terminal the moment the file exists and cannot distinguish "written" from "decided".

**This node carries the scopes its own acceptance sweep needs.** The sweep touches `docs/decisions/[0-9]*.md` (`contracts/decisions`) and both catalog views in `docs/decisions/README.md` (`contracts/navigation`), which is what is declared above. It deliberately declares no research or contract scope — see "What acceptance does and does not do".

## What is being decided

**The route is implemented and the code is not in question here.** [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) derived it, implemented it, and landed it: `tiler::silu-f32@1` compiles through `tiler_compiler::session`, the composition is stated once in `crates/tiler-compiler/src/elementary.rs` and driven into both realizations, and perturbing the shared statement was watched failing at `compile.lowering.refinement-refused`. Rejecting the record would not unwind any of that.

**What is being decided is the rule the record states over a class.** ADR 0099's Decision is "a family is admissible at the request boundary when its per-point body is expressible in `PointwiseF32Node`", with the body written once against an abstract per-point sink. That is a growth rule for the physical region vocabulary in both directions — an elementary family does not earn a node, and an open registry-driven `ScalarOpKey` body is rejected on `PointwiseF32Node`'s closedness — and it binds the next elementary family, not only the one implemented. Accepting it makes that rule the standing answer; rejecting it returns the question to per-family judgment and leaves the SiLU implementation as one instance rather than an instance of a rule.

**One item is worth Tom's attention specifically.** `implementation_status` is `partial` because the rule is implemented for exactly one named family: `ElementwiseFamily` is the closed enum `Add`, `Multiply`, `Silu`, and `silu_point_body` is the only body in the crate. So the generality the Decision states has never been exercised by a second family. If Tom's view is that a rule over a class should be catalogued only once a second member has exercised it, that is the item to say so on.

## Provenance

**No acceptance has been relayed at any point, and this node is the first place one can be.** The deriving ticket wrote the record as `proposed` and said so; [`complete-the-elementary-projection-adr-frontmatter`](complete-the-elementary-projection-adr-frontmatter.md) amended only the frontmatter block and relayed none; [`land-the-elementary-family-projection-adr`](land-the-elementary-family-projection-adr.md) transferred the body byte-identically and relayed none. **Nothing has been released on the record and no contract sentence has been rewritten under it**, which is what keeps the rollback below cheap.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted` on ADR 0099 and updates both catalog views in [the decisions index](../docs/decisions/README.md) — the theme row under "Physical planning and lowering" and the chronology row — from `proposed` to `accepted`.

**It moves no research record's frontmatter.** ADR 0099 cites [the L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) as `evidence`, for the measurement that the two SiLU spellings are different binary32 functions. That record did not propose the projection rule, and [the metadata contract](../docs/document-metadata.md) states that "`evidence`, `informs`, and `adopted_by` are independent predicates: evidence may support a decision without that decision adopting the report's proposal". **That record carries no `adopted_by` field at all** — `grep -n '^adopted_by' docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md` reports no match, against `grep -n '^informs' …` on the same file reporting line 12 — so there is no field to move and its `disposition` does not move either.

**It moves no contract frontmatter.** A contract's inbound-ADR link is the derived `governed_by`, which the metadata contract declares "invalid in stored v1 frontmatter"; the edge is stored on ADR 0099's own `applies_to` and nowhere else. [The optimizer contract](../docs/compiler/optimizer.md) is already `contract_status: accepted`, [the IR contract](../docs/ir.md) is `mixed`, and this record changes neither.

**It implements nothing and changes no public surface.** It admits no family, widens no vocabulary, moves no version string, no encoding, and no pinned identity.

**It releases nothing, and that is checked against the board rather than asserted.** No ticket other than this one depends on the carrier: `grep -rn "^dependencies:.*land-the-elementary-family-projection-adr" tickets/ | grep -v accept-adr-0099-elementary-family-projection` reports no match, over a `tickets/` population of 823 files. The excluding form is the one to use — without it the self-reference on this node's own `dependencies:` line is a hit that cannot say no, which is the failure the 0098 node had to correct.

## Rollback, kept cheap on purpose

If the record is rejected after being accepted, the repair is one field and two catalog rows: `decision_status` back to `proposed`, the theme and chronology rows back to `proposed`, and this node back to `awaiting-decision`. Nothing else moves.

Rejecting the record outright is a deletion of one file plus its two catalog rows and this node. It leaves the implemented route exactly where it is — in `crates/tiler-compiler/src/elementary.rs` and in the deriving ticket's derivation — as an instance rather than a rule.

## Closes when

Tom accepts or rejects it.

## Decided — accepted

Accepted by Tom on 2026-08-05 at the live decision review in the coordination session, witnessed first-hand by the coordinator, with the one-family caveat presented and accepted: a second family that refutes the rule supersedes it explicitly. Sweep executed in the same change: `decision_status` flipped and both catalog views updated; no research or contract frontmatter moves, per this ticket's own derivation.
