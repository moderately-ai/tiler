---
id: revise-adr-0108-with-a-complete-data-dependent-index-vertical
title: Revise ADR 0108 with a complete data-dependent index vertical
status: awaiting-decision
priority: p1
dependencies: [accept-adr-0109-fail-closed-on-unknown-index-domain-proof]
related: [accept-adr-0108-data-dependent-index-coordinate-siting, admit-the-indirect-access-class-into-the-index-layer, emit-the-indirect-gather-on-metal, admit-a-storage-carrier-for-integer-program-inputs]
scopes: [contracts/decisions, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, indexing, gather, verification, design, needs-tom]
---
## User-visible outcome

ADR 0108 is revised from a complete, source-audited vertical comparison of a
first-class verified nested read/value expression and an append-only tagged access
representation. The result either selects one coherent logical representation or
defers with a non-circular trigger; it does not implement either form.

## Facts at the creating base

- ADR 0107 admits `tiler::gather-f32@1` only at the semantic layer. The index
  language has five `IndexNode` forms, three `IndexExprClass` members, and three
  `IndexDomainUnknownReason` variants, and no form reads tensor data.
- `decide_gather_index` defines the exact data-dependent bounds check and is
  factored for reuse by a future host-side pre-dispatch validator. The current
  reference evaluator is the only named enforcement boundary.
- `encode_region` already tags existing read and write accesses separately. A
  fresh access tag can be append-only and preserve old bytes, subject to a
  complete representation and injectivity proof.
- `IndexRegionBuilder::prepare_access` enforces coordinate-count/rank equality
  before the verifier's coordinate/extent `zip` consumers run.
- `IndexNode` is private. Public construction and inspection run through
  `IndexRegionBuilder`, builder errors, and `IndexExprView`.
- No `LogicalAccess`, realization law, compiler recognition route, or backend
  construct realizes an indirect gather today.
- ADR 0109 decision 2 requires every retained index-domain obligation to be
  proved before executable coverage. Decision 4 records that ADR 0109 added no
  run-time validation, fallback, or identity widening; it documents the absence
  of current authority rather than a prohibition a future ADR must supersede. A
  host/per-dispatch result is not timeless program proof.

## Required comparison

For each candidate, derive and compare the complete contract for:

- the outer gathered coordinate, the nested source tensor, and every source
  coordinate expression;
- static proof versus named host validation, including what evidence belongs to
  timeless program identity and what belongs only to one dispatch;
- conformance to ADR 0109's refusal-before-coverage boundary: each candidate must
  prove every retained obligation before executable coverage, or explicitly
  return decision 2 to Tom for supersession before selecting a route that depends
  on run-time validation; decision 4 instead establishes that a new decision must
  supply the run-time and identity authority absent today;
- the semantic `tiler::u32@1` index value, conversion to any target address
  width, overflow behavior, and refusal of signed or lossy interpretations;
- bounds for both the outer access and nested source read, rank equality,
  reachability, aliasing, and the exact subject of every predicate, proof,
  disproof, or residual obligation;
- expression and tensor reachability, compaction, remapping, alpha-equivalence,
  canonical ordering, canonical encoding, and identity-domain consequences;
- public authoring, view, typed errors, validation, and the exact labelled-draft
  boundary under ADR 0075;
- reference construction and evaluation, including reuse of
  `decide_gather_index` and agreement diagnostics;
- compiler recognition, IR-owned discharge or host validation, typed refusal,
  explanation, feasibility, and lowering;
- the relation to scheduled `LogicalAccess` while keeping logical access meaning
  separate from physical storage addressing; and
- graph sequencing: design acceptance, a separately scoped IR-admission ticket,
  the integer storage carrier, and only then Metal emission.

The expression candidate must treat a tensor read as a nested logical read/value,
not as an untyped arithmetic leaf. The access candidate must state the exact fresh
tag and framing, demonstrate old-byte preservation, and show how predicates and
proof subjects name an indirect axis without weakening existing direct accesses.

## Deliverables

- A revised, still-`proposed` ADR 0108 body that removes the returned premises,
  states the strongest case and counterpoint for each candidate, and contains a
  verbatim-landable decision section for Tom.
- A source-first Fact audit citing searchable symbols or source-safe phrases for
  every construction, validation, proof, identity, compaction, reference,
  compiler, schedule, and graph claim.
- Contract updates to `docs/ir.md` and Q-SHAPE-007 that match the revised proposal
  without presenting it as accepted.
- If the proposal selects a form, a separate implementation/admission ticket with
  the owning code scopes and an explicit dependency added to the Metal emitter.
  This design ticket itself admits and implements nothing.

## Non-goals

Implementing an index form, a schedule relation, host validation, compiler
lowering, or backend emission. Scatter and data-dependent output shapes. Choosing
a public boundary without Tom's separate ADR 0075 decision.

## Stop conditions

Stop and return to Tom if the comparison exposes a consequential public-boundary
choice, a change to ADR 0046's accepted guarantee, an identity-domain step, or an
evidence claim that cannot be represented without changing what a verified region
means. Also stop if a candidate cannot satisfy ADR 0109 before executable coverage:
host validation cannot silently become timeless proof, and superseding ADR 0109
decision 2 is Tom's decision. Decision 4 requires no supersession, but it confirms
that a new run-time or identity contract would also need Tom's authority. Do not
disguise either decision as implementation detail.

## Closes when

The complete comparison and revised proposal are coherent, source-audited, and
ready for the acceptance ticket's decision. Closure does not satisfy or unblock
backend emission by itself.
